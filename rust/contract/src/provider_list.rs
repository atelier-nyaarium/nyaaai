//! The provider list port.

use crate::cancel::Unsubscribe;
use crate::shapes::provider::ProviderEntry;

/// The entries in priority order.
pub trait ProviderList: Send + Sync {
    fn entries(&self) -> Vec<ProviderEntry>;

    /// Told when the list or an entry changed; the result unsubscribes.
    fn on_change(&self, listener: Box<dyn Fn() + Send + Sync>) -> Unsubscribe;
}
