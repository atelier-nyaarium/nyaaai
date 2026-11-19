//! The usage sink port.

use crate::future::BoxFuture;
use crate::shapes::usage::{GenerationStats, MeteredCall, UnfilledRecord};

/// Where metered calls and their later stats are written.
pub trait UsageSink: Send + Sync {
    /// The record's id; `None` when not kept.
    fn record(&self, call: &MeteredCall) -> Option<String>;

    /// Joins stats to a record by its id.
    fn fill(&self, id: &str, stats: &GenerationStats);

    /// Whether the record already holds stats; `at` is when the call began.
    fn filled(&self, id: &str, at: i64) -> BoxFuture<'_, bool>;

    /// Records since `since` that name a generation with no stats yet.
    fn unfilled(&self, since: i64) -> BoxFuture<'_, Vec<UnfilledRecord>>;
}
