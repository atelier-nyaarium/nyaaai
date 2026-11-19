//! Pauses each model while one request probes recovery.

use std::collections::VecDeque;
use std::future::Future;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, Weak};

use nyaaai_contract::cancel::{CancelToken, Unsubscribe};
use nyaaai_contract::id::Ids;
use nyaaai_contract::order::sorted_texts;
use nyaaai_contract::shapes::health::{
    Admission, AdmissionGaveUp, AdmissionGo, AdmissionRetired, AdmissionStopped, GateOutcome,
    GateSettled, GateWaiting, Ticket,
};
use nyaaai_contract::time::{Scheduled, Scheduler};
use tokio::runtime::Handle;
use tokio::sync::oneshot;

/// Told what a queued request waits for.
pub type WaitFn = Arc<dyn Fn(GateWaiting) + Send + Sync>;

type Listener = Arc<dyn Fn(&str, &str) + Send + Sync>;

/// Told an owner and a model.
pub type ChangeFn = Box<dyn Fn(&str, &str) + Send + Sync>;

/// Told a ticket as it is made.
pub type TicketFn = Arc<dyn Fn(&Ticket) + Send + Sync>;

const PAUSE_MS: i64 = 60_000;

const MAX_PAUSES: u64 = 10;

/// A server's asked wait stops here.
const MAX_PAUSE_MS: i64 = 60 * 60_000;

/// One key era's gate rows; the gate keeps them by this book.
pub struct GateBook {
    /// Stamped on its tickets.
    pub era: u64,
    /// Told with each readiness change.
    pub owner: String,
    retired: AtomicBool,
}

impl GateBook {
    pub fn new(era: u64, owner: String) -> Self {
        Self {
            era,
            owner,
            retired: AtomicBool::new(false),
        }
    }

    pub fn is_retired(&self) -> bool {
        self.retired.load(Ordering::SeqCst)
    }
}

/// Where a model's gate stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Open,
    Paused,
    Probing,
}

/// A model's gate as data.
#[derive(Debug, Clone, PartialEq)]
pub struct GateState {
    pub phase: Phase,
    /// Bumped by every reopening.
    pub generation: u64,
    pub pauses: u64,
    /// The end of the last pause; 0 before any.
    pub resume_at: f64,
    pub probe: Option<Ticket>,
    /// Requests waiting for a turn.
    pub queued: usize,
}

pub struct GateDeps {
    /// Where a cancelled waiter's release runs, from any thread.
    pub runtime: Handle,
    /// Pauses are scheduled on it and read from it.
    pub scheduler: Arc<dyn Scheduler>,
    pub pause_ms: Option<i64>,
    pub max_pauses: Option<u64>,
    /// Told each ticket as it is made, before its holder has it.
    pub on_ticket: Option<TicketFn>,
    /// Told a ticket whose holder was gone before it arrived.
    pub on_withdrawn: Option<TicketFn>,
}

struct Waiter {
    send: oneshot::Sender<Admission>,
    on_wait: Option<WaitFn>,
    cancel: CancelToken,
    /// Dropped with the waiter; ends its cancel watcher.
    _held: oneshot::Sender<()>,
}

struct Model {
    book: Arc<GateBook>,
    name: String,
    phase: Phase,
    generation: u64,
    pauses: u64,
    resume_at: f64,
    probe: Option<Ticket>,
    queue: VecDeque<Waiter>,
    timer: Option<Scheduled>,
    /// Names the timer that may still fire.
    epoch: u64,
}

struct State {
    models: Vec<Model>,
    serial: u64,
    epoch: u64,
    disposed: bool,
}

/// What an operation did, done in order once no lock is held.
enum Effect {
    /// Answers a waiter; a ticket in it is told before the waiter has it.
    Hand(oneshot::Sender<Admission>, Admission, Arc<GateBook>),
    /// A ticket an admission returns at once.
    Made(Ticket),
    Wait(WaitFn, GateWaiting),
    Tell {
        owner: String,
        model: String,
    },
    Cancel(Scheduled),
    Schedule {
        book: Arc<GateBook>,
        model: String,
        epoch: u64,
        ms: i64,
    },
}

struct Core {
    runtime: Handle,
    scheduler: Arc<dyn Scheduler>,
    id: String,
    pause_ms: i64,
    max_pauses: u64,
    on_ticket: Option<TicketFn>,
    on_withdrawn: Option<TicketFn>,
    state: Mutex<State>,
    /// Effects made and not yet done, oldest first.
    outbox: Mutex<VecDeque<Effect>>,
    flushing: AtomicBool,
    listeners: Mutex<Vec<(u64, Listener)>>,
    next_listener: Mutex<u64>,
}

fn locked<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Open, or probing with no probe out.
fn ready_of(model: &Model) -> bool {
    model.phase == Phase::Open || (model.phase == Phase::Probing && model.probe.is_none())
}

fn stopped() -> Admission {
    Admission::Stopped(AdmissionStopped {})
}

fn index_of(state: &mut State, book: &Arc<GateBook>, name: &str) -> usize {
    if let Some(at) = find(state, book, name) {
        return at;
    }
    state.models.push(Model {
        book: Arc::clone(book),
        name: name.to_owned(),
        phase: Phase::Open,
        generation: 0,
        pauses: 0,
        resume_at: 0.0,
        probe: None,
        queue: VecDeque::new(),
        timer: None,
        epoch: 0,
    });
    state.models.len() - 1
}

fn find(state: &State, book: &Arc<GateBook>, name: &str) -> Option<usize> {
    state
        .models
        .iter()
        .position(|model| model.book.era == book.era && model.name == name)
}

/// What the gate does to one model, writing its effects as it goes.
struct Machine<'a> {
    core: &'a Core,
    state: &'a mut State,
    effects: &'a mut Vec<Effect>,
    now: i64,
}

enum Release {
    Go,
    GaveUp,
}

impl Machine<'_> {
    fn go(&mut self, at: usize, probe: bool) -> Admission {
        self.state.serial += 1;
        let model = &mut self.state.models[at];
        let ticket = Ticket {
            gate: self.core.id.clone(),
            model: model.name.clone(),
            era: model.book.era,
            generation: model.generation,
            probe,
            serial: self.state.serial,
        };
        if probe {
            model.probe = Some(ticket.clone());
        }
        Admission::Go(AdmissionGo { ticket })
    }

    /// A ticket the caller of this call gets at once.
    fn go_now(&mut self, at: usize, probe: bool) -> Admission {
        let admission = self.go(at, probe);
        if let Admission::Go(held) = &admission {
            self.effects.push(Effect::Made(held.ticket.clone()));
        }
        admission
    }

    fn wait_of(&self, at: usize) -> GateWaiting {
        let model = &self.state.models[at];
        GateWaiting {
            model: model.name.clone(),
            resume_at: (model.phase == Phase::Paused).then_some(model.resume_at),
            pause: model.pauses,
            of: self.core.max_pauses,
        }
    }

    fn notify(&mut self, at: usize) {
        let waiting = self.wait_of(at);
        for waiter in &self.state.models[at].queue {
            if let Some(on_wait) = &waiter.on_wait {
                self.effects
                    .push(Effect::Wait(Arc::clone(on_wait), waiting.clone()));
            }
        }
    }

    fn reopen(&mut self, at: usize, release: Release) {
        let model = &mut self.state.models[at];
        model.phase = Phase::Open;
        model.generation += 1;
        model.pauses = 0;
        let book = Arc::clone(&model.book);
        let queue = std::mem::take(&mut model.queue);
        for waiter in queue {
            if waiter.cancel.is_cancelled() {
                self.effects
                    .push(Effect::Hand(waiter.send, stopped(), Arc::clone(&book)));
                continue;
            }
            if waiter.send.is_closed() {
                continue;
            }
            let admission = match release {
                Release::Go => self.go(at, false),
                Release::GaveUp => Admission::GaveUp(AdmissionGaveUp {}),
            };
            self.effects
                .push(Effect::Hand(waiter.send, admission, Arc::clone(&book)));
        }
    }

    fn pause(&mut self, at: usize, retry_after_ms: Option<i64>) {
        let wait = self
            .core
            .pause_ms
            .max(retry_after_ms.unwrap_or(0).min(MAX_PAUSE_MS));
        self.state.epoch += 1;
        let epoch = self.state.epoch;
        let model = &mut self.state.models[at];
        model.phase = Phase::Paused;
        model.resume_at = self.now.saturating_add(wait) as f64;
        if let Some(old) = model.timer.take() {
            self.effects.push(Effect::Cancel(old));
        }
        model.epoch = epoch;
        self.effects.push(Effect::Schedule {
            book: Arc::clone(&model.book),
            model: model.name.clone(),
            epoch,
            ms: wait,
        });
        self.notify(at);
    }

    /// Oldest waiter probes first.
    fn next_probe(&mut self, at: usize) {
        while let Some(waiter) = self.state.models[at].queue.pop_front() {
            // A waiter is looked at again as it is chosen: one gone is passed over.
            let book = Arc::clone(&self.state.models[at].book);
            if waiter.cancel.is_cancelled() {
                self.effects
                    .push(Effect::Hand(waiter.send, stopped(), book));
                continue;
            }
            if waiter.send.is_closed() {
                continue;
            }
            let admission = self.go(at, true);
            self.effects
                .push(Effect::Hand(waiter.send, admission, book));
            break;
        }
        self.notify(at);
    }

    fn settle_probe(
        &mut self,
        at: usize,
        outcome: GateOutcome,
        retry_after_ms: Option<i64>,
    ) -> GateSettled {
        self.state.models[at].probe = None;
        match outcome {
            GateOutcome::Ok => {
                self.reopen(at, Release::Go);
                GateSettled::Done
            }
            GateOutcome::Overloaded => {
                if self.state.models[at].pauses >= self.core.max_pauses {
                    self.reopen(at, Release::GaveUp);
                    return GateSettled::GaveUp;
                }
                self.state.models[at].pauses += 1;
                self.pause(at, retry_after_ms);
                GateSettled::Again
            }
            GateOutcome::Failed | GateOutcome::Cancelled => {
                self.next_probe(at);
                GateSettled::Done
            }
        }
    }
}

/// Removes the waiters whose caller cancelled, as stopped, and those whose caller is gone.
fn sweep(state: &mut State, effects: &mut Vec<Effect>) {
    for model in &mut state.models {
        let gone = |waiter: &Waiter| waiter.cancel.is_cancelled() || waiter.send.is_closed();
        if !model.queue.iter().any(gone) {
            continue;
        }
        let queue = std::mem::take(&mut model.queue);
        for waiter in queue {
            if waiter.cancel.is_cancelled() {
                effects.push(Effect::Hand(
                    waiter.send,
                    stopped(),
                    Arc::clone(&model.book),
                ));
            } else if waiter.send.is_closed() {
                continue;
            } else {
                model.queue.push_back(waiter);
            }
        }
    }
}

impl Core {
    /// Runs an operation, then its effects in order; effect calls queue behind.
    fn act<T>(self: &Arc<Self>, run: impl FnOnce(&mut Machine<'_>) -> T) -> T {
        let mut effects = Vec::new();
        let result = {
            let mut state = locked(&self.state);
            let now = self.scheduler.now_ms();
            sweep(&mut state, &mut effects);
            run(&mut Machine {
                core: self,
                state: &mut state,
                effects: &mut effects,
                now,
            })
        };
        locked(&self.outbox).extend(effects);
        self.flush();
        result
    }

    /// Does the effects owed, one flusher at a time.
    fn flush(self: &Arc<Self>) {
        loop {
            if self.flushing.swap(true, Ordering::AcqRel) {
                return;
            }
            loop {
                let Some(effect) = locked(&self.outbox).pop_front() else {
                    break;
                };
                self.perform(effect);
            }
            self.flushing.store(false, Ordering::Release);
            if locked(&self.outbox).is_empty() {
                return;
            }
        }
    }

    /// A change of one model, told when it turned ready or not ready.
    fn shifting<T>(
        machine: &mut Machine<'_>,
        book: &Arc<GateBook>,
        name: &str,
        run: impl FnOnce(&mut Machine<'_>, usize) -> T,
    ) -> T {
        let at = index_of(machine.state, book, name);
        let was = ready_of(&machine.state.models[at]);
        let result = run(machine, at);
        if at < machine.state.models.len() && ready_of(&machine.state.models[at]) != was {
            machine.effects.push(Effect::Tell {
                owner: book.owner.clone(),
                model: name.to_owned(),
            });
        }
        result
    }

    fn perform(self: &Arc<Self>, effect: Effect) {
        match effect {
            Effect::Hand(send, admission, book) => {
                let ticket = match &admission {
                    Admission::Go(held) => Some(held.ticket.clone()),
                    _ => None,
                };
                // The ticket is told before its holder can use it.
                if let Some(ticket) = &ticket {
                    self.made(ticket);
                }
                // A gone caller passes the turn on as if it had cancelled.
                if send.send(admission).is_err()
                    && let Some(ticket) = &ticket
                {
                    if let Some(told) = &self.on_withdrawn {
                        told(ticket);
                    }
                    self.settle(&book, ticket, GateOutcome::Cancelled, None);
                }
            }
            Effect::Made(ticket) => self.made(&ticket),
            Effect::Wait(on_wait, waiting) => {
                // A callback cannot change the gate.
                let _ = catch_unwind(AssertUnwindSafe(|| on_wait(waiting)));
            }
            Effect::Tell { owner, model } => self.tell(&owner, &model),
            Effect::Cancel(timer) => timer.cancel(),
            Effect::Schedule {
                book,
                model,
                epoch,
                ms,
            } => {
                let weak = Arc::downgrade(self);
                let (name, owned) = (model.clone(), Arc::clone(&book));
                let timer = self.scheduler.schedule(
                    ms,
                    Box::new(move || {
                        if let Some(core) = weak.upgrade() {
                            core.fire(&owned, &name, epoch);
                        }
                    }),
                );
                let mut state = locked(&self.state);
                match state.models.iter_mut().find(|held| {
                    held.book.era == book.era && held.name == model && held.epoch == epoch
                }) {
                    Some(held) if held.timer.is_none() => held.timer = Some(timer),
                    _ => {
                        drop(state);
                        timer.cancel();
                    }
                }
            }
        }
    }

    fn made(&self, ticket: &Ticket) {
        if let Some(told) = &self.on_ticket {
            told(ticket);
        }
    }

    /// Settles a ticket of this gate and this book's era; another's does nothing.
    fn settle(
        self: &Arc<Self>,
        book: &Arc<GateBook>,
        ticket: &Ticket,
        outcome: GateOutcome,
        retry_after_ms: Option<i64>,
    ) -> GateSettled {
        if ticket.gate != self.id || ticket.era != book.era || book.is_retired() {
            return GateSettled::Done;
        }
        self.act(|machine| {
            if machine.state.disposed {
                return GateSettled::Done;
            }
            if find(machine.state, book, &ticket.model).is_none() {
                return GateSettled::Done;
            }
            Self::shifting(machine, book, &ticket.model, |machine, at| {
                let (is_probe, phase, generation) = {
                    let model = &machine.state.models[at];
                    (
                        model.probe.as_ref().is_some_and(|probe| probe == ticket),
                        model.phase,
                        model.generation,
                    )
                };
                if is_probe {
                    return machine.settle_probe(at, outcome, retry_after_ms);
                }
                if outcome != GateOutcome::Overloaded {
                    if outcome == GateOutcome::Ok {
                        machine.state.models[at].pauses = 0;
                    }
                    return GateSettled::Done;
                }
                if phase != Phase::Open {
                    return GateSettled::Again;
                }
                // Ignore stale probe refusals.
                if ticket.generation < generation {
                    return GateSettled::Again;
                }
                machine.state.models[at].pauses = 1;
                machine.pause(at, retry_after_ms);
                GateSettled::Again
            })
        })
    }

    /// A pause ended.
    fn fire(self: &Arc<Self>, book: &Arc<GateBook>, name: &str, epoch: u64) {
        self.act(|machine| {
            let current =
                machine.state.models.iter().any(|held| {
                    held.book.era == book.era && held.name == name && held.epoch == epoch
                });
            if !current || book.is_retired() {
                return;
            }
            Self::shifting(machine, book, name, |machine, at| {
                let model = &mut machine.state.models[at];
                model.timer = None;
                model.phase = Phase::Probing;
                machine.next_probe(at);
            });
        });
    }

    fn tell(&self, owner: &str, model: &str) {
        let held: Vec<Listener> = locked(&self.listeners)
            .iter()
            .map(|(_, listener)| Arc::clone(listener))
            .collect();
        for listener in held {
            // A listener cannot change the gate.
            let _ = catch_unwind(AssertUnwindSafe(|| listener(owner, model)));
        }
    }
}

/// Pauses each model while one request probes recovery.
pub struct ProviderGate {
    core: Arc<Core>,
}

impl ProviderGate {
    /// Asks `ids` once for the gate's name, which every ticket carries.
    pub fn new(deps: GateDeps, ids: &dyn Ids) -> Self {
        Self {
            core: Arc::new(Core {
                runtime: deps.runtime,
                scheduler: deps.scheduler,
                id: ids.new_id(),
                pause_ms: deps.pause_ms.unwrap_or(PAUSE_MS),
                max_pauses: deps.max_pauses.unwrap_or(MAX_PAUSES),
                on_ticket: deps.on_ticket,
                on_withdrawn: deps.on_withdrawn,
                state: Mutex::new(State {
                    models: Vec::new(),
                    serial: 0,
                    epoch: 0,
                    disposed: false,
                }),
                outbox: Mutex::new(VecDeque::new()),
                flushing: AtomicBool::new(false),
                listeners: Mutex::new(Vec::new()),
                next_listener: Mutex::new(0),
            }),
        }
    }

    /// Retries go first; a cancelled or dropped waiter leaves the queue and takes no turn.
    /// A retired book answers `Retired`.
    pub fn admit(
        &self,
        book: &Arc<GateBook>,
        model: &str,
        cancel: &CancelToken,
        on_wait: Option<WaitFn>,
        again: bool,
    ) -> impl Future<Output = Admission> + Send + use<> {
        let queued = if cancel.is_cancelled() || locked(&self.core.state).disposed {
            Err(stopped())
        } else {
            let (send, receive) = oneshot::channel();
            let (held, watched) = oneshot::channel::<()>();
            let waiter = Waiter {
                send,
                on_wait: on_wait.clone(),
                cancel: cancel.clone(),
                _held: held,
            };
            let waiting = self.core.act(|machine| {
                if machine.state.disposed {
                    return Err(stopped());
                }
                // The test shares the lock `retire` takes.
                if book.is_retired() {
                    return Err(Admission::Retired(AdmissionRetired {}));
                }
                let at = index_of(machine.state, book, model);
                let phase = machine.state.models[at].phase;
                if phase == Phase::Open {
                    let admission = Core::shifting(machine, book, model, |machine, at| {
                        machine.go_now(at, false)
                    });
                    return Err(admission);
                }
                if phase == Phase::Probing && machine.state.models[at].probe.is_none() {
                    let admission = Core::shifting(machine, book, model, |machine, at| {
                        machine.go_now(at, true)
                    });
                    return Err(admission);
                }
                if again {
                    machine.state.models[at].queue.push_front(waiter);
                } else {
                    machine.state.models[at].queue.push_back(waiter);
                }
                if let Some(on_wait) = on_wait {
                    let waiting = machine.wait_of(at);
                    machine.effects.push(Effect::Wait(on_wait, waiting));
                }
                Ok(())
            });
            match waiting {
                Err(admission) => Err(admission),
                Ok(()) => {
                    self.watch(cancel, watched);
                    Ok(receive)
                }
            }
        };
        async move {
            match queued {
                Err(admission) => admission,
                Ok(receive) => receive.await.unwrap_or_else(|_| stopped()),
            }
        }
    }

    /// A cancel removes its waiter without waiting for the next call.
    fn watch(&self, cancel: &CancelToken, watched: oneshot::Receiver<()>) {
        let weak: Weak<Core> = Arc::downgrade(&self.core);
        let cancel = cancel.clone();
        self.core.runtime.spawn(async move {
            // The waiter ending first ends the cancel watch.
            if cancel.run_until(watched).await.is_none()
                && let Some(core) = weak.upgrade()
            {
                core.act(|_| ());
            }
        });
    }

    /// Nonblocking, even for probes; none for a retired book.
    pub fn try_admit(&self, book: &Arc<GateBook>, model: &str) -> Option<Ticket> {
        self.core.act(|machine| {
            if machine.state.disposed || book.is_retired() {
                return None;
            }
            let at = index_of(machine.state, book, model);
            if !ready_of(&machine.state.models[at]) {
                return None;
            }
            let probe = machine.state.models[at].phase == Phase::Probing;
            match Core::shifting(machine, book, model, |machine, at| {
                machine.go_now(at, probe)
            }) {
                Admission::Go(held) => Some(held.ticket),
                _ => None,
            }
        })
    }

    /// Open or probe-ready.
    pub fn ready(&self, book: &Arc<GateBook>, model: &str) -> bool {
        self.core.act(|machine| {
            find(machine.state, book, model).is_none_or(|at| ready_of(&machine.state.models[at]))
        })
    }

    /// Pauses an open model.
    pub fn cool(&self, book: &Arc<GateBook>, model: &str, retry_after_ms: Option<i64>) {
        self.core.act(|machine| {
            if machine.state.disposed || book.is_retired() {
                return;
            }
            let at = index_of(machine.state, book, model);
            if machine.state.models[at].phase != Phase::Open {
                return;
            }
            Core::shifting(machine, book, model, |machine, at| {
                machine.state.models[at].pauses = 1;
                machine.pause(at, retry_after_ms);
            });
        });
    }

    /// Answers the book's waiters `Retired` and forgets its rows.
    pub fn retire(&self, book: &Arc<GateBook>) {
        if book.retired.swap(true, Ordering::SeqCst) {
            return;
        }
        self.core.act(|machine| {
            if machine.state.disposed {
                return;
            }
            let models = std::mem::take(&mut machine.state.models);
            let mut turned = Vec::new();
            for mut model in models {
                if model.book.era != book.era {
                    machine.state.models.push(model);
                    continue;
                }
                if !ready_of(&model) {
                    turned.push(model.name.clone());
                }
                if let Some(timer) = model.timer.take() {
                    machine.effects.push(Effect::Cancel(timer));
                }
                for waiter in model.queue.drain(..) {
                    machine.effects.push(Effect::Hand(
                        waiter.send,
                        Admission::Retired(AdmissionRetired {}),
                        Arc::clone(book),
                    ));
                }
            }
            for model in turned {
                machine.effects.push(Effect::Tell {
                    owner: book.owner.clone(),
                    model,
                });
            }
        });
    }

    /// A success resets the give-up count even while paused; another gate's or era's ticket does nothing.
    pub fn settle(
        &self,
        book: &Arc<GateBook>,
        ticket: &Ticket,
        outcome: GateOutcome,
        retry_after_ms: Option<i64>,
    ) -> GateSettled {
        self.core.settle(book, ticket, outcome, retry_after_ms)
    }

    /// One model's gate; none until the book first uses it.
    pub fn state_of(&self, book: &Arc<GateBook>, model: &str) -> Option<GateState> {
        self.core.act(|machine| {
            find(machine.state, book, model).map(|at| {
                let held = &machine.state.models[at];
                GateState {
                    phase: held.phase,
                    generation: held.generation,
                    pauses: held.pauses,
                    resume_at: held.resume_at,
                    probe: held.probe.clone(),
                    queued: held.queue.len(),
                }
            })
        })
    }

    /// The book's models whose gate is not ready, in code unit order.
    pub fn holding(&self, book: &Arc<GateBook>) -> Vec<String> {
        self.core.act(|machine| {
            let held: Vec<String> = machine
                .state
                .models
                .iter()
                .filter(|held| held.book.era == book.era && !ready_of(held))
                .map(|held| held.name.clone())
                .collect();
            sorted_texts(&held)
        })
    }

    /// Told an owner and model whose gate turned ready or not ready; the result unsubscribes.
    pub fn on_change(&self, listener: ChangeFn) -> Unsubscribe {
        let id = {
            let mut next = locked(&self.core.next_listener);
            *next += 1;
            *next
        };
        locked(&self.core.listeners).push((id, Arc::from(listener)));
        let core = Arc::downgrade(&self.core);
        Box::new(move || {
            if let Some(core) = core.upgrade() {
                locked(&core.listeners).retain(|(held, _)| *held != id);
            }
        })
    }

    /// A disposed gate stays disposed: it admits nothing and makes no state.
    pub fn dispose(&self) {
        if locked(&self.core.state).disposed {
            return;
        }
        self.core.act(|machine| {
            machine.state.disposed = true;
            let mut turned = Vec::new();
            let models = std::mem::take(&mut machine.state.models);
            for mut model in models {
                if !ready_of(&model) {
                    turned.push((model.book.owner.clone(), model.name.clone()));
                }
                if let Some(timer) = model.timer.take() {
                    machine.effects.push(Effect::Cancel(timer));
                }
                for waiter in model.queue.drain(..) {
                    machine.effects.push(Effect::Hand(
                        waiter.send,
                        stopped(),
                        Arc::clone(&model.book),
                    ));
                }
            }
            for (owner, model) in turned {
                machine.effects.push(Effect::Tell { owner, model });
            }
        });
        locked(&self.core.listeners).clear();
    }
}

#[cfg(test)]
mod tests {
    use std::task::{Context, Poll, Waker};

    use nyaaai_contract::id::CountedIds;
    use nyaaai_testkit::scheduler::{FakeScheduler, TimerWheel};

    use super::*;

    fn rt() -> tokio::runtime::Runtime {
        tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
    }

    fn gate(runtime: &tokio::runtime::Runtime) -> (ProviderGate, Arc<TimerWheel>) {
        let wheel = Arc::new(TimerWheel::new(0));
        let scheduler = Arc::new(FakeScheduler::new(Arc::clone(&wheel)));
        let gate = ProviderGate::new(
            GateDeps {
                runtime: runtime.handle().clone(),
                scheduler,
                pause_ms: None,
                max_pauses: None,
                on_ticket: None,
                on_withdrawn: None,
            },
            &CountedIds::new("gate-"),
        );
        (gate, wheel)
    }

    #[test]
    fn a_dropped_admission_gives_its_turn_to_the_next_waiter() {
        let runtime = rt();
        let (gate, wheel) = gate(&runtime);
        let book = Arc::new(GateBook::new(1, "owner".to_owned()));
        gate.cool(&book, "model", None);
        let (first, second) = (CancelToken::new(), CancelToken::new());
        let dropped = gate.admit(&book, "model", &first, None, false);
        let mut kept = Box::pin(gate.admit(&book, "model", &second, None, false));
        drop(dropped);
        wheel.advance(60_000);
        let mut context = Context::from_waker(Waker::noop());
        let Poll::Ready(Admission::Go(probe)) = kept.as_mut().poll(&mut context) else {
            panic!("the waiter that is still there should hold the probe");
        };
        assert!(probe.ticket.probe);
        assert_eq!(
            gate.state_of(&book, "model").map(|held| held.queued),
            Some(0)
        );
        assert_eq!(
            gate.settle(&book, &probe.ticket, GateOutcome::Ok, None),
            GateSettled::Done
        );
        assert!(gate.holding(&book).is_empty());
    }

    #[test]
    fn a_dropped_admission_does_not_hold_the_model_for_good() {
        let runtime = rt();
        let (gate, wheel) = gate(&runtime);
        let book = Arc::new(GateBook::new(1, "owner".to_owned()));
        gate.cool(&book, "model", None);
        drop(gate.admit(&book, "model", &CancelToken::new(), None, false));
        wheel.advance(60_000);
        assert!(gate.ready(&book, "model"));
        assert!(gate.try_admit(&book, "model").is_some());
    }

    #[test]
    fn a_waiter_cancelled_from_a_plain_thread_is_released_without_another_gate_call() {
        let runtime = rt();
        let (gate, _wheel) = gate(&runtime);
        let book = Arc::new(GateBook::new(1, "owner".to_owned()));
        let token = CancelToken::new();
        let waiting = std::thread::scope(|scope| {
            scope
                .spawn(|| {
                    gate.cool(&book, "model", None);
                    gate.admit(&book, "model", &token, None, false)
                })
                .join()
                .expect("joined")
        });
        token.cancel();
        runtime.block_on(async {
            assert!(matches!(waiting.await, Admission::Stopped(_)));
        });
    }
}
