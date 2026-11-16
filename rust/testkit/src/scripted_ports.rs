//! Scripted ports: values set by the test.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use nyaaai_contract::cancel::Unsubscribe;
use nyaaai_contract::entry_keys::{EntryKeys, KeyAnswer};
use nyaaai_contract::future::BoxFuture;
use nyaaai_contract::json::Record;
use nyaaai_contract::model_registry::ModelRegistry;
use nyaaai_contract::provider_list::ProviderList;
use nyaaai_contract::shapes::model::{ModelRow, StoredRegistry, TestRecord};
use nyaaai_contract::shapes::provider::ProviderEntry;

fn locked<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

type Held<F> = Arc<Mutex<Vec<(u64, Arc<F>)>>>;

/// Calls in subscription order; an unsubscribed listener is not called.
struct Listeners<F: ?Sized> {
    next: AtomicU64,
    held: Held<F>,
}

impl<F: ?Sized + Send + Sync + 'static> Listeners<F> {
    fn new() -> Self {
        Self {
            next: AtomicU64::new(0),
            held: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn add(&self, listener: Box<F>) -> Unsubscribe {
        let id = self.next.fetch_add(1, Ordering::Relaxed);
        locked(&self.held).push((id, Arc::from(listener)));
        let held = Arc::clone(&self.held);
        Box::new(move || locked(&held).retain(|(each, _)| *each != id))
    }

    fn tell(&self, call: impl Fn(&F)) {
        let snapshot: Vec<(u64, Arc<F>)> = locked(&self.held).clone();
        for (id, listener) in snapshot {
            if locked(&self.held).iter().any(|(each, _)| *each == id) {
                call(&listener);
            }
        }
    }
}

type KeyListener = dyn Fn(&str) + Send + Sync;
type Listener = dyn Fn() + Send + Sync;

pub struct ScriptedEntryKeys {
    answers: Mutex<HashMap<String, KeyAnswer>>,
    listeners: Listeners<KeyListener>,
    reads: AtomicUsize,
}

impl ScriptedEntryKeys {
    pub fn new() -> Self {
        Self {
            answers: Mutex::new(HashMap::new()),
            listeners: Listeners::new(),
            reads: AtomicUsize::new(0),
        }
    }

    /// Tells every listener, in subscription order.
    pub fn set(&self, id: &str, answer: KeyAnswer) {
        locked(&self.answers).insert(id.to_owned(), answer);
        self.listeners.tell(|listener| listener(id));
    }

    /// Times `get` was called.
    pub fn reads(&self) -> usize {
        self.reads.load(Ordering::Relaxed)
    }
}

impl Default for ScriptedEntryKeys {
    fn default() -> Self {
        Self::new()
    }
}

impl EntryKeys for ScriptedEntryKeys {
    fn get(&self, id: &str) -> BoxFuture<'_, KeyAnswer> {
        self.reads.fetch_add(1, Ordering::Relaxed);
        let answer = locked(&self.answers)
            .get(id)
            .cloned()
            .unwrap_or(KeyAnswer::None);
        Box::pin(async move { answer })
    }

    fn on_change(&self, listener: Box<KeyListener>) -> Unsubscribe {
        self.listeners.add(listener)
    }
}

pub struct ScriptedProviderList {
    entries: Mutex<Vec<ProviderEntry>>,
    listeners: Listeners<Listener>,
    reads: AtomicUsize,
}

impl ScriptedProviderList {
    pub fn new(entries: Vec<ProviderEntry>) -> Self {
        Self {
            entries: Mutex::new(entries),
            listeners: Listeners::new(),
            reads: AtomicUsize::new(0),
        }
    }

    pub fn set(&self, entries: Vec<ProviderEntry>) {
        *locked(&self.entries) = entries;
        self.listeners.tell(|listener| listener());
    }

    /// Times `entries` was called.
    pub fn reads(&self) -> usize {
        self.reads.load(Ordering::Relaxed)
    }
}

impl ProviderList for ScriptedProviderList {
    fn entries(&self) -> Vec<ProviderEntry> {
        self.reads.fetch_add(1, Ordering::Relaxed);
        locked(&self.entries).clone()
    }

    fn on_change(&self, listener: Box<Listener>) -> Unsubscribe {
        self.listeners.add(listener)
    }
}

struct RegistryState {
    rows: Vec<ModelRow>,
    stored: StoredRegistry,
    refused: Option<String>,
    written: Vec<(String, String, TestRecord)>,
}

pub struct ScriptedModelRegistry {
    state: Mutex<RegistryState>,
    listeners: Listeners<Listener>,
    reads: AtomicUsize,
}

impl ScriptedModelRegistry {
    pub fn new(rows: Vec<ModelRow>, stored: StoredRegistry) -> Self {
        Self {
            state: Mutex::new(RegistryState {
                rows,
                stored,
                refused: None,
                written: Vec::new(),
            }),
            listeners: Listeners::new(),
            reads: AtomicUsize::new(0),
        }
    }

    /// No rows; nothing stored.
    pub fn empty() -> Self {
        Self::new(
            Vec::new(),
            StoredRegistry {
                tested: Record::new(),
                added: Record::new(),
                hidden: Vec::new(),
            },
        )
    }

    pub fn set_rows(&self, rows: Vec<ModelRow>) {
        locked(&self.state).rows = rows;
        self.listeners.tell(|listener| listener());
    }

    pub fn set_stored(&self, stored: StoredRegistry) {
        locked(&self.state).stored = stored;
        self.listeners.tell(|listener| listener());
    }

    /// Makes `record` answer this text and write nothing; `None` writes again.
    pub fn refuse(&self, why: Option<String>) {
        locked(&self.state).refused = why;
    }

    /// Times `rows` or `stored` was called.
    pub fn reads(&self) -> usize {
        self.reads.load(Ordering::Relaxed)
    }

    /// Records written, in order: entry, id, record.
    pub fn written(&self) -> Vec<(String, String, TestRecord)> {
        locked(&self.state).written.clone()
    }
}

impl ModelRegistry for ScriptedModelRegistry {
    fn rows(&self) -> Vec<ModelRow> {
        self.reads.fetch_add(1, Ordering::Relaxed);
        locked(&self.state).rows.clone()
    }

    fn stored(&self) -> StoredRegistry {
        self.reads.fetch_add(1, Ordering::Relaxed);
        locked(&self.state).stored.clone()
    }

    fn record(&self, entry: &str, id: &str, record: TestRecord) -> BoxFuture<'_, Option<String>> {
        let (entry, id) = (entry.to_owned(), id.to_owned());
        Box::pin(async move {
            {
                let mut state = locked(&self.state);
                if let Some(why) = &state.refused {
                    return Some(why.clone());
                }
                let mut held = state.stored.tested.get(&entry).cloned().unwrap_or_default();
                held.insert(id.clone(), record.clone());
                state.stored.tested.insert(entry.clone(), held);
                state.written.push((entry, id, record));
            }
            self.listeners.tell(|listener| listener());
            None
        })
    }

    fn on_change(&self, listener: Box<Listener>) -> Unsubscribe {
        self.listeners.add(listener)
    }
}
