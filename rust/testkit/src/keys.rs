//! A scripted key source.

use std::sync::atomic::{AtomicUsize, Ordering};

use nyaaai_contract::future::BoxFuture;

/// Answers each read from `values`; the last repeats. `None` is no key.
pub struct FakeKeys {
    values: Vec<Option<String>>,
    reads: AtomicUsize,
}

impl FakeKeys {
    /// `None`: a source needs a value.
    pub fn new(values: Vec<Option<String>>) -> Option<Self> {
        (!values.is_empty()).then(|| Self {
            values,
            reads: AtomicUsize::new(0),
        })
    }

    pub fn get(&self) -> BoxFuture<'_, Option<String>> {
        let read = self.reads.fetch_add(1, Ordering::Relaxed);
        let at = read.min(self.values.len() - 1);
        Box::pin(async move { self.values[at].clone() })
    }

    /// Times the source was read.
    pub fn reads(&self) -> usize {
        self.reads.load(Ordering::Relaxed)
    }
}
