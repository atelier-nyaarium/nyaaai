//! Random ID generation.

use std::fs::File;
use std::hash::{BuildHasher, Hasher, RandomState};
use std::io::Read;
use std::sync::atomic::{AtomicU64, Ordering};

pub trait Ids {
    fn new_id(&self) -> String;
}

pub struct RandomIds;

impl Ids for RandomIds {
    fn new_id(&self) -> String {
        random_uuid()
    }
}

/// IDs start at 1.
pub struct CountedIds {
    prefix: String,
    minted: AtomicU64,
}

impl CountedIds {
    pub fn new(prefix: impl Into<String>) -> Self {
        Self {
            prefix: prefix.into(),
            minted: AtomicU64::new(0),
        }
    }
}

impl Ids for CountedIds {
    fn new_id(&self) -> String {
        let minted = self.minted.fetch_add(1, Ordering::Relaxed) + 1;
        format!("{}{minted}", self.prefix)
    }
}

/// OS randomness, keyed-hash fallback.
pub(crate) fn random_bytes() -> [u8; 16] {
    let mut bytes = [0u8; 16];
    let read = File::open("/dev/urandom").and_then(|mut device| device.read_exact(&mut bytes));
    if read.is_err() {
        let word = || RandomState::new().build_hasher().finish();
        bytes[..8].copy_from_slice(&word().to_be_bytes());
        bytes[8..].copy_from_slice(&word().to_be_bytes());
    }
    bytes
}

/// Mints a version 4 UUID.
pub fn random_uuid() -> String {
    let mut bytes = random_bytes();
    bytes[6] = (bytes[6] & 0x0F) | 0x40;
    bytes[8] = (bytes[8] & 0x3F) | 0x80;
    let hex: String = bytes.iter().map(|byte| format!("{byte:02x}")).collect();
    format!(
        "{}-{}-{}-{}-{}",
        &hex[..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..]
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mints_distinct_version_four_ids() {
        let id = random_uuid();
        assert_eq!(id.len(), 36);
        for (at, unit) in id.bytes().enumerate() {
            let hyphen = matches!(at, 8 | 13 | 18 | 23);
            assert_eq!(unit == b'-', hyphen, "{id}");
            assert!(hyphen || matches!(unit, b'0'..=b'9' | b'a'..=b'f'), "{id}");
        }
        assert_eq!(id.as_bytes()[14], b'4');
        assert!(matches!(id.as_bytes()[19], b'8' | b'9' | b'a' | b'b'));
        assert_ne!(id, random_uuid());
    }
}
