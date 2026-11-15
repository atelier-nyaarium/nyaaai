//! ECMAScript number conversion.

use crate::json::number_to_string;

/// Largest exact double integer.
const SAFE: f64 = 9_007_199_254_740_991.0;
const MOST_DIGITS: usize = 100;
/// Maximum exact fraction digits.
const EXACT_DIGITS: usize = 1100;

/// Shortest round-trip text.
pub fn number_text(value: f64) -> String {
    number_to_string(value)
}

/// Ties round toward +infinity.
pub fn round_half_up(value: f64) -> f64 {
    if !value.is_finite() {
        return value;
    }
    let floor = value.floor();
    let rounded = if value - floor >= 0.5 {
        floor + 1.0
    } else {
        floor
    };
    if rounded == 0.0 && value < 0.0 {
        -0.0
    } else {
        rounded
    }
}

fn carried(digits: &mut Vec<u8>) {
    for digit in digits.iter_mut().rev() {
        if *digit < b'9' {
            *digit += 1;
            return;
        }
        *digit = b'0';
    }
    digits.insert(0, b'1');
}

/// ECMAScript `toFixed` semantics.
pub fn fixed_text(value: f64, digits: usize) -> String {
    let digits = digits.min(MOST_DIGITS);
    if !value.is_finite() || value.abs() >= 1e21 {
        return number_to_string(value);
    }
    let exact = format!("{:.EXACT_DIGITS$}", value.abs());
    let (whole, fraction) = exact.split_once('.').unwrap_or((&exact, ""));
    let fraction = fraction.as_bytes();
    let mut held: Vec<u8> = whole
        .bytes()
        .chain(fraction.iter().copied().take(digits))
        .collect();
    if fraction.get(digits).is_some_and(|next| *next >= b'5') {
        carried(&mut held);
    }
    let (whole, fraction) = held.split_at(held.len() - digits);
    let mut text = String::with_capacity(held.len() + 2);
    if value < 0.0 {
        text.push('-');
    }
    text.extend(whole.iter().map(|digit| char::from(*digit)));
    if digits > 0 {
        text.push('.');
        text.extend(fraction.iter().map(|digit| char::from(*digit)));
    }
    text
}

fn digits_end(bytes: &[u8], at: usize) -> usize {
    at + bytes[at..]
        .iter()
        .take_while(|byte| byte.is_ascii_digit())
        .count()
}

/// Canonical signed integer text.
fn integer_end(bytes: &[u8]) -> Option<usize> {
    let from = usize::from(bytes.first() == Some(&b'-'));
    let end = digits_end(bytes, from);
    let leading_zero = bytes.get(from) == Some(&b'0') && end - from > 1;
    (end > from && !leading_zero).then_some(end)
}

/// Finite JSON number text.
pub fn number_of_text(text: &str) -> Option<f64> {
    let bytes = text.as_bytes();
    let mut at = integer_end(bytes)?;
    if bytes.get(at) == Some(&b'.') {
        let end = digits_end(bytes, at + 1);
        if end == at + 1 {
            return None;
        }
        at = end;
    }
    if matches!(bytes.get(at), Some(b'e' | b'E')) {
        let from = at + 1 + usize::from(matches!(bytes.get(at + 1), Some(b'+' | b'-')));
        let end = digits_end(bytes, from.min(bytes.len()));
        if end == from {
            return None;
        }
        at = end;
    }
    if at != bytes.len() {
        return None;
    }
    text.parse::<f64>().ok().filter(|value| value.is_finite())
}

/// Safe integer text.
pub fn integer_of_text(text: &str) -> Option<i64> {
    if integer_end(text.as_bytes())? != text.len() {
        return None;
    }
    let value = text.parse::<f64>().ok()?;
    (value.abs() <= SAFE).then_some(value as i64)
}
