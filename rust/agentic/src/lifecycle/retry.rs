//! How long a request waits before it tries a route again.

use std::sync::Arc;

use nyaaai_contract::number::round_half_up;
use nyaaai_contract::random::Random;
use nyaaai_contract::shapes::health::RetryOptions;
use nyaaai_contract::shapes::provider::NoticeKind;

/// How long a notice stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoticeLife {
    /// Run-ending notices share one entry.
    Outcome,
    /// Mid-turn notices stand alone.
    MidTurn,
}

/// Mid-turn notices stand alone; run-ending notices share one entry.
pub fn notice_lifetime(kind: NoticeKind) -> NoticeLife {
    match kind {
        NoticeKind::HistoryDropped => NoticeLife::MidTurn,
        NoticeKind::KeyMissing
        | NoticeKind::KeyRejected
        | NoticeKind::SpendLimit
        | NoticeKind::Refused
        | NoticeKind::NoZdrHost
        | NoticeKind::ModelRetired
        | NoticeKind::ModelUnavailable
        | NoticeKind::Unreachable
        | NoticeKind::BadRequest
        | NoticeKind::RetriesExhausted
        | NoticeKind::ProviderOverloaded
        | NoticeKind::StreamFailed
        | NoticeKind::OutputCut
        | NoticeKind::NoModel => NoticeLife::Outcome,
    }
}

/// Four tries; 90s wait cap.
pub struct RetryPolicy {
    tries: f64,
    budget_ms: f64,
    cap_ms: f64,
    base_ms: f64,
    random: Arc<dyn Random + Send + Sync>,
}

impl RetryPolicy {
    pub fn new(options: &RetryOptions, random: Arc<dyn Random + Send + Sync>) -> Self {
        Self {
            tries: options.tries.map_or(4.0, |held| held as f64),
            budget_ms: options.budget_ms.unwrap_or(90_000.0),
            cap_ms: options.cap_ms.unwrap_or(30_000.0),
            base_ms: options.base_ms.unwrap_or(1000.0),
            random,
        }
    }

    /// One draw per call past the tries, even when `retry_after_ms` replaces backoff; a wait is at least 0 or none.
    pub fn wait(&self, retry_after_ms: Option<i64>, attempt: i64, waited_ms: f64) -> Option<f64> {
        if attempt as f64 >= self.tries {
            return None;
        }
        let base = if self.base_ms == 0.0 {
            0.0
        } else {
            self.cap_ms
                .min(self.base_ms * 2f64.powf((attempt - 1) as f64))
        };
        let backoff = base * (0.5 + self.random.next() / 2.0);
        let rounded = round_half_up(retry_after_ms.map_or(backoff, |held| held as f64));
        if !rounded.is_finite() {
            return None;
        }
        let wait_ms = if rounded > 0.0 { rounded } else { 0.0 };
        if waited_ms + wait_ms > self.budget_ms {
            None
        } else {
            Some(wait_ms)
        }
    }
}
