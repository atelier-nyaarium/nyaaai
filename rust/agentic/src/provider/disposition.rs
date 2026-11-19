//! What a run does with a failure.

use nyaaai_contract::shapes::model::{ModelView, ModelViewStatus};
use nyaaai_contract::shapes::provider::{
    Disposition, DispositionFallThrough, DispositionRetry, DispositionStop, NoticeKind, SkipReason,
};
use nyaaai_contract::shapes::step::{Failure, FailureCategory};

pub const SKIP_REASONS: [SkipReason; 9] = SkipReason::ALL;

fn fall_through(notice: NoticeKind) -> Disposition {
    Disposition::FallThrough(DispositionFallThrough { notice })
}

fn stop(notice: NoticeKind) -> Disposition {
    Disposition::Stop(DispositionStop { notice })
}

/// Retry, fall-through, or stop.
pub fn disposition_of(failure: &Failure, view: Option<&ModelView>) -> Disposition {
    match failure.category {
        FailureCategory::KeyMissing => fall_through(NoticeKind::KeyMissing),
        FailureCategory::KeyRejected => fall_through(NoticeKind::KeyRejected),
        FailureCategory::Refused => stop(NoticeKind::Refused),
        FailureCategory::BadRequest | FailureCategory::ReplayRejected => {
            stop(NoticeKind::BadRequest)
        }
        // Retry-After spends budget.
        FailureCategory::OutOfCredit => {
            if failure.retry_after_ms.is_none() {
                fall_through(NoticeKind::SpendLimit)
            } else {
                Disposition::Retry(DispositionRetry {})
            }
        }
        // Resets are hours away.
        FailureCategory::Allowance => fall_through(NoticeKind::SpendLimit),
        FailureCategory::NotFound => match view.map(|held| held.status) {
            Some(ModelViewStatus::NoZdrHost) => fall_through(NoticeKind::NoZdrHost),
            Some(ModelViewStatus::Retired) => fall_through(NoticeKind::ModelRetired),
            _ => fall_through(NoticeKind::BadRequest),
        },
        FailureCategory::RateLimited
        | FailureCategory::Overloaded
        | FailureCategory::Unavailable
        | FailureCategory::Network
        | FailureCategory::Stream
        | FailureCategory::Empty
        | FailureCategory::Truncated
        | FailureCategory::Idle => Disposition::Retry(DispositionRetry {}),
    }
}

pub fn skip_reason_of(failure: &Failure) -> SkipReason {
    match failure.category {
        FailureCategory::KeyMissing => SkipReason::KeyMissing,
        FailureCategory::KeyRejected => SkipReason::KeyRejected,
        FailureCategory::Allowance => SkipReason::Allowance,
        FailureCategory::Overloaded => SkipReason::Overloaded,
        FailureCategory::Network => SkipReason::Network,
        FailureCategory::OutOfCredit => SkipReason::Credit,
        FailureCategory::NotFound => SkipReason::Model,
        _ => SkipReason::Unavailable,
    }
}
