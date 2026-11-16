//! Virtual-time test scheduler.

use std::future::{Future, poll_fn};
use std::pin::{Pin, pin};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::task::Poll;

use nyaaai_contract::cancel::CancelToken;
use nyaaai_contract::future::BoxFuture;
use nyaaai_contract::time::{Clock, Scheduled, Scheduler, Slept};
use tokio::sync::oneshot;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fired {
    pub tag: String,
    pub at_ms: i64,
}

struct Timer {
    fired: Fired,
    run: Option<Box<dyn FnOnce() + Send>>,
    /// A cancelled token drops the timer.
    guard: Option<CancelToken>,
}

struct State {
    at_ms: i64,
    timers: Vec<Timer>,
}

/// Tagged timers fire by deadline, insertion order on ties.
pub struct TimerWheel {
    state: Mutex<State>,
}

impl TimerWheel {
    pub fn new(at_ms: i64) -> Self {
        Self {
            state: Mutex::new(State {
                at_ms,
                timers: Vec::new(),
            }),
        }
    }

    /// The state, cancelled timers gone.
    fn locked(&self) -> MutexGuard<'_, State> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state
            .timers
            .retain(|timer| !timer.guard.as_ref().is_some_and(CancelToken::is_cancelled));
        state
    }

    /// Re-setting a tag replaces its timer.
    pub fn after(&self, ms: i64, tag: &str) {
        self.set(ms, tag, None, None);
    }

    /// The call runs as the timer fires.
    pub fn after_run(&self, ms: i64, tag: &str, run: Box<dyn FnOnce() + Send>) {
        self.set(ms, tag, Some(run), None);
    }

    /// The timer leaves the wheel once `guard` is cancelled.
    pub fn after_run_guarded(
        &self,
        ms: i64,
        tag: &str,
        guard: &CancelToken,
        run: Box<dyn FnOnce() + Send>,
    ) {
        self.set(ms, tag, Some(run), Some(guard.clone()));
    }

    fn set(
        &self,
        ms: i64,
        tag: &str,
        run: Option<Box<dyn FnOnce() + Send>>,
        guard: Option<CancelToken>,
    ) {
        let mut state = self.locked();
        state.timers.retain(|timer| timer.fired.tag != tag);
        let at_ms = state.at_ms.saturating_add(ms.max(0));
        state.timers.push(Timer {
            fired: Fired {
                tag: tag.to_owned(),
                at_ms,
            },
            run,
            guard,
        });
    }

    /// Whether a timer was waiting.
    pub fn cancel(&self, tag: &str) -> bool {
        let mut state = self.locked();
        let before = state.timers.len();
        state.timers.retain(|timer| timer.fired.tag != tag);
        state.timers.len() < before
    }

    pub fn waiting(&self) -> Vec<String> {
        self.locked()
            .timers
            .iter()
            .map(|timer| timer.fired.tag.clone())
            .collect()
    }

    /// Returns timers due after advancing; newly due ones also fire.
    pub fn advance(&self, ms: i64) -> Vec<Fired> {
        let until = self.locked().at_ms.saturating_add(ms.max(0));
        let mut fired = Vec::new();
        loop {
            let timer = {
                let mut state = self.locked();
                let next = state
                    .timers
                    .iter()
                    .enumerate()
                    .filter(|(_, timer)| timer.fired.at_ms <= until)
                    .min_by_key(|(at, timer)| (timer.fired.at_ms, *at))
                    .map(|(at, _)| at);
                let Some(at) = next else {
                    break;
                };
                let timer = state.timers.remove(at);
                state.at_ms = state.at_ms.max(timer.fired.at_ms);
                timer
            };
            fired.push(timer.fired);
            if let Some(run) = timer.run {
                run();
            }
        }
        let mut state = self.locked();
        state.at_ms = state.at_ms.max(until);
        drop(state);
        fired
    }
}

impl Clock for TimerWheel {
    fn now_ms(&self) -> i64 {
        self.locked().at_ms
    }
}

/// Time moves only when the wheel advances.
pub struct FakeScheduler {
    wheel: Arc<TimerWheel>,
    sleeps: AtomicU64,
    schedules: AtomicU64,
}

impl FakeScheduler {
    pub fn new(wheel: Arc<TimerWheel>) -> Self {
        Self {
            wheel,
            sleeps: AtomicU64::new(0),
            schedules: AtomicU64::new(0),
        }
    }

    pub fn wheel(&self) -> &Arc<TimerWheel> {
        &self.wheel
    }
}

impl Clock for FakeScheduler {
    fn now_ms(&self) -> i64 {
        self.wheel.now_ms()
    }
}

impl Scheduler for FakeScheduler {
    fn sleep(&self, ms: i64, cancel: &CancelToken) -> BoxFuture<'static, Slept> {
        if cancel.is_cancelled() {
            return Box::pin(async { Slept::Cancelled });
        }
        let tag = format!("sleep-{}", self.sleeps.fetch_add(1, Ordering::Relaxed) + 1);
        let (wake, woken) = oneshot::channel();
        self.wheel.after_run_guarded(
            ms,
            &tag,
            cancel,
            Box::new(move || {
                let _ = wake.send(());
            }),
        );
        let cancel = cancel.clone();
        Box::pin(async move {
            let mut woken = woken;
            let mut cancelled = pin!(cancel.cancelled());
            // A timer taken off the wheel by its tag leaves the sleep pending.
            let mut gone = false;
            poll_fn(|context| {
                if !gone {
                    match Pin::new(&mut woken).poll(context) {
                        Poll::Ready(Ok(())) => return Poll::Ready(Slept::Done),
                        Poll::Ready(Err(_)) => gone = true,
                        Poll::Pending => {}
                    }
                }
                match cancelled.as_mut().poll(context) {
                    Poll::Ready(()) => Poll::Ready(Slept::Cancelled),
                    Poll::Pending => Poll::Pending,
                }
            })
            .await
        })
    }

    fn schedule(&self, ms: i64, run: Box<dyn FnOnce() + Send>) -> Scheduled {
        let tag = format!(
            "schedule-{}",
            self.schedules.fetch_add(1, Ordering::Relaxed) + 1
        );
        self.wheel.after_run(ms, &tag, run);
        let wheel = Arc::clone(&self.wheel);
        Scheduled::new(move || {
            wheel.cancel(&tag);
        })
    }
}
