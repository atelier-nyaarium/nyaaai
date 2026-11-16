//! The model registry port.

use crate::cancel::Unsubscribe;
use crate::future::BoxFuture;
use crate::shapes::model::{ModelRow, StoredRegistry, TestRecord};

/// Rows and the stored registry.
pub trait ModelRegistry: Send + Sync {
    fn rows(&self) -> Vec<ModelRow>;

    /// Records, added keys and hidden keys.
    fn stored(&self) -> StoredRegistry;

    /// Writes one record; `None` when written, else why not.
    fn record(&self, entry: &str, id: &str, record: TestRecord) -> BoxFuture<'_, Option<String>>;

    /// Told when rows or the stored registry changed; the result unsubscribes.
    fn on_change(&self, listener: Box<dyn Fn() + Send + Sync>) -> Unsubscribe;
}
