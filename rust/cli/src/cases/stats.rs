//! Generation stats cases.

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::task::Poll;

use nyaaai_agentic::transport::KeyAuth;
use nyaaai_agentic::usage::stats::{StatsQueue, StatsQueueDeps, stats_of};
use nyaaai_contract::cancel::CancelToken;
use nyaaai_contract::future::BoxFuture;
use nyaaai_contract::json::Value;
use nyaaai_contract::shapes::step::NormalizedUsage;
use nyaaai_contract::shapes::usage::{CallTag, Cost, GenerationStats, MeteredCall, UnfilledRecord};
use nyaaai_contract::time::{Clock, Scheduled, Scheduler, Slept};
use nyaaai_contract::usage_sink::UsageSink;
use nyaaai_testkit::cases::{Outcome, block_on};
use nyaaai_testkit::scheduler::{FakeScheduler, TimerWheel};
use nyaaai_testkit::scripted_ports::ScriptedUsageSink;
use tokio::sync::{oneshot, watch};

use super::catalog::network_of;
use super::shapes::*;

fn locked<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub fn decode(input: StatsDecodeInput) -> Outcome<StatsDecodeResult> {
    Ok(StatsDecodeResult {
        stats: input.bodies.iter().map(stats_of).collect(),
    })
}

/// A call as a log would hold it, made at `at`.
fn call_at(at: i64) -> MeteredCall {
    MeteredCall {
        tag: CallTag {
            feature: "chat".to_owned(),
            lane: None,
            folder: None,
        },
        model: "m".to_owned(),
        entry: None,
        label: None,
        at: at as f64,
        ms: 0.0,
        first_token_ms: None,
        usage: None::<NormalizedUsage>,
        cost: Cost {
            usd: 0.0,
            estimated: false,
            unpriced: None,
        },
        provider: None,
        generation_id: None,
        finish_reason: None,
        failure: None,
        cancelled: None,
    }
}

/// A timer of the queue's scheduler, as the wheel holds it.
struct Live {
    at: i64,
    on: bool,
}

/// A script timer: an action, a held answer, a late read.
struct Wait {
    at: i64,
    done: bool,
}

/// The wheel's scheduler, noting the timers the queue sets.
struct Tracking {
    inner: Arc<FakeScheduler>,
    live: Arc<Mutex<Vec<Live>>>,
}

impl Clock for Tracking {
    fn now_ms(&self) -> i64 {
        self.inner.now_ms()
    }

    fn utc_offset_minutes(&self, at_ms: i64) -> i64 {
        self.inner.utc_offset_minutes(at_ms)
    }
}

impl Scheduler for Tracking {
    fn sleep(&self, ms: i64, cancel: &CancelToken) -> BoxFuture<'static, Slept> {
        self.inner.sleep(ms, cancel)
    }

    fn schedule(&self, ms: i64, run: Box<dyn FnOnce() + Send>) -> Scheduled {
        let at = self.inner.now_ms() + ms;
        let at_index = {
            let mut live = locked(&self.live);
            live.push(Live { at, on: true });
            live.len() - 1
        };
        let fired = Arc::clone(&self.live);
        let cancel = self.inner.schedule(
            ms,
            Box::new(move || {
                locked(&fired)[at_index].on = false;
                run();
            }),
        );
        let held = Arc::clone(&self.live);
        Scheduled::new(move || {
            locked(&held)[at_index].on = false;
            cancel.cancel();
        })
    }
}

/// What the script shares with the callbacks it hands the queue.
struct Script {
    wheel: Arc<TimerWheel>,
    /// Names the script's own timers: actions, held answers, late reads.
    timers: AtomicUsize,
    waits: Mutex<Vec<Wait>>,
    /// Counts the answers held: a queue waiting on one is parked.
    parked: watch::Sender<u64>,
    made: AtomicU64,
}

impl Script {
    fn wait_on(script: &Arc<Script>, ms: i64, name: &str, run: Box<dyn FnOnce() + Send>) {
        let timers = script.timers.fetch_add(1, Ordering::SeqCst) + 1;
        let at = script.wheel.now_ms() + ms;
        let index = {
            let mut waits = locked(&script.waits);
            waits.push(Wait { at, done: false });
            waits.len() - 1
        };
        let script = Arc::clone(script);
        script.wheel.clone().after_run(
            ms,
            &format!("{name}-{timers}"),
            Box::new(move || {
                locked(&script.waits)[index].done = true;
                run();
            }),
        );
    }
}

/// The scripted sink, failing and holding as told: the call is kept, then it panics.
struct Failing {
    inner: Arc<ScriptedUsageSink>,
    fill: Vec<String>,
    filled: Vec<String>,
    unfilled: bool,
    held: Vec<StatsRunInputHoldFilledItem>,
    script: Arc<Script>,
}

impl UsageSink for Failing {
    fn record(&self, call: &MeteredCall) -> Option<String> {
        self.inner.record(call)
    }

    fn fill(&self, id: &str, stats: &GenerationStats) {
        self.inner.fill(id, stats);
        if self.fill.iter().any(|held| held == id) {
            std::panic::resume_unwind(Box::new("fill failed"));
        }
    }

    fn filled(&self, id: &str, at: i64) -> BoxFuture<'_, bool> {
        let asked = self.inner.filled(id, at);
        let fails = self.filled.iter().any(|held| held == id);
        let hold = self
            .held
            .iter()
            .find(|held| held.id == id)
            .map(|held| held.ms);
        let script = Arc::clone(&self.script);
        Box::pin(async move {
            let answer = asked.await;
            if fails {
                std::panic::resume_unwind(Box::new("filled failed"));
            }
            match hold {
                Some(Some(ms)) => {
                    let (send, wait) = oneshot::channel::<()>();
                    Script::wait_on(
                        &script,
                        ms as i64,
                        "hold",
                        Box::new(move || {
                            let _ = send.send(());
                        }),
                    );
                    script.parked.send_modify(|count| *count += 1);
                    let _ = wait.await;
                }
                Some(None) => {
                    script.parked.send_modify(|count| *count += 1);
                    std::future::pending::<()>().await;
                }
                None => {}
            }
            answer
        })
    }

    fn unfilled(&self, since: i64) -> BoxFuture<'_, Vec<UnfilledRecord>> {
        let asked = self.inner.unfilled(since);
        let fails = self.unfilled;
        Box::pin(async move {
            let answer = asked.await;
            if fails {
                std::panic::resume_unwind(Box::new("unfilled failed"));
            }
            answer
        })
    }
}

/// What happens at a wheel time inside a step.
enum Act {
    Add {
        id: String,
        generation_id: String,
        at: Option<i64>,
        entry: Option<String>,
    },
    AddMany {
        count: u64,
        at: Option<i64>,
        entry: Option<String>,
    },
    Dispose,
}

fn act(queue: &StatsQueue, script: &Script, held: &Act) {
    match held {
        Act::Add {
            id,
            generation_id,
            at,
            entry,
        } => queue.add(id, generation_id, *at, entry.as_deref()),
        Act::AddMany { count, at, entry } => {
            for _ in 0..*count {
                let made = script.made.fetch_add(1, Ordering::SeqCst) + 1;
                queue.add(
                    &format!("r{made}"),
                    &format!("gen-r{made}"),
                    *at,
                    entry.as_deref(),
                );
            }
        }
        Act::Dispose => queue.dispose(),
    }
}

/// How a settle ended.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Settled {
    Idle,
    Parked,
}

/// Waits for the queue to end its read or wait on a held answer.
async fn settle(queue: &StatsQueue, script: &Script) -> Settled {
    let mut parked = script.parked.subscribe();
    parked.borrow_and_update();
    let mut idle: BoxFuture<'_, ()> = Box::pin(queue.idle());
    let mut changed: BoxFuture<'_, bool> = Box::pin(async { parked.changed().await.is_ok() });
    std::future::poll_fn(|context| {
        if idle.as_mut().poll(context).is_ready() {
            return Poll::Ready(Settled::Idle);
        }
        match changed.as_mut().poll(context) {
            Poll::Ready(_) => Poll::Ready(Settled::Parked),
            Poll::Pending => Poll::Pending,
        }
    })
    .await
}

type Mark = (i64, usize, usize, usize, usize, usize);

pub fn run(input: StatsRunInput) -> Outcome<StatsRunResult> {
    let net = network_of(input.start_ms, &input.exchanges, None)?;
    block_on(play(input, net))?
}

async fn play(input: StatsRunInput, net: super::catalog::Net) -> Outcome<StatsRunResult> {
    let sink = Arc::new(ScriptedUsageSink::new());
    let known = input.known.clone().unwrap_or_default();
    for held in &known {
        sink.set_ids(vec![held.id.clone()]);
        sink.record(&call_at(held.at));
    }
    sink.set_filled(known.iter().map(|held| held.id.clone()).collect());
    sink.set_unfilled(input.unfilled.clone().unwrap_or_default());
    let fails = input.fails.clone();
    let script = Arc::new(Script {
        wheel: Arc::clone(&net.wheel),
        timers: AtomicUsize::new(0),
        waits: Mutex::new(Vec::new()),
        parked: watch::channel(0).0,
        made: AtomicU64::new(0),
    });
    let live: Arc<Mutex<Vec<Live>>> = Arc::default();
    let usage: Arc<dyn UsageSink> = Arc::new(Failing {
        inner: Arc::clone(&sink),
        fill: fails
            .as_ref()
            .and_then(|held| held.fill.clone())
            .unwrap_or_default(),
        filled: fails
            .as_ref()
            .and_then(|held| held.filled.clone())
            .unwrap_or_default(),
        unfilled: fails.as_ref().and_then(|held| held.unfilled) == Some(true),
        held: input.hold_filled.clone().unwrap_or_default(),
        script: Arc::clone(&script),
    });
    let reads = Arc::new(Mutex::new(Vec::<StatsRunResultStepsItemReadsItem>::new()));
    let late = input.late.clone().unwrap_or_default();
    let queue = {
        let transport = Arc::clone(&net.transport);
        let wheel = Arc::clone(&net.wheel);
        let lines = Arc::clone(&reads);
        let script = Arc::clone(&script);
        StatsQueue::new(StatsQueueDeps {
            scheduler: Arc::new(Tracking {
                inner: net.scheduler.clone(),
                live: Arc::clone(&live),
            }),
            usage,
            read: Arc::new(move |entry, path, cancel| {
                let transport = Arc::clone(&transport);
                let wheel = Arc::clone(&wheel);
                let lines = Arc::clone(&lines);
                let script = Arc::clone(&script);
                let held = late
                    .iter()
                    .find(|held| path.ends_with(&format!("={}", held.generation_id)))
                    .cloned();
                Box::pin(async move {
                    let at_ms = wheel.now_ms();
                    if let Some(held) = held {
                        // Written as the read ends, though the queue stopped waiting.
                        let (send, wait) = oneshot::channel::<Value>();
                        let (ended, line) = (Arc::clone(&wheel), Arc::clone(&lines));
                        let body = held.body.clone();
                        Script::wait_on(
                            &script,
                            held.ms as i64,
                            "late",
                            Box::new(move || {
                                locked(&line).push(StatsRunResultStepsItemReadsItem {
                                    at_ms,
                                    ended_ms: ended.now_ms(),
                                    entry,
                                    path,
                                    failed: false,
                                });
                                let _ = send.send(body);
                            }),
                        );
                        script.parked.send_modify(|count| *count += 1);
                        return wait.await.map_err(|_| "the read was dropped".to_owned());
                    }
                    let answer = transport.get_json(&path, &cancel, &KeyAuth::Stored).await;
                    locked(&lines).push(StatsRunResultStepsItemReadsItem {
                        at_ms,
                        ended_ms: wheel.now_ms(),
                        entry,
                        path,
                        failed: answer.is_err(),
                    });
                    answer.map_err(|error| error.to_string())
                })
            }),
        })
        .map_err(|error| error.to_string())?
    };
    let times = |live: &Mutex<Vec<Live>>, script: &Script, target: i64| -> Vec<i64> {
        let mut times: Vec<i64> = locked(live)
            .iter()
            .filter(|each| each.on)
            .map(|each| each.at)
            .chain(
                locked(&script.waits)
                    .iter()
                    .filter(|each| !each.done)
                    .map(|each| each.at),
            )
            .filter(|at| *at <= target)
            .collect();
        times.sort_unstable();
        times
    };
    let mark = |queue: &StatsQueue| -> Mark {
        let calls = sink.calls();
        (
            net.wheel.now_ms(),
            calls.record,
            calls.fill,
            calls.filled,
            calls.unfilled,
            queue.queued().len(),
        )
    };
    let mut steps = Vec::new();
    for step in &input.steps {
        let reads_before = locked(&reads).len();
        let fills_before = sink.fills().len();
        match step {
            StatsRunInputStepsItem::Add(held) => act(
                &queue,
                &script,
                &Act::Add {
                    id: held.id.clone(),
                    generation_id: held.generation_id.clone(),
                    at: held.at,
                    entry: held.entry.clone(),
                },
            ),
            StatsRunInputStepsItem::AddMany(held) => act(
                &queue,
                &script,
                &Act::AddMany {
                    count: held.count,
                    at: held.at,
                    entry: held.entry.clone(),
                },
            ),
            StatsRunInputStepsItem::Advance(held) => {
                // Stops at each instant the queue or an action is set for.
                let target = net.wheel.now_ms() + held.ms as i64;
                for each in held.during.iter().flatten() {
                    if each.after_ms > held.ms {
                        continue;
                    }
                    let what = match &each.do_ {
                        StatsRunInputStepsItemAdvanceDuringItemDo::Add(add) => Act::Add {
                            id: add.id.clone(),
                            generation_id: add.generation_id.clone(),
                            at: add.at,
                            entry: add.entry.clone(),
                        },
                        StatsRunInputStepsItemAdvanceDuringItemDo::AddMany(many) => Act::AddMany {
                            count: many.count,
                            at: many.at,
                            entry: many.entry.clone(),
                        },
                        StatsRunInputStepsItemAdvanceDuringItemDo::Dispose(_) => Act::Dispose,
                    };
                    let (queue, script_held) = (queue.clone(), Arc::clone(&script));
                    Script::wait_on(
                        &script,
                        each.after_ms as i64,
                        "during",
                        Box::new(move || act(&queue, &script_held, &what)),
                    );
                }
                let mut last;
                loop {
                    let ahead = times(&live, &script, target);
                    let stop = ahead.first().copied().unwrap_or(target);
                    let before = (mark(&queue), ahead.clone());
                    net.wheel.advance((stop - net.wheel.now_ms()).max(0));
                    last = settle(&queue, &script).await;
                    if ahead.is_empty() {
                        break;
                    }
                    if (mark(&queue), times(&live, &script, target)) == before {
                        return Err("the advance made no progress".to_owned());
                    }
                }
                if last == Settled::Parked {
                    return Err("the queue waits on a held answer past the end of the step".into());
                }
            }
            StatsRunInputStepsItem::Restore(held) => {
                let skip = held.skip.clone().unwrap_or_default();
                queue
                    .restore(|entry| entry.is_none_or(|name| !skip.iter().any(|each| each == name)))
                    .await;
            }
            StatsRunInputStepsItem::Dispose(_) => queue.dispose(),
        }
        if settle(&queue, &script).await == Settled::Parked {
            return Err("the queue waits on a held answer past the end of the step".into());
        }
        let armed = queue.armed_at();
        let on: Vec<i64> = locked(&live)
            .iter()
            .filter(|each| each.on)
            .map(|each| each.at)
            .collect();
        let agrees = match armed {
            None => on.is_empty(),
            Some(at) => on == [at],
        };
        if !agrees {
            return Err("the queue's timer and the wheel disagree".to_owned());
        }
        steps.push(StatsRunResultStepsItem {
            now_ms: net.wheel.now_ms(),
            reads: locked(&reads)[reads_before..].to_vec(),
            fills: sink.fills()[fills_before..]
                .iter()
                .map(|(id, stats)| StatsRunResultStepsItemFillsItem {
                    id: id.clone(),
                    stats: stats.clone(),
                })
                .collect(),
            waiting: queue
                .queued()
                .into_iter()
                .map(|each| StatsRunResultStepsItemWaitingItem {
                    id: each.id,
                    generation_id: each.generation_id,
                    entry: each.entry,
                    at: each.at,
                    tries: each.tries,
                    due: each.due,
                })
                .collect(),
            armed_at: armed,
        });
    }
    let result = StatsRunResult {
        steps,
        requests: net.asked(),
        read: net
            .played
            .read()
            .into_iter()
            .map(|read| read as i64)
            .collect(),
        filled_asks: sink
            .filled_asks()
            .into_iter()
            .map(|(id, at)| StatsRunResultFilledAsksItem { id, at })
            .collect(),
        unfilled_asks: sink.unfilled_asks(),
        timers: net.wheel.waiting(),
    };
    queue.dispose();
    Ok(result)
}
