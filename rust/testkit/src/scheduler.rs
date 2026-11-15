//! Virtual-time test scheduler.

use std::cell::{Cell, RefCell};

use nyaaai_contract::time::Clock;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fired {
    pub tag: String,
    pub at_ms: i64,
}

/// Tagged timers fire by deadline, insertion order on ties.
pub struct TimerWheel {
    at_ms: Cell<i64>,
    timers: RefCell<Vec<Fired>>,
}

impl TimerWheel {
    pub fn new(at_ms: i64) -> Self {
        Self {
            at_ms: Cell::new(at_ms),
            timers: RefCell::default(),
        }
    }

    /// Re-setting a tag replaces its timer.
    pub fn after(&self, ms: i64, tag: &str) {
        self.cancel(tag);
        self.timers.borrow_mut().push(Fired {
            tag: tag.to_owned(),
            at_ms: self.at_ms.get().saturating_add(ms.max(0)),
        });
    }

    /// Whether a timer was waiting.
    pub fn cancel(&self, tag: &str) -> bool {
        let mut timers = self.timers.borrow_mut();
        let before = timers.len();
        timers.retain(|timer| timer.tag != tag);
        timers.len() < before
    }

    pub fn waiting(&self) -> Vec<String> {
        self.timers
            .borrow()
            .iter()
            .map(|timer| timer.tag.clone())
            .collect()
    }

    /// Returns timers due after advancing.
    pub fn advance(&self, ms: i64) -> Vec<Fired> {
        let until = self.at_ms.get().saturating_add(ms.max(0));
        let mut fired = Vec::new();
        loop {
            let mut timers = self.timers.borrow_mut();
            let next = timers
                .iter()
                .enumerate()
                .filter(|(_, timer)| timer.at_ms <= until)
                .min_by_key(|(at, timer)| (timer.at_ms, *at))
                .map(|(at, _)| at);
            let Some(at) = next else {
                break;
            };
            let timer = timers.remove(at);
            self.at_ms.set(timer.at_ms);
            fired.push(timer);
        }
        self.at_ms.set(until);
        fired
    }
}

impl Clock for TimerWheel {
    fn now_ms(&self) -> i64 {
        self.at_ms.get()
    }
}
