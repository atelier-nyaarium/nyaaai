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
    Admission, AdmissionGaveUp, AdmissionGo, AdmissionStopped, GateOutcome, GateSettled,
    GateWaiting, Ticket,
};
use nyaaai_contract::time::{Scheduled, Scheduler};
use tokio::runtime::Handle;
use tokio::sync::oneshot;

/// Told what a queued request waits for.
pub type WaitFn = Arc<dyn Fn(GateWaiting) + Send + Sync>;

type Listener = Arc<dyn Fn(&str) + Send + Sync>;

/// Told a ticket as it is made.
pub type TicketFn = Arc<dyn Fn(&Ticket) + Send + Sync>;

const PAUSE_MS: i64 = 60_000;

const MAX_PAUSES: u64 = 10;

/// A server's asked wait stops here.
const MAX_PAUSE_MS: i64 = 60 * 60_000;

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
    /// Told each ticket in the gate call that makes it, once its holder has it.
    pub on_ticket: Option<TicketFn>,
}

struct Waiter {
    send: oneshot::Sender<Admission>,
    on_wait: Option<WaitFn>,
    cancel: CancelToken,
    /// Dropped with the waiter; ends its cancel watcher.
    _held: oneshot::Sender<()>,
}

struct Model {
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
    /// Answers a waiter; a ticket in it is told once the waiter has it.
    Hand(oneshot::Sender<Admission>, Admission),
    /// A ticket an admission returns at once.
    Made(Ticket),
    Wait(WaitFn, GateWaiting),
    Tell(String),
    Cancel(Scheduled),
    Schedule {
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

fn index_of(state: &mut State, name: &str) -> usize {
    if let Some(at) = state.models.iter().position(|model| model.name == name) {
        return at;
    }
    state.models.push(Model {
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
        let queue = std::mem::take(&mut model.queue);
        for waiter in queue {
            if waiter.cancel.is_cancelled() {
                self.effects.push(Effect::Hand(waiter.send, stopped()));
                continue;
            }
            if waiter.send.is_closed() {
                continue;
            }
            let admission = match release {
                Release::Go => self.go(at, false),
                Release::GaveUp => Admission::GaveUp(AdmissionGaveUp {}),
            };
            self.effects.push(Effect::Hand(waiter.send, admission));
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
            if waiter.cancel.is_cancelled() {
                self.effects.push(Effect::Hand(waiter.send, stopped()));
                continue;
            }
            if waiter.send.is_closed() {
                continue;
            }
            let admission = self.go(at, true);
            self.effects.push(Effect::Hand(waiter.send, admission));
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
                effects.push(Effect::Hand(waiter.send, stopped()));
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
        name: &str,
        run: impl FnOnce(&mut Machine<'_>, usize) -> T,
    ) -> T {
        let at = index_of(machine.state, name);
        let was = ready_of(&machine.state.models[at]);
        let result = run(machine, at);
        if at < machine.state.models.len() && ready_of(&machine.state.models[at]) != was {
            machine.effects.push(Effect::Tell(name.to_owned()));
        }
        result
    }

    fn perform(self: &Arc<Self>, effect: Effect) {
        match effect {
            Effect::Hand(send, admission) => {
                let ticket = match &admission {
                    Admission::Go(held) => Some(held.ticket.clone()),
                    _ => None,
                };
                match send.send(admission) {
                    Ok(()) => {
                        if let Some(ticket) = &ticket {
                            self.made(ticket);
                        }
                    }
                    // The caller is gone: the turn passes on as if it had cancelled.
                    Err(_) => {
                        if let Some(ticket) = &ticket {
                            self.settle(ticket, GateOutcome::Cancelled, None);
                        }
                    }
                }
            }
            Effect::Made(ticket) => self.made(&ticket),
            Effect::Wait(on_wait, waiting) => {
                // A callback cannot change the gate.
                let _ = catch_unwind(AssertUnwindSafe(|| on_wait(waiting)));
            }
            Effect::Tell(model) => self.tell(&model),
            Effect::Cancel(timer) => timer.cancel(),
            Effect::Schedule { model, epoch, ms } => {
                let weak = Arc::downgrade(self);
                let name = model.clone();
                let timer = self.scheduler.schedule(
                    ms,
                    Box::new(move || {
                        if let Some(core) = weak.upgrade() {
                            core.fire(&name, epoch);
                        }
                    }),
                );
                let mut state = locked(&self.state);
                match state
                    .models
                    .iter_mut()
                    .find(|held| held.name == model && held.epoch == epoch)
                {
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

    /// Settles a ticket of this gate; another gate's does nothing.
    fn settle(
        self: &Arc<Self>,
        ticket: &Ticket,
        outcome: GateOutcome,
        retry_after_ms: Option<i64>,
    ) -> GateSettled {
        if ticket.gate != self.id {
            return GateSettled::Done;
        }
        self.act(|machine| {
            if machine.state.disposed {
                return GateSettled::Done;
            }
            if !machine
                .state
                .models
                .iter()
                .any(|held| held.name == ticket.model)
            {
                return GateSettled::Done;
            }
            Self::shifting(machine, &ticket.model, |machine, at| {
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
    fn fire(self: &Arc<Self>, name: &str, epoch: u64) {
        self.act(|machine| {
            let current = machine
                .state
                .models
                .iter()
                .any(|held| held.name == name && held.epoch == epoch);
            if !current {
                return;
            }
            Self::shifting(machine, name, |machine, at| {
                let model = &mut machine.state.models[at];
                model.timer = None;
                model.phase = Phase::Probing;
                machine.next_probe(at);
            });
        });
    }

    fn tell(&self, model: &str) {
        let held: Vec<Listener> = locked(&self.listeners)
            .iter()
            .map(|(_, listener)| Arc::clone(listener))
            .collect();
        for listener in held {
            // A listener cannot change the gate.
            let _ = catch_unwind(AssertUnwindSafe(|| listener(model)));
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
    pub fn admit(
        &self,
        model: &str,
        cancel: &CancelToken,
        on_wait: Option<WaitFn>,
        again: bool,
    ) -> impl Future<Output = Admission> + Send + use<> {
        let queued = if cancel.is_cancelled() {
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
                let at = index_of(machine.state, model);
                let phase = machine.state.models[at].phase;
                if phase == Phase::Open {
                    let admission =
                        Core::shifting(machine, model, |machine, at| machine.go_now(at, false));
                    return Err(admission);
                }
                if phase == Phase::Probing && machine.state.models[at].probe.is_none() {
                    let admission =
                        Core::shifting(machine, model, |machine, at| machine.go_now(at, true));
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

    /// A cancel of the caller removes its waiter on the gate's runtime, without waiting for the next call.
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

    /// Nonblocking, even for probes.
    pub fn try_admit(&self, model: &str) -> Option<Ticket> {
        self.core.act(|machine| {
            if machine.state.disposed {
                return None;
            }
            let at = index_of(machine.state, model);
            if !ready_of(&machine.state.models[at]) {
                return None;
            }
            let probe = machine.state.models[at].phase == Phase::Probing;
            match Core::shifting(machine, model, |machine, at| machine.go_now(at, probe)) {
                Admission::Go(held) => Some(held.ticket),
                _ => None,
            }
        })
    }

    /// Open or probe-ready.
    pub fn ready(&self, model: &str) -> bool {
        self.core.act(|machine| {
            machine
                .state
                .models
                .iter()
                .find(|held| held.name == model)
                .is_none_or(ready_of)
        })
    }

    /// Pauses an open model.
    pub fn cool(&self, model: &str, retry_after_ms: Option<i64>) {
        self.core.act(|machine| {
            if machine.state.disposed {
                return;
            }
            let at = index_of(machine.state, model);
            if machine.state.models[at].phase != Phase::Open {
                return;
            }
            Core::shifting(machine, model, |machine, at| {
                machine.state.models[at].pauses = 1;
                machine.pause(at, retry_after_ms);
            });
        });
    }

    /// Reopens matching gates.
    pub fn clear(&self, matches: impl Fn(&str) -> bool) {
        self.core.act(|machine| {
            if machine.state.disposed {
                return;
            }
            let names: Vec<String> = machine
                .state
                .models
                .iter()
                .filter(|held| matches(&held.name) && held.phase != Phase::Open)
                .map(|held| held.name.clone())
                .collect();
            for name in names {
                Core::shifting(machine, &name, |machine, at| {
                    let model = &mut machine.state.models[at];
                    if let Some(timer) = model.timer.take() {
                        machine.effects.push(Effect::Cancel(timer));
                    }
                    machine.state.epoch += 1;
                    let epoch = machine.state.epoch;
                    let model = &mut machine.state.models[at];
                    model.epoch = epoch;
                    model.probe = None;
                    machine.reopen(at, Release::Go);
                });
            }
        });
    }

    /// An earlier request's success resets the give-up count even while paused; another gate's ticket does nothing.
    pub fn settle(
        &self,
        ticket: &Ticket,
        outcome: GateOutcome,
        retry_after_ms: Option<i64>,
    ) -> GateSettled {
        self.core.settle(ticket, outcome, retry_after_ms)
    }

    /// One model's gate; none until the gate is first used.
    pub fn state_of(&self, model: &str) -> Option<GateState> {
        self.core.act(|machine| {
            machine
                .state
                .models
                .iter()
                .find(|held| held.name == model)
                .map(|held| GateState {
                    phase: held.phase,
                    generation: held.generation,
                    pauses: held.pauses,
                    resume_at: held.resume_at,
                    probe: held.probe.clone(),
                    queued: held.queue.len(),
                })
        })
    }

    /// Models whose gate is not ready, in code unit order.
    pub fn holding(&self) -> Vec<String> {
        self.core.act(|machine| {
            let held: Vec<String> = machine
                .state
                .models
                .iter()
                .filter(|held| !ready_of(held))
                .map(|held| held.name.clone())
                .collect();
            sorted_texts(&held)
        })
    }

    /// Told a model whose gate turned ready or not ready; the result unsubscribes.
    pub fn on_change(&self, listener: Box<dyn Fn(&str) + Send + Sync>) -> Unsubscribe {
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
                    turned.push(model.name.clone());
                }
                if let Some(timer) = model.timer.take() {
                    machine.effects.push(Effect::Cancel(timer));
                }
                for waiter in model.queue.drain(..) {
                    machine.effects.push(Effect::Hand(waiter.send, stopped()));
                }
            }
            for name in turned {
                machine.effects.push(Effect::Tell(name));
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
            },
            &CountedIds::new("gate-"),
        );
        (gate, wheel)
    }

    #[test]
    fn a_cancelled_waiter_leaves_the_queue_at_once() {
        let runtime = rt();
        runtime.block_on(async {
            let (gate, _wheel) = gate(&runtime);
            gate.cool("model", None);
            let token = CancelToken::new();
            let waiting = gate.admit("model", &token, None, false);
            assert_eq!(gate.state_of("model").map(|held| held.queued), Some(1));
            token.cancel();
            assert_eq!(gate.state_of("model").map(|held| held.queued), Some(0));
            assert!(matches!(waiting.await, Admission::Stopped(_)));
        });
    }

    #[test]
    fn a_pause_hands_the_probe_to_the_oldest_waiter() {
        let runtime = rt();
        let (gate, wheel) = gate(&runtime);
        gate.cool("model", None);
        let (first, second) = (CancelToken::new(), CancelToken::new());
        let (mut first, mut second) = (
            Box::pin(gate.admit("model", &first, None, false)),
            Box::pin(gate.admit("model", &second, None, false)),
        );
        wheel.advance(60_000);
        let mut context = Context::from_waker(Waker::noop());
        let Poll::Ready(Admission::Go(probe)) = first.as_mut().poll(&mut context) else {
            panic!("the oldest waiter should hold the probe");
        };
        assert!(probe.ticket.probe);
        assert!(second.as_mut().poll(&mut context).is_pending());
        assert_eq!(gate.holding(), vec!["model".to_owned()]);
    }

    #[test]
    fn a_dropped_admission_gives_its_turn_to_the_next_waiter() {
        let runtime = rt();
        let (gate, wheel) = gate(&runtime);
        gate.cool("model", None);
        let (first, second) = (CancelToken::new(), CancelToken::new());
        let dropped = gate.admit("model", &first, None, false);
        let mut kept = Box::pin(gate.admit("model", &second, None, false));
        drop(dropped);
        wheel.advance(60_000);
        let mut context = Context::from_waker(Waker::noop());
        let Poll::Ready(Admission::Go(probe)) = kept.as_mut().poll(&mut context) else {
            panic!("the waiter that is still there should hold the probe");
        };
        assert!(probe.ticket.probe);
        assert_eq!(gate.state_of("model").map(|held| held.queued), Some(0));
        assert_eq!(
            gate.settle(&probe.ticket, GateOutcome::Ok, None),
            GateSettled::Done
        );
        assert!(gate.holding().is_empty());
    }

    #[test]
    fn a_dropped_admission_does_not_hold_the_model_for_good() {
        let runtime = rt();
        let (gate, wheel) = gate(&runtime);
        gate.cool("model", None);
        drop(gate.admit("model", &CancelToken::new(), None, false));
        wheel.advance(60_000);
        assert!(gate.ready("model"));
        assert!(gate.try_admit("model").is_some());
    }

    #[test]
    fn a_waiter_cancelled_from_a_plain_thread_is_released_without_another_gate_call() {
        let runtime = rt();
        let (gate, _wheel) = gate(&runtime);
        let token = CancelToken::new();
        let waiting = std::thread::scope(|scope| {
            scope
                .spawn(|| {
                    gate.cool("model", None);
                    gate.admit("model", &token, None, false)
                })
                .join()
                .expect("joined")
        });
        token.cancel();
        runtime.block_on(async {
            assert!(matches!(waiting.await, Admission::Stopped(_)));
        });
    }

    #[test]
    fn a_ticket_of_another_gate_does_nothing_and_a_disposed_gate_stays_disposed() {
        let runtime = rt();
        let (gate, _wheel) = gate(&runtime);
        let mut foreign = gate.try_admit("model").expect("an open gate");
        foreign.gate = "elsewhere".to_owned();
        assert_eq!(
            gate.settle(&foreign, GateOutcome::Overloaded, None),
            GateSettled::Done
        );
        assert_eq!(gate.state_of("other"), None);
        gate.dispose();
        assert!(gate.try_admit("model").is_none());
        gate.cool("model", None);
        assert_eq!(gate.state_of("model"), None);
        let mut waiting = Box::pin(gate.admit("model", &CancelToken::new(), None, false));
        assert!(matches!(
            waiting
                .as_mut()
                .poll(&mut Context::from_waker(Waker::noop())),
            Poll::Ready(Admission::Stopped(_))
        ));
    }
}
