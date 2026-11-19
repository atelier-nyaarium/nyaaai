//! Retry cases.

use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};

use nyaaai_agentic::lifecycle::retry::{NoticeLife, RetryPolicy, notice_lifetime};
use nyaaai_contract::random::{Random, SequenceRandom};
use nyaaai_contract::shapes::provider::NoticeKind;
use nyaaai_testkit::cases::Outcome;

use super::shapes::*;

/// A scripted source that counts its draws.
struct Counting {
    inner: SequenceRandom,
    taken: Arc<AtomicI64>,
}

impl Random for Counting {
    fn next(&self) -> f64 {
        self.taken.fetch_add(1, Ordering::SeqCst);
        self.inner.next()
    }
}

pub fn wait(input: RetryWaitInput) -> Outcome<RetryWaitResult> {
    let taken = Arc::new(AtomicI64::new(0));
    let inner =
        SequenceRandom::new(input.draws.clone()).ok_or_else(|| "the draws are empty".to_owned())?;
    let policy = RetryPolicy::new(
        &input.options,
        Arc::new(Counting {
            inner,
            taken: Arc::clone(&taken),
        }),
    );
    let rows = input
        .rows
        .iter()
        .map(|row| {
            let before = taken.load(Ordering::SeqCst);
            let wait_ms = policy.wait(row.retry_after_ms, row.attempt, row.waited_ms);
            RetryWaitResultRowsItem {
                wait_ms,
                draws: taken.load(Ordering::SeqCst) - before,
            }
        })
        .collect();
    Ok(RetryWaitResult { rows })
}

pub fn lifetime(_input: RetryLifetimeInput) -> Outcome<RetryLifetimeResult> {
    Ok(RetryLifetimeResult {
        rows: NoticeKind::ALL
            .iter()
            .map(|notice| RetryLifetimeResultRowsItem {
                notice: *notice,
                lifetime: match notice_lifetime(*notice) {
                    NoticeLife::Outcome => RetryLifetimeResultRowsItemLifetime::Outcome,
                    NoticeLife::MidTurn => RetryLifetimeResultRowsItemLifetime::MidTurn,
                },
            })
            .collect(),
    })
}
