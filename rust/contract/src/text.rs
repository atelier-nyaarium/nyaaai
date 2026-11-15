//! UTF-16 cuts preserve pairs.

pub fn text_length(text: &str) -> usize {
    text.chars().map(char::len_utf16).sum()
}

#[derive(Clone, Copy, PartialEq)]
enum Side {
    Before,
    After,
}

/// UTF-16 offset mapping.
fn place(text: &str, at: usize, side: Side) -> (usize, usize) {
    let mut units = 0;
    for (byte, char) in text.char_indices() {
        if units >= at {
            return (byte, units);
        }
        let width = char.len_utf16();
        if units + width > at {
            return match side {
                Side::Before => (byte, units),
                Side::After => (byte + char.len_utf8(), units + width),
            };
        }
        units += width;
    }
    (text.len(), units)
}

/// Clamps; rounds down.
pub fn cut_before(text: &str, at: usize) -> usize {
    place(text, at, Side::Before).1
}

/// Clamps; rounds up.
pub fn cut_after(text: &str, at: usize) -> usize {
    place(text, at, Side::After).1
}

pub fn head_of(text: &str, max: usize) -> &str {
    &text[..place(text, max, Side::Before).0]
}

pub fn tail_of(text: &str, max: usize) -> &str {
    let from = text_length(text).saturating_sub(max);
    &text[place(text, from, Side::After).0..]
}

/// Surrogate-safe bounds.
pub fn slice_text(text: &str, start: usize, end: usize) -> &str {
    let from = place(text, start, Side::Before).0;
    let to = place(text, end, Side::Before).0;
    &text[from..to.max(from)]
}

/// Empty-needle offset is clamped.
pub fn index_of_text(text: &str, needle: &str, from: usize) -> Option<usize> {
    if needle.is_empty() {
        return Some(from.min(text_length(text)));
    }
    let (byte, units) = place(text, from, Side::After);
    let found = text[byte..].find(needle)?;
    Some(units + text_length(&text[byte..byte + found]))
}

/// Empty-needle offset is clamped.
pub fn last_index_of_text(text: &str, needle: &str, from: usize) -> Option<usize> {
    if needle.is_empty() {
        return Some(from.min(text_length(text)));
    }
    let limit = place(text, from, Side::Before).0;
    let (found, _) = text
        .rmatch_indices(needle)
        .find(|(byte, _)| *byte <= limit)?;
    Some(text_length(&text[..found]))
}

pub fn utf8_length(text: &str) -> usize {
    text.len()
}

/// Preserves UTF-8 boundaries.
pub fn cut_utf8(text: &str, max_bytes: usize) -> &str {
    let mut end = max_bytes.min(text.len());
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    &text[..end]
}

pub fn ascii_lower(text: &str) -> String {
    text.to_ascii_lowercase()
}

pub fn ascii_upper(text: &str) -> String {
    text.to_ascii_uppercase()
}

/// ECMAScript whitespace.
pub fn is_space(char: char) -> bool {
    matches!(
        char,
        '\t' | '\n' | '\u{B}' | '\u{C}' | '\r' | ' ' | '\u{A0}' | '\u{1680}' | '\u{2000}'
            ..='\u{200A}'
                | '\u{2028}'
                | '\u{2029}'
                | '\u{202F}'
                | '\u{205F}'
                | '\u{3000}'
                | '\u{FEFF}'
    )
}

/// Trims ECMAScript whitespace.
pub fn trim_text(text: &str) -> &str {
    text.trim_matches(is_space)
}

/// Trims and collapses whitespace.
pub fn collapse_space(text: &str) -> String {
    text.split(is_space)
        .filter(|word| !word.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}
