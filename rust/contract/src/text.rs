//! UTF-16 cuts preserve pairs.

use std::sync::OnceLock;

use crate::nfc_tables::{CCC, COMPOSE, DECOMP};

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

const S_BASE: u32 = 0xac00;
const L_BASE: u32 = 0x1100;
const V_BASE: u32 = 0x1161;
const T_BASE: u32 = 0x11a7;
const L_COUNT: u32 = 19;
const V_COUNT: u32 = 21;
const T_COUNT: u32 = 28;
const N_COUNT: u32 = V_COUNT * T_COUNT;
const S_COUNT: u32 = L_COUNT * N_COUNT;

fn ccc_of(cp: u32) -> u8 {
    let mut lo = 0usize;
    let mut hi = CCC.len();
    while lo < hi {
        let mid = (lo + hi) / 2;
        if (CCC[mid] >> 8) < cp {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    if lo < CCC.len() && (CCC[lo] >> 8) == cp {
        (CCC[lo] & 0xff) as u8
    } else {
        0
    }
}

fn decomp_of(cp: u32) -> Option<(u32, Option<u32>)> {
    let mut lo = 0usize;
    let mut hi = DECOMP.len() / 3;
    while lo < hi {
        let mid = (lo + hi) / 2;
        if DECOMP[mid * 3] < cp {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    if lo >= DECOMP.len() / 3 || DECOMP[lo * 3] != cp {
        return None;
    }
    let second = DECOMP[lo * 3 + 2];
    Some((DECOMP[lo * 3 + 1], (second != 0).then_some(second)))
}

fn compose_of(first: u32, second: u32) -> Option<u32> {
    let mut lo = 0usize;
    let mut hi = COMPOSE.len() / 3;
    while lo < hi {
        let mid = (lo + hi) / 2;
        let held = COMPOSE[mid * 3];
        if held < first || (held == first && COMPOSE[mid * 3 + 1] < second) {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    if lo >= COMPOSE.len() / 3 {
        return None;
    }
    let held = COMPOSE[lo * 3];
    (held == first && COMPOSE[lo * 3 + 1] == second).then(|| COMPOSE[lo * 3 + 2])
}

fn append_decomposed(cp: u32, out: &mut Vec<u32>) {
    if (S_BASE..S_BASE + S_COUNT).contains(&cp) {
        let index = cp - S_BASE;
        out.push(L_BASE + index / N_COUNT);
        out.push(V_BASE + (index % N_COUNT) / T_COUNT);
        let tail = T_BASE + index % T_COUNT;
        if tail != T_BASE {
            out.push(tail);
        }
        return;
    }
    if let Some((first, second)) = decomp_of(cp) {
        append_decomposed(first, out);
        if let Some(mark) = second {
            append_decomposed(mark, out);
        }
        return;
    }
    out.push(cp);
}

/// Stable sort of each combining-mark run by class.
fn reorder(codes: &mut [u32]) {
    let mut at = 0;
    while at < codes.len() {
        if ccc_of(codes[at]) == 0 {
            at += 1;
            continue;
        }
        let mut end = at;
        while end < codes.len() && ccc_of(codes[end]) != 0 {
            end += 1;
        }
        let mut run: Vec<(u32, u8)> = codes[at..end].iter().map(|&cp| (cp, ccc_of(cp))).collect();
        run.sort_by_key(|&(_, class)| class);
        for (cursor, &(cp, _)) in run.iter().enumerate() {
            codes[at + cursor] = cp;
        }
        at = end;
    }
}

fn compose(codes: &mut Vec<u32>) {
    if codes.len() < 2 {
        return;
    }
    let mut out: Vec<u32> = Vec::with_capacity(codes.len());
    out.push(codes[0]);
    let mut starter = 0usize;
    let mut last_class = -1i32;
    for &cp in &codes[1..] {
        let at = out.len() - 1;
        let previous = out[at];
        if (L_BASE..L_BASE + L_COUNT).contains(&previous)
            && (V_BASE..V_BASE + V_COUNT).contains(&cp)
        {
            out[at] = S_BASE + (previous - L_BASE) * N_COUNT + (cp - V_BASE) * T_COUNT;
            starter = at;
            last_class = -1;
            continue;
        }
        if (S_BASE..S_BASE + S_COUNT).contains(&previous)
            && (previous - S_BASE).is_multiple_of(T_COUNT)
            && cp > T_BASE
            && cp < T_BASE + T_COUNT
        {
            out[at] = previous + (cp - T_BASE);
            starter = at;
            last_class = -1;
            continue;
        }
        let class = ccc_of(cp);
        if last_class < class as i32
            && let Some(pair) = compose_of(out[starter], cp)
        {
            out[starter] = pair;
            continue;
        }
        out.push(cp);
        if class == 0 {
            starter = out.len() - 1;
            last_class = -1;
        } else {
            last_class = class as i32;
        }
    }
    *codes = out;
}

const SLOW: u8 = 255;

/// Per code point: its class if Yes, SLOW if No or Maybe; derived once from the tables.
fn quick_table() -> &'static [u8] {
    static TABLE: OnceLock<Vec<u8>> = OnceLock::new();
    TABLE.get_or_init(|| {
        let mut top = 0;
        for packed in CCC {
            top = top.max(packed >> 8);
        }
        for row in DECOMP.as_chunks::<3>().0 {
            top = top.max(row[0]);
        }
        for row in COMPOSE.as_chunks::<3>().0 {
            top = top.max(row[1]);
        }
        let mut table = vec![0u8; top as usize + 1];
        for packed in CCC {
            table[(packed >> 8) as usize] = (packed & 0xff) as u8;
        }
        for row in DECOMP.as_chunks::<3>().0 {
            if row[2] == 0 || compose_of(row[1], row[2]) != Some(row[0]) {
                table[row[0] as usize] = SLOW;
            }
        }
        for row in COMPOSE.as_chunks::<3>().0 {
            table[row[1] as usize] = SLOW;
        }
        for cp in V_BASE..V_BASE + V_COUNT {
            table[cp as usize] = SLOW;
        }
        for cp in T_BASE + 1..T_BASE + T_COUNT {
            table[cp as usize] = SLOW;
        }
        table
    })
}

/// UAX #15 quick check: true only for a text that is already NFC.
fn is_normal(text: &str) -> bool {
    let table = quick_table();
    let mut last = 0u8;
    for char in text.chars() {
        let cp = char as usize;
        if cp < 0x300 {
            last = 0;
            continue;
        }
        let held = table.get(cp).copied().unwrap_or(0);
        if held == 0 {
            last = 0;
            continue;
        }
        if held == SLOW || held < last {
            return false;
        }
        last = held;
    }
    true
}

/// Unicode 17.0 NFC; the answer is a new string even when the text is already normal.
pub fn nfc(text: &str) -> String {
    if text.is_ascii() || is_normal(text) {
        return text.to_owned();
    }
    let mut decomposed: Vec<u32> = Vec::with_capacity(text.len());
    for char in text.chars() {
        append_decomposed(char as u32, &mut decomposed);
    }
    reorder(&mut decomposed);
    compose(&mut decomposed);
    decomposed
        .into_iter()
        .map(|cp| char::from_u32(cp).expect("scalar"))
        .collect()
}

/// WHATWG UTF-8 decoding of chunks.
pub struct Utf8Decoder {
    point: u32,
    needed: u8,
    seen: u8,
    lower: u8,
    upper: u8,
    started: bool,
}

impl Utf8Decoder {
    pub fn new() -> Self {
        Self {
            point: 0,
            needed: 0,
            seen: 0,
            lower: 0x80,
            upper: 0xbf,
            started: false,
        }
    }

    /// Holds an incomplete sequence for the next chunk.
    pub fn push(&mut self, bytes: &[u8]) -> String {
        let mut out = String::new();
        let mut at = 0;
        while at < bytes.len() {
            let byte = bytes[at];
            if self.needed == 0 {
                self.lead(byte, &mut out);
                at += 1;
                continue;
            }
            if byte < self.lower || byte > self.upper {
                self.reset();
                self.emit(0xfffd, &mut out);
                continue;
            }
            self.lower = 0x80;
            self.upper = 0xbf;
            self.point = (self.point << 6) | u32::from(byte & 0x3f);
            self.seen += 1;
            at += 1;
            if self.seen == self.needed {
                let point = self.point;
                self.reset();
                self.emit(point, &mut out);
            }
        }
        out
    }

    /// Turns a held sequence into U+FFFD; the decoder starts over.
    pub fn end(&mut self) -> String {
        let mut out = String::new();
        if self.needed != 0 {
            self.emit(0xfffd, &mut out);
        }
        self.reset();
        self.started = false;
        out
    }

    fn lead(&mut self, byte: u8, out: &mut String) {
        match byte {
            0x00..=0x7f => self.emit(u32::from(byte), out),
            0xc2..=0xdf => {
                self.needed = 1;
                self.point = u32::from(byte & 0x1f);
            }
            0xe0..=0xef => {
                if byte == 0xe0 {
                    self.lower = 0xa0;
                }
                if byte == 0xed {
                    self.upper = 0x9f;
                }
                self.needed = 2;
                self.point = u32::from(byte & 0xf);
            }
            0xf0..=0xf4 => {
                if byte == 0xf0 {
                    self.lower = 0x90;
                }
                if byte == 0xf4 {
                    self.upper = 0x8f;
                }
                self.needed = 3;
                self.point = u32::from(byte & 7);
            }
            _ => self.emit(0xfffd, out),
        }
    }

    /// Drops a byte order mark that starts the stream.
    fn emit(&mut self, point: u32, out: &mut String) {
        let first = !self.started;
        self.started = true;
        if !(first && point == 0xfeff) {
            out.push(char::from_u32(point).unwrap_or('\u{fffd}'));
        }
    }

    fn reset(&mut self) {
        self.point = 0;
        self.needed = 0;
        self.seen = 0;
        self.lower = 0x80;
        self.upper = 0xbf;
    }
}

impl Default for Utf8Decoder {
    fn default() -> Self {
        Self::new()
    }
}

const HEX: &[u8; 16] = b"0123456789ABCDEF";

/// Percent-encodes as `encodeURIComponent` does.
pub fn encode_path_segment(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for char in text.chars() {
        if char.is_ascii_alphanumeric() || "-_.!~*'()".contains(char) {
            out.push(char);
            continue;
        }
        let mut buffer = [0u8; 4];
        for byte in char.encode_utf8(&mut buffer).bytes() {
            out.push('%');
            out.push(char::from(HEX[usize::from(byte >> 4)]));
            out.push(char::from(HEX[usize::from(byte & 15)]));
        }
    }
    out
}

/// A character of category Cc: C0, DEL or C1.
pub fn has_control(text: &str) -> bool {
    text.chars()
        .any(|char| matches!(u32::from(char), 0..=0x1f | 0x7f..=0x9f))
}
