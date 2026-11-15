//! UTF-16 code unit order.

use std::cmp::Ordering;

/// JavaScript code-unit string order.
pub fn compare_text(left: &str, right: &str) -> Ordering {
    left.encode_utf16().cmp(right.encode_utf16())
}

pub fn sorted_texts(texts: &[String]) -> Vec<String> {
    let mut held = texts.to_vec();
    held.sort_by(|left, right| compare_text(left, right));
    held
}
