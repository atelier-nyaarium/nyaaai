//! Virtual-time test scheduler.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

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

    fn locked(&self) -> MutexGuard<'_, State> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Re-setting a tag replaces its timer.
    pub fn after(&self, ms: i64, tag: &str) {
        self.set(ms, tag, None);
    }

    /// The call runs as the timer fires.
    pub fn after_run(&self, ms: i64, tag: &str, run: Box<dyn FnOnce() + Send>) {
        self.set(ms, tag, Some(run));
    }

    fn set(&self, ms: i64, tag: &str, run: Option<Box<dyn FnOnce() + Send>>) {
        self.cancel(tag);
        let mut state = self.locked();
        let at_ms = state.at_ms.saturating_add(ms.max(0));
        state.timers.push(Timer {
            fired: Fired {
                tag: tag.to_owned(),
                at_ms,
            },
            run,
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
                state.at_ms = timer.fired.at_ms;
                timer
            };
            fired.push(timer.fired);
            if let Some(run) = timer.run {
                run();
            }
        }
        self.locked().at_ms = until;
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
        self.wheel.after_run(
            ms,
            &tag,
            Box::new(move || {
                let _ = wake.send(());
            }),
        );
        let wheel = Arc::clone(&self.wheel);
        let cancel = cancel.clone();
        Box::pin(async move {
            match cancel.run_until(woken).await {
                Some(_) => Slept::Done,
                None => {
                    wheel.cancel(&tag);
                    Slept::Cancelled
                }
            }
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
