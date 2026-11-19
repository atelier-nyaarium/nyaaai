//! Entry health: key status, bills, pool cooldowns and gates, read through ports.

use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::mem::discriminant;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::pin::Pin;
use std::sync::{Arc, Mutex, MutexGuard, Weak};
use std::task::{Context, Poll, Waker};

use nyaaai_contract::cancel::{CancelToken, Unsubscribe};
use nyaaai_contract::entry_keys::{EntryKeys, KeyAnswer};
use nyaaai_contract::future::BoxFuture;
use nyaaai_contract::id::Ids;
use nyaaai_contract::provider_list::ProviderList;
use nyaaai_contract::shapes::health::{
    Admission, Blocked, BlockedKeyRejected, EntryHealth, GateOutcome, GateSettled, HealthChange,
    HealthChangeChangedItem, PoolEvent, PoolEventAdmitted, PoolEventBilled, PoolEventExtraUsage,
    PoolEventRefused, PoolEventRefusedReason, PoolEventSettled, PoolState, PoolVerdict, Ticket,
};
use nyaaai_contract::shapes::provider::{
    Billed, EntryFields, KeyState, KeyStateNone, KeyStateRejected, KeyStateUnreachable,
    ProviderEntry, ProviderKindName, ProviderPlan, SkipReason,
};
use nyaaai_contract::shapes::step::{Failure, FailureCategory, NormalizedUsage};
use nyaaai_contract::time::{Deadline, Scheduler};

use super::gate::{GateDeps, GateState, ProviderGate, WaitFn};
use super::key_reader::{Check, Failed, KEY_CHECK_MS, KeyReader, Shown};
use super::pool::{clear, step, verdict};
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
    pub entry: &'a ProviderEntry,
    pub key: &'a str,
}

impl<'a> From<&'a Resolved> for RouteRef<'a> {
    fn from(route: &'a Resolved) -> Self {
        Self {
            entry: &route.entry,
            key: &route.key,
        }
    }
}

/// What the lifecycle calls: per-route admission, cooldowns and skips.
pub trait Gate: Send + Sync {
    /// Waits through route pauses.
    fn admit(
        &self,
        route: RouteRef<'_>,
        cancel: &CancelToken,
        on_wait: Option<WaitFn>,
        again: bool,
    ) -> BoxFuture<'static, Admission>;

    /// Never waits.
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

    fn failed(&self, route: RouteRef<'_>, failure: &Failure, onward: bool);

    /// Balance replies join `run`.
    fn served(
        &self,
        route: RouteRef<'_>,
        usage: Option<&NormalizedUsage>,
        run: Option<&mut HashSet<String>>,
    );
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
    /// Gate pauses, rechecks and the time of every pool event.
    pub scheduler: Arc<dyn Scheduler>,
    /// The entries in priority order.
    pub list: Arc<dyn ProviderList>,
    /// The store's keys; health offers `entry_keys` over them.
    pub keys: Arc<dyn EntryKeys>,
    pub kinds: Vec<Arc<dyn ProviderKind>>,
    /// What a kind's check stands on.
    pub kind_deps: Arc<dyn Fn(ProviderKindName) -> KindDeps + Send + Sync>,
    pub pause_ms: Option<i64>,
    pub max_pauses: Option<u64>,
}

struct Status {
    state: Arc<Mutex<KeyState>>,
    reader: KeyReader,
}

/// Tracked provider fields.
#[derive(Clone, PartialEq)]
struct Identity {
    plan: Option<ProviderPlan>,
    url: Option<String>,
    extra_usage: bool,
}

type TicketKey = (String, String, u64, bool, u64);

#[derive(Default)]
struct State {
    statuses: HashMap<String, Arc<Status>>,
    /// Readers dropped while a check ran.
    retired: Vec<KeyReader>,
    rechecked: HashMap<String, i64>,
    bills: HashMap<String, Billed>,
    /// Absent pool means clear.
    pools: HashMap<String, PoolState>,
    /// Admitted tickets' entries, by the ticket's data.
    probes: HashMap<TicketKey, String>,
    identities: Vec<(String, Identity)>,
    /// Depth of the running changes whose telling waits for their end.
    batching: usize,
    pending: Vec<(String, Vec<Part>)>,
}

struct Core {
    scheduler: Arc<dyn Scheduler>,
    list: Arc<dyn ProviderList>,
    keys: Arc<dyn EntryKeys>,
    kinds: Vec<Arc<dyn ProviderKind>>,
    kind_deps: Arc<dyn Fn(ProviderKindName) -> KindDeps + Send + Sync>,
    pause_ms: i64,
    gate: ProviderGate,
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

/// The entry a gate's model key belongs to.
fn entry_of_model(model: &str) -> &str {
    model.split_once('\n').map_or(model, |(entry, _)| entry)
}

fn ticket_key(ticket: &Ticket) -> TicketKey {
    (
        ticket.gate.clone(),
        ticket.model.clone(),
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

fn identity_of(entry: &ProviderEntry) -> Identity {
    Identity {
        plan: entry.plan,
        url: entry.url.clone(),
        extra_usage: entry.extra_usage == Some(true),
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

/// Runs the future up to its first wait now, as the start of an `async` function does.
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

    /// Tells each entry's parts once, in the shape's order.
    fn flush(&self) {
        let held = std::mem::take(&mut locked(&self.state).pending);
        for (entry, parts) in held {
            let changed: Vec<Part> = PARTS
                .into_iter()
                .filter(|part| parts.contains(part))
                .collect();
            let change = HealthChange { entry, changed };
            let listeners: Vec<Listener> = locked(&self.listeners)
                .iter()
                .map(|(_, listener)| Arc::clone(listener))
                .collect();
            for listener in listeners {
                // A listener cannot change health.
                let _ = catch_unwind(AssertUnwindSafe(|| listener(&change)));
            }
        }
    }

    fn entries(&self) -> Vec<ProviderEntry> {
        self.list.entries()
    }

    fn entry_of(&self, id: &str) -> Option<ProviderEntry> {
        self.entries().into_iter().find(|entry| entry.id == id)
    }

    fn listed(&self, id: &str) -> bool {
        self.entry_of(id).is_some()
    }

    fn kind_of(&self, entry: &ProviderEntry) -> Option<&Arc<dyn ProviderKind>> {
        self.kinds.iter().find(|kind| kind.kind() == entry.kind)
    }

    fn by_url(&self, entry: &ProviderEntry) -> bool {
        self.kind_of(entry)
            .is_some_and(|kind| kind.field() == KindField::Url)
    }

    fn status(&self, id: &str) -> KeyState {
        let held = locked(&self.state).statuses.get(id).cloned();
        match held {
            Some(status) => locked(&status.state).clone(),
            None => none(),
        }
    }

    fn rejected(&self, entry: &ProviderEntry) -> bool {
        matches!(self.status(&entry.id), KeyState::Rejected(_))
    }

    fn now(&self) -> i64 {
        self.scheduler.now_ms()
    }

    /// Unhealthy entries are held out; asking rechecks an unreachable server.
    fn standing(self: &Arc<Self>, entry: &ProviderEntry) -> Option<KeyAnswer> {
        let status = self.status(&entry.id);
        if matches!(status, KeyState::Rejected(_)) {
            return Some(KeyAnswer::Refused);
        }
        if !self.by_url(entry) {
            return None;
        }
        let unreachable = matches!(status, KeyState::Unreachable(_));
        if unreachable {
            self.recheck(&entry.id);
        }
        // Out until the recheck passes.
        (unreachable || (matches!(status, KeyState::Checking(_)) && self.rechecking(&entry.id)))
            .then_some(KeyAnswer::Unready)
    }

    /// A later turn may find it back.
    fn recheck(self: &Arc<Self>, id: &str) {
        let now = self.now();
        {
            let mut state = locked(&self.state);
            if let Some(last) = state.rechecked.get(id)
                && now - last < RECHECK_MS
            {
                return;
            }
            state.rechecked.insert(id.to_owned(), now);
        }
        self.check(id);
    }

    fn rechecking(&self, id: &str) -> bool {
        let last = locked(&self.state).rechecked.get(id).copied();
        last.is_some_and(|last| self.now() - last < KEY_CHECK_MS)
    }

    /// The only pool writer.
    fn apply(&self, id: &str, event: PoolEvent) {
        let now = self.now() as f64;
        let changed = {
            let mut state = locked(&self.state);
            let was = state.pools.get(id).cloned().unwrap_or_else(clear);
            let next = step(&was, &event, now, self.pause_ms as f64);
            if matches!(next, PoolState::Clear(_)) {
                state.pools.remove(id);
            } else {
                state.pools.insert(id.to_owned(), next.clone());
            }
            next != was
        };
        if changed {
            self.touch(id, Part::Pool);
        }
    }

    fn admitted(&self, id: &str, ticket: Ticket) {
        let _batch = self.batch();
        locked(&self.state)
            .probes
            .insert(ticket_key(&ticket), id.to_owned());
        self.apply(id, PoolEvent::Admitted(PoolEventAdmitted { ticket }));
    }

    /// Extra usage or `keep` exempts balance.
    fn verdict(
        &self,
        entry: &ProviderEntry,
        keep: Option<&HashSet<String>>,
    ) -> Option<PoolVerdict> {
        let exempt =
            entry.extra_usage == Some(true) || keep.is_some_and(|keep| keep.contains(&entry.id));
        let pool = locked(&self.state)
            .pools
            .get(&entry.id)
            .cloned()
            .unwrap_or_else(clear);
        verdict(&pool, exempt, self.now() as f64)
    }

    /// Inference stands until recheck.
    fn mark(self: &Arc<Self>, id: &str, state: KeyState) {
        if !self.listed(id) {
            return;
        }
        let status = self.status_of(id);
        let changed = {
            let mut held = locked(&status.state);
            if discriminant(&*held) == discriminant(&state) {
                false
            } else {
                *held = state;
                true
            }
        };
        if changed {
            self.touch(id, Part::Key);
        }
    }

    fn status_of(self: &Arc<Self>, id: &str) -> Arc<Status> {
        if let Some(known) = locked(&self.state).statuses.get(id) {
            return Arc::clone(known);
        }
        let state = Arc::new(Mutex::new(none()));
        let check: Check = {
            let (weak, id) = (Arc::downgrade(self), id.to_owned());
            Arc::new(move |limit| match weak.upgrade() {
                Some(core) => core.key_check(&id, limit),
                None => Box::pin(async { Err(Failed) }),
            })
        };
        let shown: Shown = {
            let (weak, id, state): (Weak<Core>, String, _) =
                (Arc::downgrade(self), id.to_owned(), Arc::clone(&state));
            Arc::new(move |next| {
                *locked(&state) = next;
                if let Some(core) = weak.upgrade() {
                    core.touch(&id, Part::Key);
                }
            })
        };
        let fresh = Arc::new(Status {
            state,
            reader: KeyReader::new(Arc::clone(&self.scheduler), check, shown),
        });
        locked(&self.state)
            .statuses
            .insert(id.to_owned(), Arc::clone(&fresh));
        fresh
    }

    fn key_check(
        self: &Arc<Self>,
        id: &str,
        limit: Arc<Deadline>,
    ) -> BoxFuture<'static, Result<KeyState, Failed>> {
        let (core, id) = (Arc::clone(self), id.to_owned());
        let work = eager(Caught(Box::pin(async move {
            core.key_state(&id, &limit).await
        })));
        Box::pin(async move { work.await.unwrap_or(Err(Failed)) })
    }

    async fn key_state(&self, id: &str, limit: &Deadline) -> Result<KeyState, Failed> {
        let entry = self.entry_of(id);
        let kind = entry
            .as_ref()
            .and_then(|entry| self.kind_of(entry))
            .cloned();
        let (Some(entry), Some(kind)) = (entry, kind) else {
            return Ok(none());
        };
        let answer = self.keys.get(id).await;
        // The store's answer arrives after a turn, so a change that follows the start drops the check.
        Turn(false).await;
        let secret = match answer {
            KeyAnswer::Refused => return Ok(KeyState::Rejected(KeyStateRejected {})),
            KeyAnswer::Unready => return Ok(KeyState::Unreachable(KeyStateUnreachable {})),
            KeyAnswer::Key(key) => Some(key),
            KeyAnswer::None => None,
        };
        if kind.field() == KindField::Key && secret.is_none() {
            return Ok(none());
        }
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
            .map_err(|_| Failed)
    }

    /// Replaces pending checks.
    fn check(self: &Arc<Self>, id: &str) {
        if !self.listed(id) {
            return;
        }
        let _batch = self.batch();
        let status = self.status_of(id);
        drop(status.reader.read());
    }

    fn invalidate(&self, id: &str, unbill: bool) {
        let _batch = self.batch();
        let status = {
            let mut state = locked(&self.state);
            let status = state.statuses.remove(id);
            if let Some(status) = &status {
                status.reader.dispose();
                if status.reader.busy() {
                    state.retired.push(status.reader.clone());
                }
            }
            status
        };
        if let Some(status) = status
            && !matches!(*locked(&status.state), KeyState::None(_))
        {
            self.touch(id, Part::Key);
        }
        if unbill && locked(&self.state).bills.remove(id).is_some() {
            self.touch(id, Part::Billed);
        }
        if locked(&self.state).pools.remove(id).is_some() {
            self.touch(id, Part::Pool);
        }
        let prefix = format!("{id}\n");
        self.gate.clear(|key| key.starts_with(&prefix));
    }

    fn snapshot(&self) -> Vec<(String, Identity)> {
        self.entries()
            .iter()
            .map(|entry| (entry.id.clone(), identity_of(entry)))
            .collect()
    }

    fn diff(self: &Arc<Self>) {
        let _batch = self.batch();
        let next = self.snapshot();
        let before = locked(&self.state).identities.clone();
        for (id, was) in &before {
            match next.iter().find(|(held, _)| held == id) {
                None => self.invalidate(id, true),
                Some((_, now)) if now.plan != was.plan || now.url != was.url => {
                    self.invalidate(id, false);
                    self.check(id);
                }
                Some((_, now)) if now.extra_usage != was.extra_usage => {
                    let last_bill = locked(&self.state)
                        .bills
                        .get(id)
                        .map(|held| held.billing.clone());
                    self.apply(
                        id,
                        PoolEvent::ExtraUsage(PoolEventExtraUsage {
                            on: now.extra_usage,
                            last_bill,
                        }),
                    );
                }
                Some(_) => {}
            }
        }
        let added: Vec<String> = next
            .iter()
            .filter(|(id, _)| !before.iter().any(|(held, _)| held == id))
            .map(|(id, _)| id.clone())
            .collect();
        locked(&self.state).identities = next;
        for id in added {
            self.check(&id);
        }
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
            let Some(entry) = core.entry_of(&id) else {
                return KeyAnswer::None;
            };
            if let Some(standing) = core.standing(&entry) {
                return standing;
            }
            core.keys.get(&id).await
        })));
        Box::pin(async move { work.await.unwrap_or(KeyAnswer::Unready) })
    }

    fn on_change(&self, listener: Box<dyn Fn(&str) + Send + Sync>) -> Unsubscribe {
        self.core.keys.on_change(listener)
    }
}

impl ProviderHealth {
    /// Defers initial key checks. Asks `ids` once, for the gate's name.
    pub fn new(deps: HealthDeps, ids: &dyn Ids) -> Self {
        let gate = ProviderGate::new(
            GateDeps {
                scheduler: Arc::clone(&deps.scheduler),
                pause_ms: deps.pause_ms,
                max_pauses: deps.max_pauses,
            },
            ids,
        );
        let core = Arc::new(Core {
            scheduler: deps.scheduler,
            list: deps.list,
            keys: deps.keys,
            kinds: deps.kinds,
            kind_deps: deps.kind_deps,
            pause_ms: deps.pause_ms.unwrap_or(PAUSE_MS),
            gate,
            state: Mutex::new(State::default()),
            listeners: Mutex::new(Vec::new()),
            next_listener: Mutex::new(0),
            unsubscribe: Mutex::new(Vec::new()),
        });
        locked(&core.state).identities = core.snapshot();
        let keys_off = {
            let weak = Arc::downgrade(&core);
            core.keys.on_change(Box::new(move |id| {
                if let Some(core) = weak.upgrade() {
                    let _batch = core.batch();
                    core.invalidate(id, true);
                    core.check(id);
                }
            }))
        };
        let list_off = {
            let weak = Arc::downgrade(&core);
            core.list.on_change(Box::new(move || {
                if let Some(core) = weak.upgrade() {
                    core.diff();
                }
            }))
        };
        let gate_off = {
            let weak = Arc::downgrade(&core);
            core.gate.on_change(Box::new(move |model| {
                if let Some(core) = weak.upgrade() {
                    core.touch(entry_of_model(model), Part::Waiting);
                }
            }))
        };
        locked(&core.unsubscribe).extend([keys_off, list_off, gate_off]);
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
        self.core.status(id)
    }

    /// Listed replies create bills.
    pub fn bill(&self, id: &str) -> Option<Billed> {
        locked(&self.core.state).bills.get(id).cloned()
    }

    /// Everything health tracks of one entry.
    pub fn state(&self, id: &str) -> EntryHealth {
        let prefix = format!("{id}\n");
        let pool = locked(&self.core.state)
            .pools
            .get(id)
            .cloned()
            .unwrap_or_else(clear);
        EntryHealth {
            entry: id.to_owned(),
            key: self.status(id),
            billed: self.bill(id),
            pool,
            waiting: self
                .core
                .gate
                .holding()
                .iter()
                .filter_map(|model| model.strip_prefix(&prefix))
                .map(str::to_owned)
                .collect(),
        }
    }

    /// One model's gate as data; none until it is first used.
    pub fn gate_state(&self, route: RouteRef<'_>) -> Option<GateState> {
        self.core.gate.state_of(&key_of(route))
    }

    /// Accumulates reply billing.
    pub fn billed(&self, id: &str, reply: &Billing) {
        if !self.core.listed(id) {
            return;
        }
        {
            let mut state = locked(&self.core.state);
            let next = billed_with(state.bills.get(id), reply);
            if state.bills.get(id) == Some(&next) {
                return;
            }
            state.bills.insert(id.to_owned(), next);
        }
        self.core.touch(id, Part::Billed);
    }

    /// Replaces pending checks.
    pub fn check(&self, id: &str) {
        self.core.check(id);
    }

    pub fn check_unread(&self) {
        for entry in self.core.entries() {
            let unread = !locked(&self.core.state).statuses.contains_key(&entry.id);
            if unread {
                self.core.check(&entry.id);
            }
        }
    }

    /// Answers when no key check is running, having waited for those that were.
    pub async fn idle(&self) {
        loop {
            let busy: Vec<KeyReader> = {
                let state = locked(&self.core.state);
                state
                    .statuses
                    .values()
                    .map(|status| status.reader.clone())
                    .chain(state.retired.iter().cloned())
                    .filter(KeyReader::busy)
                    .collect()
            };
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
        let offs = std::mem::take(&mut *locked(&self.core.unsubscribe));
        for off in offs {
            off();
        }
        let statuses: Vec<Arc<Status>> = {
            let mut state = locked(&self.core.state);
            let held = state.statuses.drain().map(|(_, status)| status).collect();
            state.bills.clear();
            state.pools.clear();
            state.probes.clear();
            held
        };
        for status in statuses {
            status.reader.dispose();
        }
        locked(&self.core.listeners).clear();
        self.core.gate.dispose();
    }
}

impl Gate for ProviderHealth {
    /// Probes require admitted tickets.
    fn admit(
        &self,
        route: RouteRef<'_>,
        cancel: &CancelToken,
        on_wait: Option<WaitFn>,
        again: bool,
    ) -> BoxFuture<'static, Admission> {
        let core = Arc::clone(&self.core);
        let id = route.entry.id.clone();
        let admission = {
            let _batch = core.batch();
            core.gate.admit(&key_of(route), cancel, on_wait, again)
        };
        Box::pin(async move {
            let admission = admission.await;
            if let Admission::Go(held) = &admission {
                core.admitted(&id, held.ticket.clone());
            }
            admission
        })
    }

    fn try_admit(&self, route: RouteRef<'_>) -> Option<Ticket> {
        let _batch = self.core.batch();
        let ticket = self.core.gate.try_admit(&key_of(route));
        if let Some(ticket) = &ticket {
            self.core.admitted(&route.entry.id, ticket.clone());
        }
        ticket
    }

    fn settle(
        &self,
        ticket: &Ticket,
        outcome: GateOutcome,
        retry_after_ms: Option<i64>,
    ) -> GateSettled {
        let _batch = self.core.batch();
        let id = locked(&self.core.state).probes.remove(&ticket_key(ticket));
        if let Some(id) = id {
            self.core.apply(
                &id,
                PoolEvent::Settled(PoolEventSettled {
                    ticket: ticket.clone(),
                }),
            );
        }
        self.core.gate.settle(ticket, outcome, retry_after_ms)
    }

    fn deferred(&self, route: RouteRef<'_>, keep: Option<&HashSet<String>>) -> Option<SkipReason> {
        if self.core.rejected(route.entry) {
            return Some(SkipReason::KeyRejected);
        }
        if let Some(pool) = self.core.verdict(route.entry, keep) {
            return Some(pool.skip);
        }
        if self.core.gate.ready(&key_of(route)) {
            None
        } else {
            Some(SkipReason::Paused)
        }
    }

    fn blocked(&self, route: RouteRef<'_>, keep: Option<&HashSet<String>>) -> Option<Blocked> {
        if self.core.rejected(route.entry) {
            return Some(Blocked::KeyRejected(BlockedKeyRejected {}));
        }
        self.core.verdict(route.entry, keep)?.blocked
    }

    /// Balance replies extend `run`.
    fn served(
        &self,
        route: RouteRef<'_>,
        usage: Option<&NormalizedUsage>,
        run: Option<&mut HashSet<String>>,
    ) {
        let Some(source) = usage.and_then(|held| held.billing_source.clone()) else {
            return;
        };
        // Settings may change mid-request.
        let entry = self
            .core
            .entry_of(&route.entry.id)
            .unwrap_or_else(|| route.entry.clone());
        self.core.apply(
            &entry.id,
            PoolEvent::Billed(PoolEventBilled {
                source: source.clone(),
                plan: entry.plan.map(|plan| plan.as_str().to_owned()),
                extra_usage: entry.extra_usage == Some(true),
            }),
        );
        if source == "balance"
            && let Some(run) = run
        {
            run.insert(entry.id);
        }
    }

    fn failed(&self, route: RouteRef<'_>, failure: &Failure, onward: bool) {
        let core = &self.core;
        let entry = route.entry;
        let _batch = core.batch();
        match failure.category {
            FailureCategory::KeyRejected => {
                core.mark(&entry.id, KeyState::Rejected(KeyStateRejected {}));
            }
            // One pool per entry.
            FailureCategory::Allowance | FailureCategory::OutOfCredit => core.apply(
                &entry.id,
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
                    core.mark(&entry.id, KeyState::Unreachable(KeyStateUnreachable {}));
                } else if onward {
                    core.gate.cool(&key_of(route), failure.retry_after_ms);
                }
            }
            // Settlement owns overload pauses.
            FailureCategory::Overloaded
            | FailureCategory::KeyMissing
            | FailureCategory::Refused
            | FailureCategory::BadRequest
            | FailureCategory::ReplayRejected => {}
            _ => {
                if onward {
                    core.gate.cool(&key_of(route), failure.retry_after_ms);
                }
            }
        }
    }
}
