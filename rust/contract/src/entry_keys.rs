//! The key source port.

use crate::cancel::Unsubscribe;
use crate::future::BoxFuture;

/// A key with what it was given for (`bound_key` makes both); none; refused by its server; or not yet read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyAnswer {
    Key { key: String, bound_to: String },
    None,
    Refused,
    Unready,
}

/// Keys by entry id.
pub trait EntryKeys: Send + Sync {
    /// Never fails: a store that cannot be read answers `Unready`.
    fn get(&self, id: &str) -> BoxFuture<'_, KeyAnswer>;

    /// Told the id of an entry whose key changed; the result unsubscribes.
    fn on_change(&self, listener: Box<dyn Fn(&str) + Send + Sync>) -> Unsubscribe;
}
