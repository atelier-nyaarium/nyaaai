//! One incarnation per listed entry id, and the key eras under it.

use std::collections::{HashMap, HashSet, VecDeque};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, Weak};

use nyaaai_contract::cancel::Unsubscribe;
use nyaaai_contract::entry_keys::EntryKeys;
use nyaaai_contract::json::{Value, canonical_json};
use nyaaai_contract::provider_list::ProviderList;
use nyaaai_contract::shapes::health::PoolState;
use nyaaai_contract::shapes::provider::{Billed, KeyState, ProviderEntry};

use crate::health::gate::GateBook;
use crate::health::key_reader::KeyReader;
use crate::health::pool::clear;

/// A ticket's data, without its gate-independent parts.
pub type TicketKey = (String, String, u64, u64, bool, u64);

/// A key state and its reader.
pub struct KeyStatus {
    pub state: Arc<Mutex<KeyState>>,
    pub reader: KeyReader,
}

/// A change, told once the registry holds it.
#[derive(Clone)]
pub enum RegistryEvent {
    Added(Arc<Incarnation>),
    Retired(Arc<Incarnation>),
    ExtraUsage {
        incarnation: Arc<Incarnation>,
        on: bool,
    },
    Rekeyed {
        incarnation: Arc<Incarnation>,
        retired: Arc<Era>,
    },
}

pub type Watcher = Arc<dyn Fn(&[RegistryEvent]) + Send + Sync>;

type KeyListener = Arc<dyn Fn(&str) + Send + Sync>;

/// What an entry is: its kind, plan and URL.
pub fn identity_of(entry: &ProviderEntry) -> String {
    canonical_json(&Value::Array(vec![
        Value::String(entry.kind.as_str().to_owned()),
        entry
            .plan
            .map_or(Value::Null, |plan| Value::String(plan.as_str().to_owned())),
        entry.url.clone().map_or(Value::Null, Value::String),
    ]))
}

fn locked<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// What an era holds; health writes it, always under this lock.
pub struct EraState {
    /// Set by the registry; a report meeting it is dropped.
    pub retired: bool,
    pub status: Option<Arc<KeyStatus>>,
    pub pool: PoolState,
    pub bill: Option<Billed>,
    pub rechecked: Option<i64>,
    /// Admitted tickets, by data.
    pub probes: HashSet<TicketKey>,
}

/// One key generation. The registry holds it while live; its incarnation is a weak back reference.
pub struct Era {
    pub number: u64,
    /// The entry id, the name health tells changes under.
    pub id: String,
    pub gates: Arc<GateBook>,
    incarnation: Weak<Incarnation>,
    state: Mutex<EraState>,
}

impl Era {
    /// The one lock of everything the era holds; held across no call out.
    pub fn state(&self) -> MutexGuard<'_, EraState> {
        locked(&self.state)
    }

    /// None once nothing holds the incarnation any more.
    pub fn incarnation(&self) -> Option<Arc<Incarnation>> {
        self.incarnation.upgrade()
    }

    pub fn retired(&self) -> bool {
        locked(&self.state).retired
    }

    fn new(number: u64, incarnation: &Arc<Incarnation>) -> Arc<Self> {
        Arc::new(Self {
            number,
            id: incarnation.id.clone(),
            gates: Arc::new(GateBook::new(number, incarnation.id.clone())),
            incarnation: Arc::downgrade(incarnation),
            state: Mutex::new(EraState {
                retired: false,
                status: None,
                pool: clear(),
                bill: None,
                rechecked: None,
                probes: HashSet::new(),
            }),
        })
    }
}

/// One identity under one id; retired for good.
pub struct Incarnation {
    pub number: u64,
    pub id: String,
    pub identity: String,
    live: AtomicBool,
    /// Replaced by the registry on a list change.
    entry: Mutex<ProviderEntry>,
    /// Replaced by the registry on a key change.
    era: Mutex<Option<Arc<Era>>>,
    /// The registry that made it; weak, so a route held past the registry makes no cycle.
    registry: Weak<Core>,
}

impl Incarnation {
    pub fn live(&self) -> bool {
        self.live.load(Ordering::SeqCst)
    }

    /// The row as the list holds it.
    pub fn entry(&self) -> ProviderEntry {
        locked(&self.entry).clone()
    }

    pub fn era(&self) -> Arc<Era> {
        locked(&self.era)
            .as_ref()
            .map(Arc::clone)
            .expect("an incarnation always has an era")
    }

    /// Whether this registry made it.
    pub fn belongs_to(&self, registry: &EntryRegistry) -> bool {
        self.registry
            .upgrade()
            .is_some_and(|core| Arc::ptr_eq(&core, &registry.core))
    }

    /// An ended incarnation of the registry: what a held route of a removed entry stands on.
    pub fn ended(registry: &EntryRegistry, entry: ProviderEntry) -> Arc<Self> {
        let incarnation = Self::new(0, entry, 0, Arc::downgrade(&registry.core));
        incarnation.live.store(false, Ordering::SeqCst);
        incarnation.era().state().retired = true;
        incarnation
    }

    fn new(number: u64, entry: ProviderEntry, first_era: u64, registry: Weak<Core>) -> Arc<Self> {
        let incarnation = Arc::new(Self {
            number,
            id: entry.id.clone(),
            identity: identity_of(&entry),
            live: AtomicBool::new(true),
            entry: Mutex::new(entry),
            era: Mutex::new(None),
            registry,
        });
        *locked(&incarnation.era) = Some(Era::new(first_era, &incarnation));
        incarnation
    }
}

/// What the registry tells next.
enum Told {
    Events(Vec<RegistryEvent>),
    Key(String),
    /// The last tell after a dispose: nothing is told after it.
    Clear,
}

#[derive(Default)]
struct State {
    /// Live, in list order.
    order: Vec<Arc<Incarnation>>,
    /// Live eras, by number.
    eras: HashMap<u64, Arc<Era>>,
    incarnations: u64,
    era_count: u64,
    /// The newest list read applied.
    applied: u64,
}

struct Core {
    me: Weak<Core>,
    list: Arc<dyn ProviderList>,
    /// Guards the live set and the counters; nothing runs under it but pushes to `outbox`.
    state: Mutex<State>,
    /// Tells made and not yet told, in the order made.
    outbox: Mutex<VecDeque<Told>>,
    delivering: AtomicBool,
    /// Guards the watcher list; copied before any call.
    watchers: Mutex<Vec<(u64, Watcher)>>,
    /// Guards the key listener list; copied before any call.
    key_listeners: Mutex<Vec<(u64, KeyListener)>>,
    next: AtomicU64,
    /// Orders the list reads.
    reads: AtomicU64,
    unsubscribe: Mutex<Vec<Unsubscribe>>,
    disposed: AtomicBool,
}

/// Sole subscriber of the list and key changes.
#[derive(Clone)]
pub struct EntryRegistry {
    core: Arc<Core>,
}

fn rows_of(list: &dyn ProviderList) -> Vec<ProviderEntry> {
    let mut held: Vec<ProviderEntry> = Vec::new();
    // A repeated id keeps its first row.
    for entry in list.entries() {
        if !held.iter().any(|each| each.id == entry.id) {
            held.push(entry);
        }
    }
    held
}

impl Core {
    fn make(&self, state: &mut State, entry: ProviderEntry) -> Arc<Incarnation> {
        state.incarnations += 1;
        state.era_count += 1;
        let incarnation =
            Incarnation::new(state.incarnations, entry, state.era_count, self.me.clone());
        let era = incarnation.era();
        state.eras.insert(era.number, era);
        incarnation
    }

    fn retire(state: &mut State, incarnation: &Arc<Incarnation>) {
        incarnation.live.store(false, Ordering::SeqCst);
        let era = incarnation.era();
        era.state().retired = true;
        state.eras.remove(&era.number);
    }

    fn push(&self, told: Told) {
        locked(&self.outbox).push_back(told);
    }

    /// Tells in the order made; a callback's change queues behind.
    fn flush(&self) {
        loop {
            if self.delivering.swap(true, Ordering::AcqRel) {
                return;
            }
            loop {
                let Some(told) = locked(&self.outbox).pop_front() else {
                    break;
                };
                match told {
                    Told::Events(events) => {
                        let held: Vec<Watcher> = locked(&self.watchers)
                            .iter()
                            .map(|(_, watcher)| Arc::clone(watcher))
                            .collect();
                        for watcher in held {
                            // A watcher cannot change what it is told.
                            let _ = catch_unwind(AssertUnwindSafe(|| watcher(&events)));
                        }
                    }
                    Told::Clear => {
                        locked(&self.watchers).clear();
                        locked(&self.key_listeners).clear();
                    }
                    Told::Key(id) => {
                        let held: Vec<KeyListener> = locked(&self.key_listeners)
                            .iter()
                            .map(|(_, listener)| Arc::clone(listener))
                            .collect();
                        for listener in held {
                            let _ = catch_unwind(AssertUnwindSafe(|| listener(&id)));
                        }
                    }
                }
            }
            self.delivering.store(false, Ordering::Release);
            if locked(&self.outbox).is_empty() {
                return;
            }
        }
    }

    fn list_changed(&self) {
        if self.disposed.load(Ordering::SeqCst) {
            return;
        }
        let read = self.reads.fetch_add(1, Ordering::SeqCst) + 1;
        let rows = rows_of(&*self.list);
        {
            let mut state = locked(&self.state);
            // A newer read was applied first.
            if read < state.applied {
                return;
            }
            state.applied = read;
            let mut events: Vec<RegistryEvent> = Vec::new();
            let mut made: HashMap<String, Arc<Incarnation>> = HashMap::new();
            for was in state.order.clone() {
                match rows.iter().find(|row| row.id == was.id) {
                    None => {
                        Self::retire(&mut state, &was);
                        events.push(RegistryEvent::Retired(Arc::clone(&was)));
                    }
                    Some(row) if identity_of(row) != was.identity => {
                        Self::retire(&mut state, &was);
                        events.push(RegistryEvent::Retired(Arc::clone(&was)));
                        let next = self.make(&mut state, row.clone());
                        made.insert(was.id.clone(), Arc::clone(&next));
                        events.push(RegistryEvent::Added(next));
                    }
                    Some(row) => {
                        let flipped = (row.extra_usage == Some(true))
                            != (was.entry().extra_usage == Some(true));
                        *locked(&was.entry) = row.clone();
                        made.insert(was.id.clone(), Arc::clone(&was));
                        if flipped {
                            events.push(RegistryEvent::ExtraUsage {
                                incarnation: Arc::clone(&was),
                                on: row.extra_usage == Some(true),
                            });
                        }
                    }
                }
            }
            let mut order = Vec::new();
            for row in rows {
                let held = match made.remove(&row.id) {
                    Some(held) => held,
                    None => {
                        let fresh = self.make(&mut state, row);
                        events.push(RegistryEvent::Added(Arc::clone(&fresh)));
                        fresh
                    }
                };
                order.push(held);
            }
            state.order = order;
            if !events.is_empty() {
                self.push(Told::Events(events));
            }
        }
        self.flush();
    }

    fn key_changed(&self, id: &str) {
        if self.disposed.load(Ordering::SeqCst) {
            return;
        }
        {
            let mut state = locked(&self.state);
            if let Some(incarnation) = state.order.iter().find(|each| each.id == id).cloned() {
                let was = incarnation.era();
                was.state().retired = true;
                state.eras.remove(&was.number);
                state.era_count += 1;
                let next = Era::new(state.era_count, &incarnation);
                state.eras.insert(next.number, Arc::clone(&next));
                *locked(&incarnation.era) = Some(next);
                self.push(Told::Events(vec![RegistryEvent::Rekeyed {
                    incarnation,
                    retired: was,
                }]));
            }
            self.push(Told::Key(id.to_owned()));
        }
        self.flush();
    }
}

impl EntryRegistry {
    pub fn new(list: Arc<dyn ProviderList>, keys: &dyn EntryKeys) -> Self {
        let core = Arc::new_cyclic(|me| Core {
            me: me.clone(),
            list,
            state: Mutex::new(State::default()),
            outbox: Mutex::new(VecDeque::new()),
            delivering: AtomicBool::new(false),
            watchers: Mutex::new(Vec::new()),
            key_listeners: Mutex::new(Vec::new()),
            next: AtomicU64::new(0),
            reads: AtomicU64::new(0),
            unsubscribe: Mutex::new(Vec::new()),
            disposed: AtomicBool::new(false),
        });
        {
            let rows = rows_of(&*core.list);
            let mut state = locked(&core.state);
            for row in rows {
                let incarnation = core.make(&mut state, row);
                state.order.push(incarnation);
            }
        }
        let weak = Arc::downgrade(&core);
        let keys_off = keys.on_change(Box::new({
            let weak = weak.clone();
            move |id| {
                if let Some(core) = weak.upgrade() {
                    core.key_changed(id);
                }
            }
        }));
        let list_off = core.list.on_change(Box::new(move || {
            if let Some(core) = weak.upgrade() {
                core.list_changed();
            }
        }));
        locked(&core.unsubscribe).extend([keys_off, list_off]);
        Self { core }
    }

    /// Live, in list order.
    pub fn incarnations(&self) -> Vec<Arc<Incarnation>> {
        locked(&self.core.state).order.clone()
    }

    pub fn incarnation(&self, id: &str) -> Option<Arc<Incarnation>> {
        locked(&self.core.state)
            .order
            .iter()
            .find(|each| each.id == id)
            .cloned()
    }

    /// A live era.
    pub fn era(&self, number: u64) -> Option<Arc<Era>> {
        locked(&self.core.state).eras.get(&number).cloned()
    }

    /// Told each change in order; the result unsubscribes.
    pub fn watch(&self, watcher: Watcher) -> Unsubscribe {
        let id = self.core.next.fetch_add(1, Ordering::SeqCst);
        locked(&self.core.watchers).push((id, watcher));
        let weak = Arc::downgrade(&self.core);
        Box::new(move || {
            if let Some(core) = weak.upgrade() {
                locked(&core.watchers).retain(|(held, _)| *held != id);
            }
        })
    }

    /// Told each key change after the registry holds it; the result unsubscribes.
    pub fn on_key_change(&self, listener: KeyListener) -> Unsubscribe {
        let id = self.core.next.fetch_add(1, Ordering::SeqCst);
        locked(&self.core.key_listeners).push((id, listener));
        let weak = Arc::downgrade(&self.core);
        Box::new(move || {
            if let Some(core) = weak.upgrade() {
                locked(&core.key_listeners).retain(|(held, _)| *held != id);
            }
        })
    }

    /// Stops listening, retires every incarnation and tells the watchers before it forgets them.
    pub fn dispose(&self) {
        if self.core.disposed.swap(true, Ordering::SeqCst) {
            return;
        }
        let offs = std::mem::take(&mut *locked(&self.core.unsubscribe));
        for off in offs {
            off();
        }
        {
            let mut state = locked(&self.core.state);
            let order = std::mem::take(&mut state.order);
            let mut events = Vec::new();
            for incarnation in order {
                Core::retire(&mut state, &incarnation);
                events.push(RegistryEvent::Retired(incarnation));
            }
            if !events.is_empty() {
                self.core.push(Told::Events(events));
            }
            self.core.push(Told::Clear);
        }
        self.core.flush();
    }
}
