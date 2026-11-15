//! Padded standard base64.

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub fn base64_encode(bytes: &[u8]) -> String {
    let mut text = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let first = chunk[0];
        let second = chunk.get(1).copied().unwrap_or(0);
        let third = chunk.get(2).copied().unwrap_or(0);
        let sixes = [
            first >> 2,
            ((first & 3) << 4) | (second >> 4),
            ((second & 15) << 2) | (third >> 6),
            third & 63,
        ];
        for (place, six) in sixes.into_iter().enumerate() {
            if place <= chunk.len() {
                text.push(char::from(ALPHABET[usize::from(six)]));
            } else {
                text.push('=');
            }
        }
    }
    text
}

/// Strict padded base64.
pub fn base64_decode(text: &str) -> Option<Vec<u8>> {
    let held = text.as_bytes();
    if !held.len().is_multiple_of(4) {
        return None;
    }
    let padding = held
        .iter()
        .rev()
        .take(2)
        .take_while(|byte| **byte == b'=')
        .count();
    let mut bytes = Vec::with_capacity(held.len() / 4 * 3);
    for (index, quad) in held.chunks(4).enumerate() {
        let last = (index + 1) * 4 == held.len();
        let mut word: u32 = 0;
        for (place, byte) in quad.iter().enumerate() {
            let padded = last && place >= 4 - padding;
            let six = if padded {
                0
            } else {
                ALPHABET.iter().position(|letter| letter == byte)? as u32
            };
            word = (word << 6) | six;
        }
        let count = if last { 3 - padding } else { 3 };
        // Reject stray bits.
        if word & ((1 << (8 * (3 - count))) - 1) != 0 {
            return None;
        }
        for place in 0..count {
            bytes.push((word >> (16 - 8 * place)) as u8);
        }
    }
    Some(bytes)
}
