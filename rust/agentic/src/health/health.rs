//! Entry health: key status, bills, pool cooldowns and gates, read through ports.

use std::collections::{HashSet, VecDeque};
use std::future::Future;
use std::mem::discriminant;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, Weak};
use std::task::{Context, Poll, Waker};

use nyaaai_contract::cancel::{CancelToken, Unsubscribe};
use nyaaai_contract::entry_keys::{EntryKeys, KeyAnswer};
use nyaaai_contract::future::BoxFuture;
use nyaaai_contract::id::Ids;
use nyaaai_contract::shapes::health::{
    Admission, AdmissionRetired, Blocked, BlockedKeyRejected, EntryHealth, GateOutcome,
    GateSettled, HealthChange, HealthChangeChangedItem, PoolEvent, PoolEventAdmitted,
    PoolEventBilled, PoolEventExtraUsage, PoolEventRefused, PoolEventRefusedReason,
    PoolEventSettled, PoolVerdict, Ticket,
};
use nyaaai_contract::shapes::provider::{
    Billed, EntryFields, KeyState, KeyStateNone, KeyStateRejected, KeyStateUnreachable,
    ProviderEntry, ProviderKindName, SkipReason,
};
use nyaaai_contract::shapes::step::{Failure, FailureCategory, NormalizedUsage};
use nyaaai_contract::time::{Deadline, Scheduler};
use tokio::runtime::Handle;

use super::gate::{GateDeps, GateState, ProviderGate, WaitFn};
use super::key_reader::{Check, Failed, KEY_CHECK_MS, KeyReader, Shown};
use super::pool::{clear, step, verdict};
use crate::provider::incarnation::{
    EntryRegistry, Era, Incarnation, KeyStatus, RegistryEvent, TicketKey,
};
use crate::provider::kind::{KindDeps, KindField, ProviderKind};
use crate::provider::resolver::Resolved;

type Part = HealthChangeChangedItem;

type Listener = Arc<dyn Fn(&HealthChange) + Send + Sync>;

const PAUSE_MS: i64 = 60_000;

const RECHECK_MS: i64 = 60_000;

/// Declaration order of the shape's parts.
const PARTS: [Part; 4] = [Part::Key, Part::Billed, Part::Pool, Part::Waiting];

/// The fields health reads of a route.
#[derive(Clone, Copy)]
pub struct RouteRef<'a> {
    /// The row when resolved.
    pub entry: &'a ProviderEntry,
    pub key: &'a str,
    pub incarnation: &'a Arc<Incarnation>,
}

impl<'a> From<&'a Resolved> for RouteRef<'a> {
    fn from(route: &'a Resolved) -> Self {
        Self {
            entry: &route.entry,
            key: &route.key,
            incarnation: &route.incarnation,
        }
    }
}

/// What the lifecycle calls: per-route admission, cooldowns and skips.
pub trait Gate: Send + Sync {
    /// Waits through route pauses; `Retired` once its incarnation ended.
    fn admit(
        &self,
        route: RouteRef<'_>,
        cancel: &CancelToken,
        on_wait: Option<WaitFn>,
        again: bool,
    ) -> BoxFuture<'static, Admission>;

    /// Never waits; none once its incarnation ended.
    fn try_admit(&self, route: RouteRef<'_>) -> Option<Ticket>;

    fn settle(
        &self,
        ticket: &Ticket,
        outcome: GateOutcome,
        retry_after_ms: Option<i64>,
    ) -> GateSettled;

    /// Balance-cooldown exemption.
    fn deferred(&self, route: RouteRef<'_>, keep: Option<&HashSet<String>>) -> Option<SkipReason>;

    /// Last-route refusal.
    fn blocked(&self, route: RouteRef<'_>, keep: Option<&HashSet<String>>) -> Option<Blocked>;

    /// True when the report was dropped: its era ended.
    fn failed(
        &self,
        route: RouteRef<'_>,
        failure: &Failure,
        onward: bool,
        ticket: Option<&Ticket>,
    ) -> bool;

    /// Balance replies join `run`; true when the report was dropped as `failed` says.
    fn served(
        &self,
        route: RouteRef<'_>,
        usage: Option<&NormalizedUsage>,
        run: Option<&mut HashSet<String>>,
        ticket: Option<&Ticket>,
    ) -> bool;
}

/// One reply's billing.
#[derive(Debug, Clone, PartialEq)]
pub struct Billing {
    pub source: String,
    /// Weighted USD.
    pub allowance: Option<f64>,
    pub cost: Option<f64>,
}

pub struct HealthDeps {
    /// Where checks and waits run; nothing answers once it has ended.
    pub runtime: Handle,
    /// Gate pauses, rechecks and the time of every pool event.
    pub scheduler: Arc<dyn Scheduler>,
    /// Shared with the resolver.
    pub entries: Arc<EntryRegistry>,
    /// The store's keys; health offers `entry_keys` over them.
    pub keys: Arc<dyn EntryKeys>,
    pub kinds: Vec<Arc<dyn ProviderKind>>,
    /// What a kind's check stands on.
    pub kind_deps: Arc<dyn Fn(ProviderKindName) -> KindDeps + Send + Sync>,
    pub pause_ms: Option<i64>,
    pub max_pauses: Option<u64>,
}

/// What the health holds outside the eras.
#[derive(Default)]
struct State {
    /// Readers dropped while a check ran.
    retired: Vec<KeyReader>,
    /// Depth of the running changes whose telling waits for their end.
    batching: usize,
    pending: Vec<(String, Vec<Part>)>,
}

struct Core {
    runtime: Handle,
    scheduler: Arc<dyn Scheduler>,
    entries: Arc<EntryRegistry>,
    keys: Arc<dyn EntryKeys>,
    kinds: Vec<Arc<dyn ProviderKind>>,
    kind_deps: Arc<dyn Fn(ProviderKindName) -> KindDeps + Send + Sync>,
    pause_ms: i64,
    gate: ProviderGate,
    disposed: AtomicBool,
    /// Changes made and not yet told, oldest first.
    outbox: Mutex<VecDeque<HealthChange>>,
    delivering: AtomicBool,
    /// Guards the batch depth, the parts waiting to be told and the retired readers.
    state: Mutex<State>,
    listeners: Mutex<Vec<(u64, Listener)>>,
    next_listener: Mutex<u64>,
    unsubscribe: Mutex<Vec<Unsubscribe>>,
}

fn locked<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn none() -> KeyState {
    KeyState::None(KeyStateNone {})
}

fn key_of(route: RouteRef<'_>) -> String {
    format!("{}\n{}", route.entry.id, route.key)
}

fn ticket_key(ticket: &Ticket) -> TicketKey {
    (
        ticket.gate.clone(),
        ticket.model.clone(),
        ticket.era,
        ticket.generation,
        ticket.probe,
        ticket.serial,
    )
}

fn total(sum: Option<f64>, more: Option<f64>) -> Option<f64> {
    if sum.is_none() && more.is_none() {
        None
    } else {
        Some(sum.unwrap_or(0.0) + more.unwrap_or(0.0))
    }
}

fn billed_with(was: Option<&Billed>, reply: &Billing) -> Billed {
    Billed {
        billing: reply.source.clone(),
        allowance: total(was.and_then(|held| held.allowance), reply.allowance),
        cost: total(was.and_then(|held| held.cost), reply.cost),
    }
}

/// A future whose panic is its answer.
struct Caught<T>(BoxFuture<'static, T>);

impl<T> Future for Caught<T> {
    type Output = Result<T, ()>;

    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        match catch_unwind(AssertUnwindSafe(|| self.0.as_mut().poll(context))) {
            Ok(Poll::Ready(held)) => Poll::Ready(Ok(held)),
            Ok(Poll::Pending) => Poll::Pending,
            Err(_) => Poll::Ready(Err(())),
        }
    }
}

/// Runs the future to its first wait immediately.
fn eager<F>(future: F) -> BoxFuture<'static, F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    let mut future: BoxFuture<'static, F::Output> = Box::pin(future);
    match future
        .as_mut()
        .poll(&mut Context::from_waker(Waker::noop()))
    {
        Poll::Ready(held) => Box::pin(async move { held }),
        Poll::Pending => future,
    }
}

/// Ends a change's batch, and tells it once the last batch ends.
struct Batch<'a>(&'a Core);

impl Drop for Batch<'_> {
    fn drop(&mut self) {
        let last = {
            let mut state = locked(&self.0.state);
            state.batching -= 1;
            state.batching == 0
        };
        if last {
            self.0.flush();
        }
    }
}

impl Core {
    fn batch(&self) -> Batch<'_> {
        locked(&self.state).batching += 1;
        Batch(self)
    }

    fn touch(&self, id: &str, part: Part) {
        let now = {
            let mut state = locked(&self.state);
            match state.pending.iter_mut().find(|(held, _)| held == id) {
                Some((_, parts)) => {
                    if !parts.contains(&part) {
                        parts.push(part);
                    }
                }
                None => state.pending.push((id.to_owned(), vec![part])),
            }
            state.batching == 0
        };
        if now {
            self.flush();
        }
    }

    /// Tells parts in shape order, then change order; a listener's change queues behind.
    fn flush(&self) {
        let held = std::mem::take(&mut locked(&self.state).pending);
        if self.disposed.load(Ordering::SeqCst) {
            return;
        }
        {
            let mut outbox = locked(&self.outbox);
            for (entry, parts) in held {
                let changed: Vec<Part> = PARTS
                    .into_iter()
                    .filter(|part| parts.contains(part))
                    .collect();
                outbox.push_back(HealthChange { entry, changed });
            }
        }
        loop {
            if self.delivering.swap(true, Ordering::AcqRel) {
                return;
            }
            loop {
                let Some(change) = locked(&self.outbox).pop_front() else {
                    break;
                };
                let listeners: Vec<Listener> = locked(&self.listeners)
                    .iter()
                    .map(|(_, listener)| Arc::clone(listener))
                    .collect();
                for listener in listeners {
                    // A listener cannot change health.
                    let _ = catch_unwind(AssertUnwindSafe(|| listener(&change)));
                }
            }
            self.delivering.store(false, Ordering::Release);
            if locked(&self.outbox).is_empty() {
                return;
            }
        }
    }

    fn kind_of(&self, entry: &ProviderEntry) -> Option<&Arc<dyn ProviderKind>> {
        self.kinds.iter().find(|kind| kind.kind() == entry.kind)
    }

    fn by_url(&self, entry: &ProviderEntry) -> bool {
        self.kind_of(entry)
            .is_some_and(|kind| kind.field() == KindField::Url)
    }

    /// A URL kind that takes no key at all.
    fn keyless(&self, entry: &ProviderEntry) -> bool {
        self.kind_of(entry)
            .is_some_and(|kind| kind.field() == KindField::Url && !kind.optional_key())
    }

    fn now(&self) -> i64 {
        self.scheduler.now_ms()
    }

    /// A route of another registry is a host error; its door answers as a dropped route.
    fn owned(&self, incarnation: &Incarnation) -> bool {
        incarnation.belongs_to(&self.entries)
    }

    /// The current era of a listed id.
    fn era_of(&self, id: &str) -> Option<Arc<Era>> {
        self.entries.incarnation(id).map(|each| each.era())
    }

    /// The era's key state.
    fn status_of_era(era: &Era) -> KeyState {
        let held = era.state().status.clone();
        match held {
            Some(status) => locked(&status.state).clone(),
            None => none(),
        }
    }

    fn rejected(era: &Era) -> bool {
        matches!(Self::status_of_era(era), KeyState::Rejected(_))
    }

    /// The ticket's era, else the live one; none drops the report.
    fn era_for(&self, incarnation: &Incarnation, ticket: Option<&Ticket>) -> Option<Arc<Era>> {
        match ticket {
            Some(ticket) => self.entries.era(ticket.era),
            None => incarnation.live().then(|| incarnation.era()),
        }
    }

    /// Unhealthy entries are held out; asking rechecks an unreachable server.
    fn standing(self: &Arc<Self>, incarnation: &Incarnation) -> Option<KeyAnswer> {
        let era = incarnation.era();
        let status = Self::status_of_era(&era);
        if matches!(status, KeyState::Rejected(_)) {
            return Some(KeyAnswer::Refused);
        }
        if !self.by_url(&incarnation.entry()) {
            return None;
        }
        let unreachable = matches!(status, KeyState::Unreachable(_));
        if unreachable {
            self.recheck(&era);
        }
        // Out until the recheck passes.
        (unreachable || (matches!(status, KeyState::Checking(_)) && self.rechecking(&era)))
            .then_some(KeyAnswer::Unready)
    }

    /// A later turn may find it back.
    fn recheck(self: &Arc<Self>, era: &Arc<Era>) {
        let now = self.now();
        {
            let mut held = era.state();
            if let Some(last) = held.rechecked
                && now - last < RECHECK_MS
            {
                return;
            }
            held.rechecked = Some(now);
        }
        self.check(&era.id);
    }

    fn rechecking(&self, era: &Era) -> bool {
        let last = era.state().rechecked;
        last.is_some_and(|last| self.now() - last < KEY_CHECK_MS)
    }

    /// The only pool writer; false when the era had retired, and nothing was applied.
    fn apply(&self, era: &Era, event: PoolEvent) -> bool {
        let now = self.now() as f64;
        let changed = {
            let mut held = era.state();
            // The test and the apply share this lock; `retire` takes it to set the flag.
            if held.retired {
                return false;
            }
            let was = held.pool.clone();
            held.pool = step(&was, &event, now, self.pause_ms as f64);
            held.pool != was
        };
        if changed {
            self.touch(&era.id, Part::Pool);
        }
        true
    }

    fn admitted(&self, ticket: &Ticket) {
        let Some(era) = self.entries.era(ticket.era) else {
            return;
        };
        let _batch = self.batch();
        let now = self.now() as f64;
        let changed = {
            let mut held = era.state();
            if held.retired {
                return;
            }
            held.probes.insert(ticket_key(ticket));
            let was = held.pool.clone();
            held.pool = step(
                &was,
                &PoolEvent::Admitted(PoolEventAdmitted {
                    ticket: ticket.clone(),
                }),
                now,
                self.pause_ms as f64,
            );
            held.pool != was
        };
        if changed {
            self.touch(&era.id, Part::Pool);
        }
    }

    /// Settles the ticket's probe in the pool, when it had one.
    fn settled(&self, era: &Era, ticket: &Ticket) {
        let now = self.now() as f64;
        let changed = {
            let mut held = era.state();
            if held.retired || !held.probes.remove(&ticket_key(ticket)) {
                return;
            }
            let was = held.pool.clone();
            held.pool = step(
                &was,
                &PoolEvent::Settled(PoolEventSettled {
                    ticket: ticket.clone(),
                }),
                now,
                self.pause_ms as f64,
            );
            held.pool != was
        };
        if changed {
            self.touch(&era.id, Part::Pool);
        }
    }

    /// Extra usage is read live; `keep` exempts balance.
    fn verdict(
        &self,
        incarnation: &Incarnation,
        keep: Option<&HashSet<String>>,
    ) -> Option<PoolVerdict> {
        let exempt = incarnation.entry().extra_usage == Some(true)
            || keep.is_some_and(|keep| keep.contains(&incarnation.id));
        let pool = incarnation.era().state().pool.clone();
        verdict(&pool, exempt, self.now() as f64)
    }

    /// Inference stands until recheck; false when the era had retired.
    fn mark(self: &Arc<Self>, era: &Arc<Era>, state: KeyState) -> bool {
        let Some(status) = self.status_in(era) else {
            return false;
        };
        let changed = {
            // The test and the mark share the era's lock.
            let held = era.state();
            if held.retired {
                return false;
            }
            let mut current = locked(&status.state);
            if discriminant(&*current) == discriminant(&state) {
                false
            } else {
                *current = state;
                true
            }
        };
        if changed {
            self.touch(&era.id, Part::Key);
        }
        true
    }

    /// The era's status, made on first use; none once the era retired.
    fn status_in(self: &Arc<Self>, era: &Arc<Era>) -> Option<Arc<KeyStatus>> {
        let mut held = era.state();
        if held.retired {
            return None;
        }
        if let Some(known) = &held.status {
            return Some(Arc::clone(known));
        }
        let state = Arc::new(Mutex::new(none()));
        let check: Check = {
            let (core, era) = (Arc::downgrade(self), Arc::downgrade(era));
            Arc::new(move |limit| match (core.upgrade(), era.upgrade()) {
                (Some(core), Some(era)) => core.key_check(era, limit),
                _ => Box::pin(async { Err(Failed) }),
            })
        };
        let shown: Shown = {
            let (weak, id, state): (Weak<Core>, String, _) =
                (Arc::downgrade(self), era.id.clone(), Arc::clone(&state));
            Arc::new(move |next| {
                *locked(&state) = next;
                if let Some(core) = weak.upgrade() {
                    core.touch(&id, Part::Key);
                }
            })
        };
        let fresh = Arc::new(KeyStatus {
            state,
            reader: KeyReader::new(
                Arc::clone(&self.scheduler),
                self.runtime.clone(),
                check,
                shown,
            ),
        });
        held.status = Some(Arc::clone(&fresh));
        Some(fresh)
    }

    fn key_check(
        self: &Arc<Self>,
        era: Arc<Era>,
        limit: Arc<Deadline>,
    ) -> BoxFuture<'static, Result<Option<KeyState>, Failed>> {
        let core = Arc::clone(self);
        Box::pin(async move { core.key_state(&era, &limit).await })
    }

    /// Ends without effect once its token ended or its era retired.
    async fn key_state(
        &self,
        era: &Arc<Era>,
        limit: &Deadline,
    ) -> Result<Option<KeyState>, Failed> {
        let Some(incarnation) = era.incarnation() else {
            return Ok(None);
        };
        let entry = incarnation.entry();
        let Some(kind) = self.kind_of(&entry).cloned() else {
            return Ok(Some(none()));
        };
        let stale = || limit.token().is_cancelled() || era.retired();
        let answer = self.keys.get(&entry.id).await;
        if stale() {
            return Ok(None);
        }
        let state = match answer {
            KeyAnswer::Refused => KeyState::Rejected(KeyStateRejected {}),
            KeyAnswer::Unready if kind.field() == KindField::Key || kind.optional_key() => {
                KeyState::Unreachable(KeyStateUnreachable {})
            }
            held => {
                let secret = match held {
                    KeyAnswer::Key(key) => Some(key),
                    _ => None,
                };
                if kind.field() == KindField::Key && secret.is_none() {
                    none()
                } else {
                    let fields = EntryFields {
                        kind: entry.kind,
                        label: entry.label.clone(),
                        plan: entry.plan,
                        url: entry.url.clone(),
                        extra_usage: entry.extra_usage,
                    };
                    let deps = (self.kind_deps)(entry.kind);
                    kind.check(&fields, secret.as_deref(), &deps, limit.token())
                        .await
                        .map_err(|_| Failed)?
                }
            }
        };
        Ok(if stale() { None } else { Some(state) })
    }

    /// Replaces pending checks.
    fn check(self: &Arc<Self>, id: &str) {
        let Some(incarnation) = self.entries.incarnation(id) else {
            return;
        };
        if self.disposed.load(Ordering::SeqCst) {
            return;
        }
        let _batch = self.batch();
        if let Some(status) = self.status_in(&incarnation.era()) {
            drop(status.reader.read());
        }
    }

    /// Applies the registry's changes in order.
    fn observe(self: &Arc<Self>, events: &[RegistryEvent]) {
        let _batch = self.batch();
        for event in events {
            match event {
                RegistryEvent::Retired(incarnation) => self.retire(&incarnation.era()),
                RegistryEvent::Rekeyed {
                    incarnation,
                    retired,
                } => {
                    self.retire(retired);
                    self.check(&incarnation.id);
                }
                RegistryEvent::Added(incarnation) => self.check(&incarnation.id),
                RegistryEvent::ExtraUsage { incarnation, on } => {
                    let era = incarnation.era();
                    let last_bill = era.state().bill.as_ref().map(|held| held.billing.clone());
                    self.apply(
                        &era,
                        PoolEvent::ExtraUsage(PoolEventExtraUsage { on: *on, last_bill }),
                    );
                }
            }
        }
    }

    /// Ends what a retired era held.
    fn retire(&self, era: &Arc<Era>) {
        let _batch = self.batch();
        let (status, billed, pooled) = {
            let mut held = era.state();
            (
                held.status.take(),
                held.bill.is_some(),
                held.pool != clear(),
            )
        };
        if let Some(status) = status {
            status.reader.dispose();
            if status.reader.busy() {
                locked(&self.state).retired.push(status.reader.clone());
            }
            if !matches!(*locked(&status.state), KeyState::None(_)) {
                self.touch(&era.id, Part::Key);
            }
        }
        if billed {
            self.touch(&era.id, Part::Billed);
        }
        if pooled {
            self.touch(&era.id, Part::Pool);
        }
        self.gate.retire(&era.gates);
    }
}

/// Entry health and routing.
#[derive(Clone)]
pub struct ProviderHealth {
    core: Arc<Core>,
}

/// The keys a resolver reads: the store's, unless health holds the entry out.
struct HealthKeys {
    core: Arc<Core>,
}

impl EntryKeys for HealthKeys {
    fn get(&self, id: &str) -> BoxFuture<'_, KeyAnswer> {
        let (core, id) = (Arc::clone(&self.core), id.to_owned());
        let work = eager(Caught(Box::pin(async move {
            let Some(incarnation) = core.entries.incarnation(&id) else {
                return KeyAnswer::None;
            };
            if let Some(standing) = core.standing(&incarnation) {
                return standing;
            }
            let answer = core.keys.get(&id).await;
            // A URL kind needs no key, so an unreadable store is no key.
            if answer == KeyAnswer::Unready && core.keyless(&incarnation.entry()) {
                return KeyAnswer::None;
            }
            answer
        })));
        Box::pin(async move { work.await.unwrap_or(KeyAnswer::Unready) })
    }

    fn on_change(&self, listener: Box<dyn Fn(&str) + Send + Sync>) -> Unsubscribe {
        self.core.entries.on_key_change(Arc::from(listener))
    }
}

impl ProviderHealth {
    /// Defers initial key checks; asks `ids` once, for the gate's name.
    pub fn new(deps: HealthDeps, ids: &dyn Ids) -> Self {
        let core = Arc::new_cyclic(|weak: &Weak<Core>| {
            let told = weak.clone();
            let gate = ProviderGate::new(
                GateDeps {
                    runtime: deps.runtime.clone(),
                    scheduler: Arc::clone(&deps.scheduler),
                    pause_ms: deps.pause_ms,
                    max_pauses: deps.max_pauses,
                    on_ticket: Some(Arc::new(move |ticket: &Ticket| {
                        if let Some(core) = told.upgrade() {
                            core.admitted(ticket);
                        }
                    })),
                    on_withdrawn: Some(Arc::new({
                        let gone = weak.clone();
                        move |ticket: &Ticket| {
                            if let Some(core) = gone.upgrade()
                                && let Some(era) = core.entries.era(ticket.era)
                            {
                                let _batch = core.batch();
                                core.settled(&era, ticket);
                            }
                        }
                    })),
                },
                ids,
            );
            Core {
                runtime: deps.runtime,
                scheduler: deps.scheduler,
                entries: deps.entries,
                keys: deps.keys,
                kinds: deps.kinds,
                kind_deps: deps.kind_deps,
                pause_ms: deps.pause_ms.unwrap_or(PAUSE_MS),
                gate,
                disposed: AtomicBool::new(false),
                outbox: Mutex::new(VecDeque::new()),
                delivering: AtomicBool::new(false),
                state: Mutex::new(State::default()),
                listeners: Mutex::new(Vec::new()),
                next_listener: Mutex::new(0),
                unsubscribe: Mutex::new(Vec::new()),
            }
        });
        let watch_off = {
            let weak = Arc::downgrade(&core);
            core.entries.watch(Arc::new(move |events| {
                if let Some(core) = weak.upgrade() {
                    core.observe(events);
                }
            }))
        };
        let gate_off = {
            let weak = Arc::downgrade(&core);
            core.gate.on_change(Box::new(move |owner, _model| {
                if let Some(core) = weak.upgrade() {
                    core.touch(owner, Part::Waiting);
                }
            }))
        };
        locked(&core.unsubscribe).extend([watch_off, gate_off]);
        Self { core }
    }

    /// The keys a resolver reads.
    pub fn entry_keys(&self) -> Arc<dyn EntryKeys> {
        Arc::new(HealthKeys {
            core: Arc::clone(&self.core),
        })
    }

    /// Cached key state.
    pub fn status(&self, id: &str) -> KeyState {
        self.core
            .era_of(id)
            .map_or_else(none, |era| Core::status_of_era(&era))
    }

    /// Listed replies create bills.
    pub fn bill(&self, id: &str) -> Option<Billed> {
        self.core.era_of(id)?.state().bill.clone()
    }

    /// Everything health tracks of one entry.
    pub fn state(&self, id: &str) -> EntryHealth {
        let era = self.core.era_of(id);
        let prefix = format!("{id}\n");
        EntryHealth {
            entry: id.to_owned(),
            key: self.status(id),
            billed: self.bill(id),
            pool: era
                .as_ref()
                .map_or_else(clear, |era| era.state().pool.clone()),
            waiting: era.map_or_else(Vec::new, |era| {
                self.core
                    .gate
                    .holding(&era.gates)
                    .iter()
                    .filter_map(|model| model.strip_prefix(&prefix))
                    .map(str::to_owned)
                    .collect()
            }),
        }
    }

    /// One model's gate as data; none until it is first used.
    pub fn gate_state(&self, route: RouteRef<'_>) -> Option<GateState> {
        (route.incarnation.live() && self.core.owned(route.incarnation))
            .then(|| {
                self.core
                    .gate
                    .state_of(&route.incarnation.era().gates, &key_of(route))
            })
            .flatten()
    }

    /// A ticket's era takes the bill; with none, the route's incarnation does.
    pub fn billed(
        &self,
        id: &str,
        reply: &Billing,
        ticket: Option<&Ticket>,
        incarnation: Option<&Incarnation>,
    ) {
        if incarnation.is_some_and(|held| !self.core.owned(held)) {
            return;
        }
        let era = match (ticket, incarnation) {
            (Some(ticket), _) => self.core.entries.era(ticket.era),
            (None, Some(held)) => held.live().then(|| held.era()),
            (None, None) => self.core.era_of(id),
        };
        let Some(era) = era else {
            return;
        };
        {
            let mut held = era.state();
            if held.retired {
                return;
            }
            let next = billed_with(held.bill.as_ref(), reply);
            if held.bill.as_ref() == Some(&next) {
                return;
            }
            held.bill = Some(next);
        }
        self.core.touch(&era.id, Part::Billed);
    }

    /// Replaces pending checks.
    pub fn check(&self, id: &str) {
        self.core.check(id);
    }

    pub fn check_unread(&self) {
        for incarnation in self.core.entries.incarnations() {
            let unread = incarnation.era().state().status.is_none();
            if unread {
                self.core.check(&incarnation.id);
            }
        }
    }

    /// Answers when no key check is running, having waited for those that were.
    pub async fn idle(&self) {
        loop {
            let mut busy: Vec<KeyReader> = self
                .core
                .entries
                .incarnations()
                .iter()
                .filter_map(|incarnation| incarnation.era().state().status.clone())
                .map(|status| status.reader.clone())
                .collect();
            busy.extend(locked(&self.core.state).retired.iter().cloned());
            busy.retain(KeyReader::busy);
            if busy.is_empty() {
                locked(&self.core.state).retired.clear();
                return;
            }
            for reader in busy {
                reader.idle().await;
            }
        }
    }

    /// Told each change once, with its parts in the shape's order; the result unsubscribes.
    pub fn on_change(&self, listener: Box<dyn Fn(&HealthChange) + Send + Sync>) -> Unsubscribe {
        let id = {
            let mut next = locked(&self.core.next_listener);
            *next += 1;
            *next
        };
        locked(&self.core.listeners).push((id, Arc::from(listener)));
        let weak = Arc::downgrade(&self.core);
        Box::new(move || {
            if let Some(core) = weak.upgrade() {
                locked(&core.listeners).retain(|(held, _)| *held != id);
            }
        })
    }

    pub fn dispose(&self) {
        self.core.disposed.store(true, Ordering::SeqCst);
        let offs = std::mem::take(&mut *locked(&self.core.unsubscribe));
        for off in offs {
            off();
        }
        for incarnation in self.core.entries.incarnations() {
            let era = incarnation.era();
            let status = {
                let mut held = era.state();
                held.bill = None;
                held.pool = clear();
                held.probes.clear();
                held.status.take()
            };
            if let Some(status) = status {
                status.reader.dispose();
            }
        }
        locked(&self.core.listeners).clear();
        self.core.gate.dispose();
    }
}

impl Gate for ProviderHealth {
    /// A key change retires old-era waiters, which ask again under the new era's open gate.
    fn admit(
        &self,
        route: RouteRef<'_>,
        cancel: &CancelToken,
        on_wait: Option<WaitFn>,
        again: bool,
    ) -> BoxFuture<'static, Admission> {
        let incarnation = Arc::clone(route.incarnation);
        let retired = || Admission::Retired(AdmissionRetired {});
        if !incarnation.live() || !self.core.owned(&incarnation) {
            return Box::pin(async move { retired() });
        }
        let (core, key, cancel) = (Arc::clone(&self.core), key_of(route), cancel.clone());
        let first = {
            let _batch = core.batch();
            core.gate.admit(
                &incarnation.era().gates,
                &key,
                &cancel,
                on_wait.clone(),
                again,
            )
        };
        Box::pin(async move {
            let mut asked: Pin<Box<dyn Future<Output = Admission> + Send>> = Box::pin(first);
            loop {
                let admission = asked.await;
                // A key change moves the waiter to the new era.
                if !matches!(admission, Admission::Retired(_)) || !incarnation.live() {
                    return admission;
                }
                let _batch = core.batch();
                asked = Box::pin(core.gate.admit(
                    &incarnation.era().gates,
                    &key,
                    &cancel,
                    on_wait.clone(),
                    again,
                ));
            }
        })
    }

    fn try_admit(&self, route: RouteRef<'_>) -> Option<Ticket> {
        let incarnation = route.incarnation;
        if !incarnation.live() || !self.core.owned(incarnation) {
            return None;
        }
        let _batch = self.core.batch();
        self.core
            .gate
            .try_admit(&incarnation.era().gates, &key_of(route))
    }

    fn settle(
        &self,
        ticket: &Ticket,
        outcome: GateOutcome,
        retry_after_ms: Option<i64>,
    ) -> GateSettled {
        let Some(era) = self.core.entries.era(ticket.era) else {
            return GateSettled::Done;
        };
        if self.core.disposed.load(Ordering::SeqCst) {
            return GateSettled::Done;
        }
        let _batch = self.core.batch();
        self.core.settled(&era, ticket);
        self.core
            .gate
            .settle(&era.gates, ticket, outcome, retry_after_ms)
    }

    fn deferred(&self, route: RouteRef<'_>, keep: Option<&HashSet<String>>) -> Option<SkipReason> {
        let incarnation = route.incarnation;
        if !self.core.owned(incarnation) {
            return Some(SkipReason::Unavailable);
        }
        let era = incarnation.era();
        if Core::rejected(&era) {
            return Some(SkipReason::KeyRejected);
        }
        if let Some(pool) = self.core.verdict(incarnation, keep) {
            return Some(pool.skip);
        }
        if self.core.gate.ready(&era.gates, &key_of(route)) {
            None
        } else {
            Some(SkipReason::Paused)
        }
    }

    fn blocked(&self, route: RouteRef<'_>, keep: Option<&HashSet<String>>) -> Option<Blocked> {
        let incarnation = route.incarnation;
        if !self.core.owned(incarnation) {
            return None;
        }
        if Core::rejected(&incarnation.era()) {
            return Some(Blocked::KeyRejected(BlockedKeyRejected {}));
        }
        self.core.verdict(incarnation, keep)?.blocked
    }

    /// Balance replies extend `run`.
    fn served(
        &self,
        route: RouteRef<'_>,
        usage: Option<&NormalizedUsage>,
        run: Option<&mut HashSet<String>>,
        ticket: Option<&Ticket>,
    ) -> bool {
        if !self.core.owned(route.incarnation) {
            return true;
        }
        let Some(source) = usage.and_then(|held| held.billing_source.clone()) else {
            return false;
        };
        let Some(era) = self.core.era_for(route.incarnation, ticket) else {
            return true;
        };
        // Settings may change mid-request.
        let entry = route.incarnation.entry();
        let applied = self.core.apply(
            &era,
            PoolEvent::Billed(PoolEventBilled {
                source: source.clone(),
                plan: entry.plan.map(|plan| plan.as_str().to_owned()),
                extra_usage: entry.extra_usage == Some(true),
            }),
        );
        if !applied {
            return true;
        }
        if source == "balance"
            && let Some(run) = run
        {
            run.insert(entry.id);
        }
        false
    }

    fn failed(
        &self,
        route: RouteRef<'_>,
        failure: &Failure,
        onward: bool,
        ticket: Option<&Ticket>,
    ) -> bool {
        let core = &self.core;
        if !core.owned(route.incarnation) {
            return true;
        }
        let Some(era) = core.era_for(route.incarnation, ticket) else {
            return true;
        };
        let entry = route.entry;
        let _batch = core.batch();
        let cool = |era: &Arc<Era>| {
            core.gate
                .cool(&era.gates, &key_of(route), failure.retry_after_ms);
            true
        };
        let applied = match failure.category {
            FailureCategory::KeyRejected => {
                core.mark(&era, KeyState::Rejected(KeyStateRejected {}))
            }
            // One pool per entry.
            FailureCategory::Allowance | FailureCategory::OutOfCredit => core.apply(
                &era,
                PoolEvent::Refused(PoolEventRefused {
                    reason: if failure.category == FailureCategory::Allowance {
                        PoolEventRefusedReason::Allowance
                    } else {
                        PoolEventRefusedReason::Credit
                    },
                    resets_at: failure.resets_at.map(|held| held as f64),
                    retry_after_ms: failure.retry_after_ms.map(|held| held as f64),
                }),
            ),
            FailureCategory::Network => {
                if core.by_url(entry) {
                    core.mark(&era, KeyState::Unreachable(KeyStateUnreachable {}))
                } else {
                    !onward || cool(&era)
                }
            }
            // Settlement owns overload pauses.
            FailureCategory::Overloaded
            | FailureCategory::KeyMissing
            | FailureCategory::Refused
            | FailureCategory::BadRequest
            | FailureCategory::ReplayRejected => true,
            _ => !onward || cool(&era),
        };
        !applied
    }
}

#[cfg(test)]
mod tests {
    use nyaaai_contract::id::CountedIds;
    use nyaaai_contract::shapes::step::FailureStatus;
    use nyaaai_testkit::scheduler::{FakeScheduler, TimerWheel};
    use nyaaai_testkit::scripted_ports::{ScriptedEntryKeys, ScriptedProviderList};

    use super::*;

    #[test]
    fn a_route_of_another_registry_is_refused_at_every_door() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        let scheduler = Arc::new(FakeScheduler::new(Arc::new(TimerWheel::new(0))));
        let entry = ProviderEntry {
            id: "a".to_owned(),
            kind: ProviderKindName::Openrouter,
            label: "First".to_owned(),
            plan: None,
            url: None,
            extra_usage: None,
        };
        let list = Arc::new(ScriptedProviderList::new(vec![entry.clone()]));
        let keys = Arc::new(ScriptedEntryKeys::new());
        let own = Arc::new(EntryRegistry::new(list.clone(), &*keys));
        let other = EntryRegistry::new(list, &*keys);
        let health = ProviderHealth::new(
            HealthDeps {
                runtime: runtime.handle().clone(),
                scheduler,
                entries: Arc::clone(&own),
                keys: keys.clone(),
                kinds: Vec::new(),
                kind_deps: Arc::new(|_| unreachable!("no kind asks")),
                pause_ms: None,
                max_pauses: None,
            },
            &CountedIds::new("gate-"),
        );
        let route = |registry: &EntryRegistry| {
            let incarnation = registry.incarnation("a").expect("listed");
            (entry.clone(), incarnation)
        };
        let (row, mine) = route(&own);
        let (_, foreign) = route(&other);
        let mine = RouteRef {
            entry: &row,
            key: "lab/model-a",
            incarnation: &mine,
        };
        let foreign = RouteRef {
            incarnation: &foreign,
            ..mine
        };
        let failure = Failure {
            category: FailureCategory::KeyRejected,
            status: FailureStatus::Number(401.0),
            retry_after_ms: None,
            resets_at: None,
            message: "no".to_owned(),
            reasons: None,
            billed: None,
        };
        assert!(health.try_admit(mine).is_some());
        assert!(health.try_admit(foreign).is_none());
        assert!(matches!(
            runtime.block_on(health.admit(foreign, &CancelToken::new(), None, false)),
            Admission::Retired(_)
        ));
        assert!(health.gate_state(mine).is_some());
        assert!(health.gate_state(foreign).is_none());
        assert_eq!(
            health.deferred(foreign, None),
            Some(SkipReason::Unavailable)
        );
        assert!(health.blocked(foreign, None).is_none());
        assert!(health.served(foreign, None, None, None));
        assert!(health.failed(foreign, &failure, false, None));
        assert!(matches!(health.status("a"), KeyState::None(_)));
        health.dispose();
        other.dispose();
        own.dispose();
    }
}
