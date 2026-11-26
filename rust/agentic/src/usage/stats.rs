//! Delayed generation stats: reading them back, with retries and spacing.

use std::future::Future;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::pin::Pin;
use std::sync::{Arc, Mutex, MutexGuard};
use std::task::{Context, Poll, Waker};

use nyaaai_contract::cancel::CancelToken;
use nyaaai_contract::fetch::{BudgetName, Budgets, Span, span};
use nyaaai_contract::future::BoxFuture;
use nyaaai_contract::json::{Object, Value};
use nyaaai_contract::shapes::step::NormalizedUsage;
use nyaaai_contract::shapes::usage::GenerationStats;
use nyaaai_contract::time::{Scheduled, Scheduler, deadline};
use nyaaai_contract::usage_sink::UsageSink;
use tokio::runtime::Handle;
use tokio::sync::watch;

use super::cost::reported_cost;
use crate::kinds::readers::{
    boolean_field, field_of, fields_of, number_field, object_field, text_field,
};

/// Reads a generation through its serving entry's transport, never another entry's key; should end on its span; a failure or panic is `Err`.
pub type ReadStats = Arc<
    dyn Fn(Option<String>, String, Arc<Span>) -> BoxFuture<'static, Result<Value, String>>
        + Send
        + Sync,
>;

/// What a queue stands on.
pub struct StatsQueueDeps {
    /// Where reads run, wherever a timer fires.
    pub runtime: Handle,
    pub scheduler: Arc<dyn Scheduler>,
    /// Where stats are written, and where unfilled records are listed.
    pub usage: Arc<dyn UsageSink>,
    pub read: ReadStats,
    /// `stats` bounds reads and asks; default the presets.
    pub budgets: Option<Budgets>,
}

/// A record whose stats are still to be read.
#[derive(Debug, Clone, PartialEq)]
pub struct WaitingRead {
    pub id: String,
    pub generation_id: String,
    /// The serving entry.
    pub entry: Option<String>,
    /// Request timestamp.
    pub at: i64,
    /// Reads that missed.
    pub tries: i64,
    /// When the next read may start.
    pub due: i64,
}

#[derive(Clone, Copy)]
enum FieldKind {
    Text,
    Number,
    Flag,
}

/// Request-relative retry delays.
const DELAYS_MS: [i64; 4] = [7 * 60_000, 15 * 60_000, 30 * 60_000, 60 * 60_000];

/// Spaces endpoint reads.
const SPACING_MS: i64 = 250;

/// Queue limit.
const MAX_WAITING: usize = 500;

/// How far back `restore` looks for records without stats.
pub const UNFILLED_MS: i64 = 2 * 60 * 60_000;

const GENERATION_PREFIX: &str = "gen-";

/// The part after the prefix is 1 to 124 characters.
const GENERATION_REST_MAX: usize = 124;

/// Each field may be missing or null; one of another type makes it no stats.
const FIELDS: [(&str, FieldKind); 11] = [
    ("provider_name", FieldKind::Text),
    ("latency", FieldKind::Number),
    ("generation_time", FieldKind::Number),
    ("total_cost", FieldKind::Number),
    ("is_byok", FieldKind::Flag),
    ("upstream_inference_cost", FieldKind::Number),
    ("tokens_prompt", FieldKind::Number),
    ("tokens_completion", FieldKind::Number),
    ("native_tokens_prompt", FieldKind::Number),
    ("native_tokens_completion", FieldKind::Number),
    ("finish_reason", FieldKind::Text),
];

fn fits(fields: &Object, key: &str, kind: FieldKind) -> bool {
    match field_of(fields, key) {
        None | Some(Value::Null) => true,
        Some(Value::String(_)) => matches!(kind, FieldKind::Text),
        Some(Value::Number(held)) => matches!(kind, FieldKind::Number) && held.is_finite(),
        Some(Value::Bool(_)) => matches!(kind, FieldKind::Flag),
        Some(_) => false,
    }
}

fn is_generation_id(text: &str) -> bool {
    let Some(rest) = text.strip_prefix(GENERATION_PREFIX) else {
        return false;
    };
    let length = rest.encode_utf16().count();
    (1..=GENERATION_REST_MAX).contains(&length)
        && rest
            .chars()
            .all(|char| char.is_ascii_alphanumeric() || char == '-')
}

/// None for a body that is not a generation's stats.
pub fn stats_of(body: &Value) -> Option<GenerationStats> {
    let data = object_field(fields_of(body)?, "data")?;
    if !FIELDS.iter().all(|(key, kind)| fits(data, key, *kind)) {
        return None;
    }
    let account = number_field(data, "total_cost");
    let byok = boolean_field(data, "is_byok");
    let upstream = number_field(data, "upstream_inference_cost");
    let cost = reported_cost(&NormalizedUsage {
        input: 0.0,
        output: 0.0,
        cached: None,
        cache_write: None,
        reasoning: None,
        cost: account,
        upstream_cost: upstream,
        byok,
        billing_source: None,
        list_cost: None,
        allowance: None,
    });
    Some(GenerationStats {
        provider: text_field(data, "provider_name").map(str::to_owned),
        latency_ms: number_field(data, "latency"),
        generation_ms: number_field(data, "generation_time"),
        cost,
        account_cost: account,
        upstream_cost: upstream,
        byok,
        in_: number_field(data, "native_tokens_prompt")
            .or_else(|| number_field(data, "tokens_prompt")),
        out: number_field(data, "native_tokens_completion")
            .or_else(|| number_field(data, "tokens_completion")),
        finish: text_field(data, "finish_reason").map(str::to_owned),
    })
}

/// A future whose panic ends it with none.
struct Caught<F>(Pin<Box<F>>);

impl<F: Future> Future for Caught<F> {
    type Output = Option<F::Output>;

    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        match catch_unwind(AssertUnwindSafe(|| self.0.as_mut().poll(context))) {
            Ok(Poll::Ready(held)) => Poll::Ready(Some(held)),
            Ok(Poll::Pending) => Poll::Pending,
            Err(_) => Poll::Ready(None),
        }
    }
}

/// A queue timer; its callback acts only while the queue holds it.
struct Timer {
    id: u64,
    at: i64,
    /// Set once `schedule` has returned.
    cancel: Option<Scheduled>,
}

struct State {
    waiting: Vec<WaitingRead>,
    timer: Option<Timer>,
    timers: u64,
    reading: bool,
    /// The id of the record being read.
    current: Option<String>,
    disposed: bool,
}

struct Shared {
    d: StatsQueueDeps,
    runtime: Handle,
    /// Every deadline follows it; a dispose cancels it.
    stop: CancelToken,
    state: Mutex<State>,
    /// Reads running.
    running: watch::Sender<usize>,
}

impl Shared {
    fn locked(&self) -> MutexGuard<'_, State> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

/// A read running; its end, by a panic too, is told.
struct Running(Arc<Shared>);

impl Drop for Running {
    fn drop(&mut self) {
        self.0.running.send_modify(|held| *held -= 1);
    }
}

/// Gives up the current turn once.
struct Turn(bool);

impl Future for Turn {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<()> {
        if self.0 {
            return Poll::Ready(());
        }
        self.0 = true;
        context.waker().wake_by_ref();
        Poll::Pending
    }
}

/// One read at a time at 7, 15, 30, 60 min, 250 ms apart; a record waits once, at its earlier due.
#[derive(Clone)]
pub struct StatsQueue {
    shared: Arc<Shared>,
}

impl StatsQueue {
    pub fn new(d: StatsQueueDeps) -> Self {
        let runtime = d.runtime.clone();
        Self {
            shared: Arc::new(Shared {
                d,
                runtime,
                stop: CancelToken::new(),
                state: Mutex::new(State {
                    waiting: Vec::new(),
                    timer: None,
                    timers: 0,
                    reading: false,
                    current: None,
                    disposed: false,
                }),
                running: watch::channel(0).0,
            }),
        }
    }

    /// Schedules stats reads; a past time starts at once; a record waits once, at its earlier due.
    pub fn add(&self, id: &str, generation_id: &str, at: Option<i64>, entry: Option<&str>) {
        let shared = &self.shared;
        if !is_generation_id(generation_id) {
            return;
        }
        let now = shared.d.scheduler.now_ms();
        let at = at.unwrap_or(now);
        {
            let mut state = shared.locked();
            if state.disposed || state.current.as_deref() == Some(id) {
                return;
            }
            let passed = DELAYS_MS.iter().filter(|delay| at + **delay <= now).count() as i64;
            let (tries, due) = if passed == 0 {
                (0, at + DELAYS_MS[0])
            } else {
                (passed - 1, now)
            };
            let fresh = WaitingRead {
                id: id.to_owned(),
                generation_id: generation_id.to_owned(),
                entry: entry.map(str::to_owned),
                at,
                tries,
                due,
            };
            match state.waiting.iter().position(|each| each.id == id) {
                Some(held) => {
                    if fresh.due < state.waiting[held].due {
                        state.waiting[held] = fresh;
                    }
                }
                None => {
                    if state.waiting.len() >= MAX_WAITING {
                        state.waiting.remove(0);
                    }
                    state.waiting.push(fresh);
                }
            }
        }
        arm(shared, 0);
    }

    /// Adds recent records lacking stats; `keep` drops entries that are gone.
    pub async fn restore(&self, keep: impl Fn(Option<&str>) -> bool) {
        let usage = &self.shared.d.usage;
        let since = self.shared.d.scheduler.now_ms() - UNFILLED_MS;
        let asked = catch_unwind(AssertUnwindSafe(|| usage.unfilled(since)));
        let Ok(asked) = asked else {
            return;
        };
        let Some(records) = Caught(Box::pin(asked)).await else {
            return;
        };
        for record in records {
            if keep(record.entry.as_deref()) {
                self.add(
                    &record.id,
                    &record.generation_id,
                    Some(record.at as i64),
                    record.entry.as_deref(),
                );
            }
        }
    }

    /// Drops waits and ends running reads; their results are unused.
    pub fn dispose(&self) {
        let timer = {
            let mut state = self.shared.locked();
            state.disposed = true;
            state.waiting.clear();
            state.timer.take()
        };
        self.shared.stop.cancel();
        if let Some(Timer {
            cancel: Some(timer),
            ..
        }) = timer
        {
            timer.cancel();
        }
    }

    /// Answers when no read is running, having waited for the one that was.
    pub async fn idle(&self) {
        let mut running = self.shared.running.subscribe();
        let _ = running.wait_for(|held| *held == 0).await;
    }

    /// Records that wait, in queue order.
    pub fn queued(&self) -> Vec<WaitingRead> {
        self.shared.locked().waiting.clone()
    }

    /// The instant the next read is set for.
    pub fn armed_at(&self) -> Option<i64> {
        self.shared.locked().timer.as_ref().map(|held| held.at)
    }
}

/// Enforces read spacing.
fn arm(shared: &Arc<Shared>, spacing: i64) {
    let now = shared.d.scheduler.now_ms();
    let (id, at, old) = {
        let mut state = shared.locked();
        if state.disposed || state.reading || state.waiting.is_empty() {
            return;
        }
        let due = state
            .waiting
            .iter()
            .map(|each| each.due)
            .min()
            .unwrap_or(now);
        let at = due.max(now + spacing);
        if state.timer.as_ref().is_some_and(|held| held.at <= at) {
            return;
        }
        let old = state.timer.take();
        state.timers += 1;
        let id = state.timers;
        // The record exists before the timer can fire.
        state.timer = Some(Timer {
            id,
            at,
            cancel: None,
        });
        (id, at, old)
    };
    if let Some(Timer {
        cancel: Some(old), ..
    }) = old
    {
        old.cancel();
    }
    let held = Arc::clone(shared);
    let scheduled = shared.d.scheduler.schedule(
        (at - now).max(0),
        Box::new(move || {
            let mine = {
                let mut state = held.locked();
                let mine = state.timer.as_ref().is_some_and(|timer| timer.id == id);
                if mine {
                    state.timer = None;
                }
                mine
            };
            if mine {
                start(&held);
            }
        }),
    );
    let mut state = shared.locked();
    match state.timer.as_mut() {
        Some(timer) if timer.id == id => timer.cancel = Some(scheduled),
        _ => {
            drop(state);
            scheduled.cancel();
        }
    }
}

/// The timer's part: take the record, set the mark, begin the ask.
fn start(shared: &Arc<Shared>) {
    let _entered = shared.runtime.enter();
    let Some(each) = begin(shared) else {
        return;
    };
    shared.running.send_modify(|held| *held += 1);
    let running = Running(Arc::clone(shared));
    let held = Arc::clone(shared);
    let mut task: BoxFuture<'static, ()> = Box::pin(async move {
        let _running = running;
        finish(held, each).await;
    });
    if task
        .as_mut()
        .poll(&mut Context::from_waker(Waker::noop()))
        .is_pending()
    {
        shared.runtime.spawn(task);
    }
}

/// Takes the first record that is due; none arms again.
fn begin(shared: &Arc<Shared>) -> Option<WaitingRead> {
    let now = shared.d.scheduler.now_ms();
    let each = {
        let mut state = shared.locked();
        let index = state.waiting.iter().position(|each| each.due <= now);
        let each = index.map(|index| state.waiting.remove(index));
        if let Some(each) = &each
            && !state.disposed
        {
            state.reading = true;
            state.current = Some(each.id.clone());
        }
        each.filter(|_| state.reading)
    };
    if each.is_none() {
        arm(shared, 0);
    }
    each
}

/// A port's answer, or none at the window's end, dispose or failure.
async fn within<'a, T>(limit: &CancelToken, call: impl FnOnce() -> BoxFuture<'a, T>) -> Option<T> {
    match catch_unwind(AssertUnwindSafe(call)) {
        Ok(asked) => {
            // The answer comes after a turn, as a port's does.
            Turn(false).await;
            limit.run_until(Caught(Box::pin(asked))).await.flatten()
        }
        Err(_) => None,
    }
}

/// Reads or reschedules.
async fn finish(shared: Arc<Shared>, mut each: WaitingRead) {
    let scheduler = &shared.d.scheduler;
    let budget = shared
        .d
        .budgets
        .clone()
        .unwrap_or_default()
        .get(BudgetName::Stats)
        .clone();
    // The ask is a host port's wait, not an exchange.
    let asked = deadline(&**scheduler, budget.total_ms as i64, &shared.stop);
    // A failed ask reads the record.
    let filled =
        within(asked.token(), || shared.d.usage.filled(&each.id, each.at)).await == Some(true);
    asked.clear();
    let disposed = shared.locked().disposed;
    let body = if filled || disposed {
        None
    } else {
        let reading = Arc::new(span(scheduler, &budget, Some(&shared.stop)));
        let path = format!("/generation?id={}", each.generation_id);
        let read = within(reading.token(), || {
            (shared.d.read)(each.entry.clone(), path, Arc::clone(&reading))
        })
        .await
        .and_then(Result::ok);
        reading.clear();
        read
    };
    let mut spacing = SPACING_MS;
    let stop = {
        let mut state = shared.locked();
        if state.disposed {
            state.reading = false;
            state.current = None;
            true
        } else {
            false
        }
    };
    if stop {
        return;
    }
    if filled {
        spacing = 0;
    } else {
        match body.as_ref().and_then(stats_of) {
            Some(stats) => {
                // A fill that fails leaves the record without stats.
                let _ = catch_unwind(AssertUnwindSafe(|| shared.d.usage.fill(&each.id, &stats)));
            }
            None => {
                each.tries += 1;
                if let Some(delay) = usize::try_from(each.tries)
                    .ok()
                    .and_then(|at| DELAYS_MS.get(at))
                {
                    let due = each.at + delay;
                    let mut state = shared.locked();
                    if state.waiting.len() >= MAX_WAITING {
                        state.waiting.remove(0);
                    }
                    state.waiting.push(WaitingRead { due, ..each });
                }
            }
        }
    }
    {
        let mut state = shared.locked();
        state.reading = false;
        state.current = None;
    }
    arm(&shared, spacing);
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use nyaaai_contract::time::{Clock, Slept};
    use nyaaai_testkit::scheduler::{FakeScheduler, TimerWheel};
    use nyaaai_testkit::scripted_ports::ScriptedUsageSink;

    use super::*;

    /// A scheduler that runs a zero delay before `schedule` returns.
    struct Early(FakeScheduler);

    impl Clock for Early {
        fn now_ms(&self) -> i64 {
            self.0.now_ms()
        }

        fn utc_offset_minutes(&self, at_ms: i64) -> i64 {
            self.0.utc_offset_minutes(at_ms)
        }
    }

    impl Scheduler for Early {
        fn sleep(&self, ms: i64, cancel: &CancelToken) -> BoxFuture<'static, Slept> {
            self.0.sleep(ms, cancel)
        }

        fn schedule(&self, ms: i64, run: Box<dyn FnOnce() + Send>) -> Scheduled {
            if ms == 0 {
                run();
                return Scheduled::new(|| {});
            }
            self.0.schedule(ms, run)
        }
    }

    fn deps(scheduler: Arc<dyn Scheduler>, reads: &Arc<AtomicUsize>) -> StatsQueueDeps {
        let reads = Arc::clone(reads);
        StatsQueueDeps {
            runtime: Handle::current(),
            scheduler,
            usage: Arc::new(ScriptedUsageSink::new()),
            read: Arc::new(move |_, _, _| {
                reads.fetch_add(1, Ordering::SeqCst);
                Box::pin(async { Err("no stats".to_owned()) })
            }),
            budgets: None,
        }
    }

    fn runtime() -> tokio::runtime::Runtime {
        tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
    }

    #[test]
    fn a_timer_that_fires_before_schedule_returns_leaves_the_queue_working() {
        runtime().block_on(async {
            let wheel = Arc::new(TimerWheel::new(10_000_000));
            let reads = Arc::new(AtomicUsize::new(0));
            let early = Arc::new(Early(FakeScheduler::new(Arc::clone(&wheel))));
            let queue = StatsQueue::new(deps(early, &reads));
            queue.add("a", "gen-a", Some(0), None);
            queue.idle().await;
            assert_eq!(reads.load(Ordering::SeqCst), 1);
            queue.add("b", "gen-b", Some(0), None);
            queue.idle().await;
            assert_eq!(reads.load(Ordering::SeqCst), 2);
            queue.dispose();
        });
    }

    #[test]
    fn a_timer_fired_from_a_plain_thread_starts_its_read() {
        let runtime = runtime();
        runtime.block_on(async {
            let wheel = Arc::new(TimerWheel::new(0));
            let reads = Arc::new(AtomicUsize::new(0));
            let scheduler = Arc::new(FakeScheduler::new(Arc::clone(&wheel)));
            let queue = StatsQueue::new(deps(scheduler, &reads));
            queue.add("a", "gen-a", Some(0), None);
            assert_eq!(queue.armed_at(), Some(420_000));
            let fired = Arc::clone(&wheel);
            std::thread::spawn(move || {
                fired.advance(420_000);
            })
            .join()
            .unwrap();
            queue.idle().await;
            assert_eq!(reads.load(Ordering::SeqCst), 1);
            queue.dispose();
        });
    }
}
