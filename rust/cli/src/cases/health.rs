//! Provider health cases.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, MutexGuard};

use nyaaai_agentic::health::health::{Billing, Gate, HealthDeps, ProviderHealth, RouteRef};
use nyaaai_agentic::provider::kind::KINDS;
use nyaaai_contract::cancel::{CancelToken, Unsubscribe};
use nyaaai_contract::entry_keys::{EntryKeys, KeyAnswer};
use nyaaai_contract::future::BoxFuture;
use nyaaai_contract::id::CountedIds;
use nyaaai_contract::shapes::health::{Admission, HealthChange, Ticket};
use nyaaai_contract::shapes::step::{Failure, FailureCategory, FailureStatus, NormalizedUsage};
use nyaaai_contract::time::Clock;
use nyaaai_testkit::cases::Outcome;
use nyaaai_testkit::cases::block_on;
use nyaaai_testkit::scripted_ports::{ScriptedEntryKeys, ScriptedProviderList};

use super::local::net_of;
use super::shapes::*;

fn locked<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn answer_of(held: &HealthKeyAnswer) -> KeyAnswer {
    match held {
        HealthKeyAnswer::Key(held) => KeyAnswer::Key(held.key.clone()),
        HealthKeyAnswer::None(_) => KeyAnswer::None,
        HealthKeyAnswer::Refused(_) => KeyAnswer::Refused,
        HealthKeyAnswer::Unready(_) => KeyAnswer::Unready,
    }
}

fn shown_of(held: &KeyAnswer) -> HealthKeyAnswer {
    match held {
        KeyAnswer::Key(key) => HealthKeyAnswer::Key(HealthKeyAnswerKey { key: key.clone() }),
        KeyAnswer::None => HealthKeyAnswer::None(HealthKeyAnswerNone {}),
        KeyAnswer::Refused => HealthKeyAnswer::Refused(HealthKeyAnswerRefused {}),
        KeyAnswer::Unready => HealthKeyAnswer::Unready(HealthKeyAnswerUnready {}),
    }
}

/// Remembers an entry the script named, in the order first named.
fn name(named: &mut Vec<String>, id: &str) {
    if !named.iter().any(|held| held == id) {
        named.push(id.to_owned());
    }
}

fn route_of(route: &HealthRoute) -> RouteRef<'_> {
    RouteRef {
        entry: &route.entry,
        key: &route.key,
    }
}

fn deps_of(
    net: &super::local::Net,
    list: Arc<ScriptedProviderList>,
    keys: Arc<dyn EntryKeys>,
    pause_ms: Option<i64>,
) -> HealthDeps {
    let kind_deps = net.deps.clone();
    HealthDeps {
        scheduler: net.scheduler.clone(),
        list,
        keys,
        kinds: KINDS.clone(),
        kind_deps: Arc::new(move |_| kind_deps.clone()),
        pause_ms,
        max_pauses: None,
    }
}

pub fn run(input: HealthRunInput) -> Outcome<HealthRunResult> {
    block_on(play(input))?
}

async fn play(input: HealthRunInput) -> Outcome<HealthRunResult> {
    let net = net_of(input.start_ms, &input.exchanges);
    let list = Arc::new(ScriptedProviderList::new(input.entries.clone()));
    let keys = Arc::new(ScriptedEntryKeys::new());
    for (id, answer) in input.keys.iter() {
        keys.set(id, answer_of(answer));
    }
    let health = ProviderHealth::new(
        deps_of(
            &net,
            Arc::clone(&list),
            keys.clone(),
            input.pause_ms.map(|held| held as i64),
        ),
        &CountedIds::new("gate-"),
    );
    let changes: Arc<Mutex<Vec<HealthChange>>> = Arc::default();
    {
        let changes = Arc::clone(&changes);
        drop(health.on_change(Box::new(move |change| {
            locked(&changes).push(change.clone());
        })));
    }
    let mut named: Vec<String> = Vec::new();
    for entry in &input.entries {
        name(&mut named, &entry.id);
    }
    let mut run: HashSet<String> = HashSet::new();
    let mut tickets: HashMap<String, Ticket> = HashMap::new();
    let mut steps = Vec::new();
    for step in &input.steps {
        let mut answer =
            HealthRunResultStepsItemAnswer::None(HealthRunResultStepsItemAnswerNone {});
        match step {
            HealthRunInputStepsItem::List(held) => {
                for entry in &held.entries {
                    name(&mut named, &entry.id);
                }
                list.set(held.entries.clone());
            }
            HealthRunInputStepsItem::Key(held) => {
                name(&mut named, &held.id);
                keys.set(&held.id, answer_of(&held.answer));
            }
            HealthRunInputStepsItem::Check(held) => {
                name(&mut named, &held.id);
                health.check(&held.id);
            }
            HealthRunInputStepsItem::CheckUnread(_) => health.check_unread(),
            HealthRunInputStepsItem::Keys(held) => {
                name(&mut named, &held.id);
                let answered = health.entry_keys().get(&held.id).await;
                answer = HealthRunResultStepsItemAnswer::Keys(HealthRunResultStepsItemAnswerKeys {
                    answer: shown_of(&answered),
                });
            }
            HealthRunInputStepsItem::Served(held) => {
                name(&mut named, &held.route.entry.id);
                let usage = NormalizedUsage {
                    input: 1.0,
                    output: 1.0,
                    cached: None,
                    cache_write: None,
                    reasoning: None,
                    cost: None,
                    upstream_cost: None,
                    byok: None,
                    billing_source: held.billing_source.clone(),
                    list_cost: None,
                    allowance: None,
                };
                let keep = (held.keep == Some(true)).then_some(&mut run);
                health.served(route_of(&held.route), Some(&usage), keep);
            }
            HealthRunInputStepsItem::Failed(held) => {
                name(&mut named, &held.route.entry.id);
                health.failed(route_of(&held.route), &held.failure, held.onward);
            }
            HealthRunInputStepsItem::Billed(held) => {
                name(&mut named, &held.id);
                health.billed(
                    &held.id,
                    &Billing {
                        source: held.reply.source.clone(),
                        allowance: held.reply.allowance,
                        cost: held.reply.cost,
                    },
                );
            }
            HealthRunInputStepsItem::Deferred(held) => {
                name(&mut named, &held.route.entry.id);
                let keep = (held.keep == Some(true)).then_some(&run);
                answer = HealthRunResultStepsItemAnswer::Skip(HealthRunResultStepsItemAnswerSkip {
                    skip: health.deferred(route_of(&held.route), keep),
                });
            }
            HealthRunInputStepsItem::Blocked(held) => {
                name(&mut named, &held.route.entry.id);
                let keep = (held.keep == Some(true)).then_some(&run);
                answer = HealthRunResultStepsItemAnswer::Blocked(
                    HealthRunResultStepsItemAnswerBlocked {
                        blocked: health.blocked(route_of(&held.route), keep),
                    },
                );
            }
            HealthRunInputStepsItem::TryAdmit(held) => {
                name(&mut named, &held.route.entry.id);
                let ticket = health.try_admit(route_of(&held.route));
                if let Some(ticket) = &ticket {
                    tickets.insert(held.request.clone(), ticket.clone());
                }
                answer =
                    HealthRunResultStepsItemAnswer::Ticket(HealthRunResultStepsItemAnswerTicket {
                        ticket,
                    });
            }
            HealthRunInputStepsItem::Admit(held) => {
                name(&mut named, &held.route.entry.id);
                let route = route_of(&held.route);
                if let Some(gate) = health.gate_state(route)
                    && (gate.phase == nyaaai_agentic::health::gate::Phase::Paused
                        || (gate.phase == nyaaai_agentic::health::gate::Phase::Probing
                            && gate.probe.is_some()))
                {
                    return Err(format!("The admission of {} would wait.", held.request));
                }
                let admission = health.admit(route, &CancelToken::new(), None, false).await;
                if let Admission::Go(go) = &admission {
                    tickets.insert(held.request.clone(), go.ticket.clone());
                }
                answer = HealthRunResultStepsItemAnswer::Admission(
                    HealthRunResultStepsItemAnswerAdmission { admission },
                );
            }
            HealthRunInputStepsItem::Settle(held) => {
                let ticket = tickets
                    .get(&held.request)
                    .ok_or_else(|| format!("No ticket for {}.", held.request))?;
                answer = HealthRunResultStepsItemAnswer::Settled(
                    HealthRunResultStepsItemAnswerSettled {
                        settled: health.settle(
                            ticket,
                            held.outcome,
                            held.retry_after_ms.map(|after| after as i64),
                        ),
                    },
                );
            }
            HealthRunInputStepsItem::Advance(held) => {
                net.wheel.advance(held.ms as i64);
            }
            HealthRunInputStepsItem::Dispose(_) => health.dispose(),
        }
        health.idle().await;
        let mut sorted: Vec<String> = run.iter().cloned().collect();
        sorted.sort_by(|left, right| nyaaai_contract::order::compare_text(left, right));
        steps.push(HealthRunResultStepsItem {
            now_ms: net.wheel.now_ms(),
            answer,
            changes: std::mem::take(&mut *locked(&changes)),
            entries: named.iter().map(|id| health.state(id)).collect(),
            run: sorted,
            requests: net.played.requests().len() as i64,
        });
    }
    health.idle().await;
    health.dispose();
    Ok(HealthRunResult {
        steps,
        requests: net
            .played
            .requests()
            .iter()
            .map(|sent| sent.url.clone())
            .collect(),
        waiting: net.wheel.waiting(),
    })
}

type KeyListener = Arc<dyn Fn(&str) + Send + Sync>;

/// A store whose answer the row sets, and whose changes the row tells.
struct Store {
    current: Mutex<KeyAnswer>,
    listeners: Mutex<Vec<(u64, KeyListener)>>,
    next: Mutex<u64>,
}

impl Store {
    fn tell(&self, id: &str) {
        let held: Vec<KeyListener> = locked(&self.listeners)
            .iter()
            .map(|(_, listener)| Arc::clone(listener))
            .collect();
        for listener in held {
            listener(id);
        }
    }
}

struct Keys(Arc<Store>);

impl EntryKeys for Keys {
    fn get(&self, _id: &str) -> BoxFuture<'_, KeyAnswer> {
        let answer = locked(&self.0.current).clone();
        Box::pin(async move { answer })
    }

    fn on_change(&self, listener: Box<dyn Fn(&str) + Send + Sync>) -> Unsubscribe {
        let id = {
            let mut next = locked(&self.0.next);
            *next += 1;
            *next
        };
        locked(&self.0.listeners).push((id, Arc::from(listener)));
        let store = Arc::clone(&self.0);
        Box::new(move || locked(&store.listeners).retain(|(held, _)| *held != id))
    }
}

pub fn keys(input: HealthKeysInput) -> Outcome<HealthKeysResult> {
    block_on(ask(input))?
}

async fn ask(input: HealthKeysInput) -> Outcome<HealthKeysResult> {
    let mut rows = Vec::new();
    for row in &input.rows {
        let net = net_of(0, &row.exchanges);
        let store = Arc::new(Store {
            current: Mutex::new(answer_of(&row.stored)),
            listeners: Mutex::new(Vec::new()),
            next: Mutex::new(0),
        });
        let list = Arc::new(ScriptedProviderList::new(if row.listed == Some(false) {
            Vec::new()
        } else {
            vec![row.entry.clone()]
        }));
        let health = ProviderHealth::new(
            deps_of(&net, list, Arc::new(Keys(Arc::clone(&store))), None),
            &CountedIds::new("gate-"),
        );
        let route = RouteRef {
            entry: &row.entry,
            key: "lab/model-a",
        };
        let id = row.entry.id.as_str();
        let failure = |category: FailureCategory, status: FailureStatus, message: &str| Failure {
            category,
            status,
            retry_after_ms: None,
            resets_at: None,
            message: message.to_owned(),
            reasons: None,
            billed: None,
        };
        match row.prepare {
            HealthKeysInputRowsItemPrepare::None => {}
            HealthKeysInputRowsItemPrepare::Checking => health.check(id),
            HealthKeysInputRowsItemPrepare::Ok => {
                health.check(id);
                health.idle().await;
            }
            HealthKeysInputRowsItemPrepare::Rejected => health.failed(
                route,
                &failure(
                    FailureCategory::KeyRejected,
                    FailureStatus::Number(401.0),
                    "no",
                ),
                false,
            ),
            HealthKeysInputRowsItemPrepare::Unreachable => health.failed(
                route,
                &failure(
                    FailureCategory::Network,
                    FailureStatus::Text("network".to_owned()),
                    "down",
                ),
                false,
            ),
        }
        if let Some(then) = &row.stored_then {
            *locked(&store.current) = answer_of(then);
        }
        let status = health.status(id);
        // The asks and the tells run before any check gets a turn.
        let keys = health.entry_keys();
        let asks: Vec<BoxFuture<'_, KeyAnswer>> =
            (0..row.asks.unwrap_or(1)).map(|_| keys.get(id)).collect();
        let heard: Arc<Mutex<Vec<String>>> = Arc::default();
        let stop = {
            let heard = Arc::clone(&heard);
            keys.on_change(Box::new(move |told| locked(&heard).push(told.to_owned())))
        };
        for told in row.tell.iter().flatten() {
            store.tell(told);
        }
        let held = locked(&heard).len();
        stop();
        for told in row.tell_after.iter().flatten() {
            store.tell(told);
        }
        let mut answers = Vec::new();
        for ask in asks {
            answers.push(shown_of(&ask.await));
        }
        health.idle().await;
        let status_after = health.status(id);
        health.dispose();
        let heard = locked(&heard).clone();
        rows.push(HealthKeysResultRowsItem {
            status,
            answers,
            status_after,
            told: heard[..held].to_vec(),
            told_after: heard[held..].to_vec(),
            requests: net.played.requests().len() as i64,
        });
    }
    Ok(HealthKeysResult { rows })
}
