//! Randomness port.

use std::sync::atomic::{AtomicUsize, Ordering};

use crate::id::random_bytes;

pub trait Random {
    /// In [0, 1).
    fn next(&self) -> f64;
}

pub struct SystemRandom;

impl Random for SystemRandom {
    fn next(&self) -> f64 {
        let bytes = random_bytes();
        let mut word = [0u8; 8];
        word.copy_from_slice(&bytes[..8]);
        // Use 53 mantissa bits.
        (u64::from_be_bytes(word) >> 11) as f64 / (1u64 << 53) as f64
    }
}

/// Cycles supplied values.
pub struct SequenceRandom {
    values: Vec<f64>,
    drawn: AtomicUsize,
}

impl SequenceRandom {
    /// `None` for an empty list.
    pub fn new(values: Vec<f64>) -> Option<Self> {
        (!values.is_empty()).then(|| Self {
            values,
            drawn: AtomicUsize::new(0),
        })
    }
}

impl Random for SequenceRandom {
    fn next(&self) -> f64 {
        let drawn = self.drawn.fetch_add(1, Ordering::Relaxed);
        self.values[drawn % self.values.len()]
    }
}
