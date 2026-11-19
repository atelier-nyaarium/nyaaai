//! Provider health cases.

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use nyaaai_agentic::codec::Codec;
use nyaaai_agentic::health::gate::Phase;
use nyaaai_agentic::health::health::{Billing, Gate, HealthDeps, ProviderHealth, RouteRef};
use nyaaai_agentic::provider::incarnation::{EntryRegistry, Incarnation};
use nyaaai_agentic::provider::kind::{
    CatalogDeps, EntryCatalog, KINDS, KindDeps, KindError, KindField, ProviderKind,
};
use nyaaai_agentic::transport::{KeySource, Transport};
use nyaaai_contract::cancel::{CancelToken, Unsubscribe};
use nyaaai_contract::entry_keys::{EntryKeys, KeyAnswer};
use nyaaai_contract::future::BoxFuture;
use nyaaai_contract::id::CountedIds;
use nyaaai_contract::shapes::health::{Admission, HealthChange, Ticket};
use nyaaai_contract::shapes::model::ModelView;
use nyaaai_contract::shapes::provider::{
    EntryFields, KeyState, KeyStateNone, ProviderEntry, ProviderKindName,
};
use nyaaai_contract::shapes::step::{Failure, FailureCategory, FailureStatus, NormalizedUsage};
use nyaaai_contract::time::Clock;
use nyaaai_testkit::cases::Outcome;
use nyaaai_testkit::cases::block_on;
use nyaaai_testkit::scripted_ports::{ScriptedEntryKeys, ScriptedProviderList};
use tokio::sync::{oneshot, watch};

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
fn name(named: &Mutex<Vec<String>>, id: &str) {
    let mut named = locked(named);
    if !named.iter().any(|held| held == id) {
        named.push(id.to_owned());
    }
}

/// A route held by the script: the row, the key and the incarnation it stands over.
#[derive(Clone)]
struct Owned {
    entry: ProviderEntry,
    key: String,
    incarnation: Arc<Incarnation>,
}

impl Owned {
    fn as_ref(&self) -> RouteRef<'_> {
        RouteRef {
            entry: &self.entry,
            key: &self.key,
            incarnation: &self.incarnation,
        }
    }
}

/// An incarnation whose entry has already left the list.
fn left_incarnation(entry: &ProviderEntry) -> Arc<Incarnation> {
    let list = Arc::new(ScriptedProviderList::new(vec![entry.clone()]));
    let scratch = EntryRegistry::new(list.clone(), &ScriptedEntryKeys::new());
    let incarnation = scratch.incarnation(&entry.id).expect("the entry is listed");
    list.set(Vec::new());
    scratch.dispose();
    incarnation
}

/// The route over the entry's incarnation as it stands now.
fn standing(entries: &EntryRegistry, route: &HealthRoute) -> Owned {
    Owned {
        entry: route.entry.clone(),
        key: route.key.clone(),
        incarnation: entries
            .incarnation(&route.entry.id)
            .unwrap_or_else(|| left_incarnation(&route.entry)),
    }
}

fn routed(
    entries: &EntryRegistry,
    taken: &HashMap<String, Owned>,
    route: &HealthRoute,
) -> Outcome<Owned> {
    match &route.taken {
        None => Ok(standing(entries, route)),
        Some(name) => taken
            .get(name)
            .cloned()
            .ok_or_else(|| format!("No route taken as {name}.")),
    }
}

fn deps_of(
    net: &super::local::Net,
    entries: Arc<EntryRegistry>,
    keys: Arc<dyn EntryKeys>,
    pause_ms: Option<i64>,
) -> HealthDeps {
    let kind_deps = net.deps.clone();
    HealthDeps {
        runtime: nyaaai_testkit::cases::runtime(),
        scheduler: net.scheduler.clone(),
        entries,
        keys,
        kinds: KINDS.clone(),
        kind_deps: Arc::new(move |_| kind_deps.clone()),
        pause_ms,
        max_pauses: None,
    }
}

/// What a released hold lets go.
#[derive(Default)]
struct Go {
    state: Option<KeyState>,
    answer: Option<KeyAnswer>,
}

/// A place a check parks until the script releases it.
struct Hold {
    at: HealthRunInputStepsItemHoldAt,
    seen: AtomicBool,
    parked: AtomicBool,
    reached: watch::Sender<bool>,
    release: Mutex<Option<oneshot::Sender<Go>>>,
    wait: Mutex<Option<oneshot::Receiver<Go>>>,
}

impl Hold {
    fn new(at: HealthRunInputStepsItemHoldAt) -> Arc<Self> {
        let (release, wait) = oneshot::channel();
        Arc::new(Self {
            at,
            seen: AtomicBool::new(false),
            parked: AtomicBool::new(false),
            reached: watch::channel(false).0,
            release: Mutex::new(Some(release)),
            wait: Mutex::new(Some(wait)),
        })
    }

    fn arrive(&self) {
        self.parked.store(true, Ordering::SeqCst);
        self.seen.store(true, Ordering::SeqCst);
        self.reached.send_replace(true);
    }

    fn let_go(&self, go: Go) {
        self.parked.store(false, Ordering::SeqCst);
        if let Some(release) = locked(&self.release).take() {
            let _ = release.send(go);
        }
    }

    async fn released(&self) -> Go {
        let wait = locked(&self.wait).take();
        match wait {
            Some(wait) => wait.await.unwrap_or_default(),
            None => Go::default(),
        }
    }

    async fn reached(&self) {
        let mut reached = self.reached.subscribe();
        let _ = reached.wait_for(|held| *held).await;
    }
}

/// The holds armed and not yet taken by a check, by place.
#[derive(Default)]
struct Armed {
    store: Mutex<VecDeque<Arc<Hold>>>,
    check: Mutex<VecDeque<Arc<Hold>>>,
}

/// The store's keys, whose next read can be held.
struct Held {
    raw: Arc<ScriptedEntryKeys>,
    armed: Arc<Armed>,
}

impl EntryKeys for Held {
    fn get(&self, id: &str) -> BoxFuture<'_, KeyAnswer> {
        let Some(hold) = locked(&self.armed.store).pop_front() else {
            return self.raw.get(id);
        };
        let id = id.to_owned();
        Box::pin(async move {
            hold.arrive();
            match hold.released().await.answer {
                Some(answer) => answer,
                None => self.raw.get(&id).await,
            }
        })
    }

    fn on_change(&self, listener: Box<dyn Fn(&str) + Send + Sync>) -> Unsubscribe {
        self.raw.on_change(listener)
    }
}

/// A real kind whose next check can be held.
struct Parking {
    real: Arc<dyn ProviderKind>,
    armed: Arc<Armed>,
}

impl ProviderKind for Parking {
    fn kind(&self) -> ProviderKindName {
        self.real.kind()
    }

    fn label(&self) -> &str {
        self.real.label()
    }

    fn field(&self) -> KindField {
        self.real.field()
    }

    fn optional_key(&self) -> bool {
        self.real.optional_key()
    }

    fn check<'a>(
        &'a self,
        entry: &'a EntryFields,
        secret: Option<&'a str>,
        deps: &'a KindDeps,
        cancel: &'a CancelToken,
    ) -> BoxFuture<'a, Result<KeyState, KindError>> {
        let Some(hold) = locked(&self.armed.check).pop_front() else {
            return self.real.check(entry, secret, deps, cancel);
        };
        Box::pin(async move {
            hold.arrive();
            match hold.released().await.state {
                Some(state) => Ok(state),
                None => self.real.check(entry, secret, deps, cancel).await,
            }
        })
    }

    fn transport(
        &self,
        entry: &EntryFields,
        key: Arc<dyn KeySource>,
        deps: &KindDeps,
    ) -> Result<Arc<Transport>, String> {
        self.real.transport(entry, key, deps)
    }

    fn catalog(
        &self,
        entry: &ProviderEntry,
        transport: Arc<Transport>,
        deps: CatalogDeps,
    ) -> Arc<dyn EntryCatalog> {
        self.real.catalog(entry, transport, deps)
    }

    fn codec(
        &self,
        view: Option<&ModelView>,
        entry: &EntryFields,
        deps: &KindDeps,
    ) -> Box<dyn Codec> {
        self.real.codec(view, entry, deps)
    }

    fn failure(
        &self,
        failure: &Failure,
        view: Option<&ModelView>,
    ) -> nyaaai_contract::shapes::provider::Disposition {
        self.real.failure(failure, view)
    }
}

pub fn run(input: HealthRunInput) -> Outcome<HealthRunResult> {
    block_on(play(input))?
}

/// Whether a step leaves the wait for `idle` out.
fn no_idle(step: &HealthRunInputStepsItem) -> bool {
    match step {
        HealthRunInputStepsItem::Release(held) => held.idle.is_some(),
        HealthRunInputStepsItem::List(held) => held.idle.is_some(),
        HealthRunInputStepsItem::Key(held) => held.idle.is_some(),
        HealthRunInputStepsItem::Check(held) => held.idle.is_some(),
        HealthRunInputStepsItem::Advance(held) => held.idle.is_some(),
        _ => false,
    }
}

async fn play(input: HealthRunInput) -> Outcome<HealthRunResult> {
    let net = net_of(input.start_ms, &input.exchanges);
    let list = Arc::new(ScriptedProviderList::new(input.entries.clone()));
    let raw = Arc::new(ScriptedEntryKeys::new());
    for (id, answer) in input.keys.iter() {
        raw.set(id, answer_of(answer));
    }
    let entries = Arc::new(EntryRegistry::new(list.clone(), &*raw));
    let armed: Arc<Armed> = Arc::default();
    let mut holds: Vec<Arc<Hold>> = Vec::new();
    let keys = Arc::new(Held {
        raw: Arc::clone(&raw),
        armed: Arc::clone(&armed),
    });
    let mut deps = deps_of(
        &net,
        Arc::clone(&entries),
        keys,
        input.pause_ms.map(|held| held as i64),
    );
    deps.kinds = KINDS
        .iter()
        .map(|real| {
            Arc::new(Parking {
                real: Arc::clone(real),
                armed: Arc::clone(&armed),
            }) as Arc<dyn ProviderKind>
        })
        .collect();
    let health = ProviderHealth::new(deps, &CountedIds::new("gate-"));
    let changes: Arc<Mutex<Vec<HealthChange>>> = Arc::default();
    let named: Arc<Mutex<Vec<String>>> = Arc::default();
    let starters: Arc<Mutex<HashSet<String>>> = Arc::default();
    {
        let (starters, named, health) = (Arc::clone(&starters), Arc::clone(&named), health.clone());
        drop(health.clone().on_change(Box::new(move |change| {
            if !locked(&starters).remove(&change.entry) {
                return;
            }
            name(&named, &change.entry);
            health.check(&change.entry);
        })));
    }
    {
        let changes = Arc::clone(&changes);
        drop(health.on_change(Box::new(move |change| {
            locked(&changes).push(change.clone());
        })));
    }
    for entry in &input.entries {
        name(&named, &entry.id);
    }
    let mut run: HashSet<String> = HashSet::new();
    let mut tickets: HashMap<String, Ticket> = HashMap::new();
    let mut taken: HashMap<String, Owned> = HashMap::new();
    let mut steps = Vec::new();
    for step in &input.steps {
        let mut answer =
            HealthRunResultStepsItemAnswer::None(HealthRunResultStepsItemAnswerNone {});
        let seen_before: Vec<bool> = holds
            .iter()
            .map(|held| held.seen.load(Ordering::SeqCst))
            .collect();
        match step {
            HealthRunInputStepsItem::Take(held) => {
                taken.insert(held.name.clone(), standing(&entries, &held.route));
            }
            HealthRunInputStepsItem::Hold(held) => {
                let hold = Hold::new(held.at);
                match held.at {
                    HealthRunInputStepsItemHoldAt::Store => {
                        locked(&armed.store).push_back(Arc::clone(&hold));
                    }
                    HealthRunInputStepsItemHoldAt::Check => {
                        locked(&armed.check).push_back(Arc::clone(&hold));
                    }
                }
                holds.push(hold);
            }
            HealthRunInputStepsItem::Release(held) => {
                let hold = held
                    .hold
                    .checked_sub(1)
                    .and_then(|at| holds.get(at as usize))
                    .ok_or_else(|| format!("No hold {}.", held.hold))?;
                hold.let_go(Go {
                    state: held.state.clone(),
                    answer: held.answer.as_ref().map(answer_of),
                });
            }
            HealthRunInputStepsItem::ListenAndCheck(held) => {
                locked(&starters).insert(held.id.clone());
            }
            HealthRunInputStepsItem::List(held) => {
                for entry in &held.entries {
                    name(&named, &entry.id);
                }
                list.set(held.entries.clone());
            }
            HealthRunInputStepsItem::Key(held) => {
                name(&named, &held.id);
                raw.set(&held.id, answer_of(&held.answer));
            }
            HealthRunInputStepsItem::Check(held) => {
                name(&named, &held.id);
                health.check(&held.id);
            }
            HealthRunInputStepsItem::CheckUnread(_) => health.check_unread(),
            HealthRunInputStepsItem::Keys(held) => {
                name(&named, &held.id);
                let answered = health.entry_keys().get(&held.id).await;
                answer = HealthRunResultStepsItemAnswer::Keys(HealthRunResultStepsItemAnswerKeys {
                    answer: shown_of(&answered),
                });
            }
            HealthRunInputStepsItem::Served(held) => {
                name(&named, &held.route.entry.id);
                let owned = routed(&entries, &taken, &held.route)?;
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
                health.served(owned.as_ref(), Some(&usage), keep, None);
            }
            HealthRunInputStepsItem::Failed(held) => {
                name(&named, &held.route.entry.id);
                let owned = routed(&entries, &taken, &held.route)?;
                health.failed(owned.as_ref(), &held.failure, held.onward, None);
            }
            HealthRunInputStepsItem::Billed(held) => {
                name(&named, &held.id);
                health.billed(
                    &held.id,
                    &Billing {
                        source: held.reply.source.clone(),
                        allowance: held.reply.allowance,
                        cost: held.reply.cost,
                    },
                    None,
                );
            }
            HealthRunInputStepsItem::Deferred(held) => {
                name(&named, &held.route.entry.id);
                let owned = routed(&entries, &taken, &held.route)?;
                let keep = (held.keep == Some(true)).then_some(&run);
                answer = HealthRunResultStepsItemAnswer::Skip(HealthRunResultStepsItemAnswerSkip {
                    skip: health.deferred(owned.as_ref(), keep),
                });
            }
            HealthRunInputStepsItem::Blocked(held) => {
                name(&named, &held.route.entry.id);
                let owned = routed(&entries, &taken, &held.route)?;
                let keep = (held.keep == Some(true)).then_some(&run);
                answer = HealthRunResultStepsItemAnswer::Blocked(
                    HealthRunResultStepsItemAnswerBlocked {
                        blocked: health.blocked(owned.as_ref(), keep),
                    },
                );
            }
            HealthRunInputStepsItem::TryAdmit(held) => {
                name(&named, &held.route.entry.id);
                let owned = routed(&entries, &taken, &held.route)?;
                let ticket = health.try_admit(owned.as_ref());
                if let Some(ticket) = &ticket {
                    tickets.insert(held.request.clone(), ticket.clone());
                }
                answer =
                    HealthRunResultStepsItemAnswer::Ticket(HealthRunResultStepsItemAnswerTicket {
                        ticket,
                    });
            }
            HealthRunInputStepsItem::Admit(held) => {
                name(&named, &held.route.entry.id);
                let owned = routed(&entries, &taken, &held.route)?;
                let route = owned.as_ref();
                if let Some(gate) = health.gate_state(route)
                    && (gate.phase == Phase::Paused
                        || (gate.phase == Phase::Probing && gate.probe.is_some()))
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
        // A check-starting step meets earlier armed holds and leaves them parked.
        let starts = matches!(
            step,
            HealthRunInputStepsItem::List(_)
                | HealthRunInputStepsItem::Key(_)
                | HealthRunInputStepsItem::Check(_)
                | HealthRunInputStepsItem::CheckUnread(_)
                | HealthRunInputStepsItem::Keys(_)
        );
        let mut met = false;
        if starts {
            for (at, hold) in holds.iter().enumerate() {
                if seen_before.get(at).copied().unwrap_or(false) {
                    continue;
                }
                met = true;
                hold.reached().await;
            }
        }
        if !met && !no_idle(step) {
            health.idle().await;
        }
        let mut sorted: Vec<String> = run.iter().cloned().collect();
        sorted.sort_by(|left, right| nyaaai_contract::order::compare_text(left, right));
        steps.push(HealthRunResultStepsItem {
            now_ms: net.wheel.now_ms(),
            answer,
            changes: std::mem::take(&mut *locked(&changes)),
            entries: locked(&named).iter().map(|id| health.state(id)).collect(),
            run: sorted,
            requests: net.played.requests().len() as i64,
            parked: holds
                .iter()
                .enumerate()
                .filter(|(_, hold)| hold.parked.load(Ordering::SeqCst))
                .map(|(at, _)| at as i64 + 1)
                .collect(),
        });
    }
    for hold in &holds {
        if hold.parked.load(Ordering::SeqCst) {
            hold.let_go(match hold.at {
                HealthRunInputStepsItemHoldAt::Check => Go {
                    state: Some(KeyState::None(KeyStateNone {})),
                    answer: None,
                },
                HealthRunInputStepsItemHoldAt::Store => Go {
                    state: None,
                    answer: Some(KeyAnswer::None),
                },
            });
        }
    }
    health.idle().await;
    health.dispose();
    entries.dispose();
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

/// A store whose answer and changes the row sets.
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
        let keys = Arc::new(Keys(Arc::clone(&store)));
        let entries = Arc::new(EntryRegistry::new(list, &*keys));
        let health = ProviderHealth::new(
            deps_of(&net, Arc::clone(&entries), keys, None),
            &CountedIds::new("gate-"),
        );
        let owned = Owned {
            entry: row.entry.clone(),
            key: "lab/model-a".to_owned(),
            incarnation: entries
                .incarnation(&row.entry.id)
                .unwrap_or_else(|| left_incarnation(&row.entry)),
        };
        let route = owned.as_ref();
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
            HealthKeysInputRowsItemPrepare::Rejected => {
                health.failed(
                    route,
                    &failure(
                        FailureCategory::KeyRejected,
                        FailureStatus::Number(401.0),
                        "no",
                    ),
                    false,
                    None,
                );
            }
            HealthKeysInputRowsItemPrepare::Unreachable => {
                health.failed(
                    route,
                    &failure(
                        FailureCategory::Network,
                        FailureStatus::Text("network".to_owned()),
                        "down",
                    ),
                    false,
                    None,
                );
            }
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
        entries.dispose();
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
