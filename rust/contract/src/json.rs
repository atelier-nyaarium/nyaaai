//! ECMAScript-compatible JSON.

use std::collections::{BTreeMap, HashMap};
use std::fmt::Write as _;

use crate::order::compare_text;

/// Maximum accepted container depth.
pub const MAX_DEPTH: usize = 1024;

const INDEXED_FROM: usize = 16;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Value>),
    Object(Object),
}

/// ECMAScript key order: indices first, then insertion.
#[derive(Debug, Clone)]
pub struct Record<T> {
    /// Numeric-indexed keys in order.
    indexed: BTreeMap<u32, (String, T)>,
    /// Other keys, insertion order.
    named: Vec<(String, T)>,
    lookup: Option<HashMap<String, usize>>,
}

pub type Object = Record<Value>;

impl<T> Default for Record<T> {
    fn default() -> Self {
        Self {
            indexed: BTreeMap::new(),
            named: Vec::new(),
            lookup: None,
        }
    }
}

impl<T: PartialEq> PartialEq for Record<T> {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.iter().eq(other.iter())
    }
}

impl<T> Record<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.indexed.len() + self.named.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get(&self, key: &str) -> Option<&T> {
        match array_index(key) {
            Some(index) => self.indexed.get(&index).map(|(_, value)| value),
            None => self.position(key).map(|at| &self.named[at].1),
        }
    }

    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.iter().map(|(key, _)| key)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &T)> {
        self.indexed
            .values()
            .chain(self.named.iter())
            .map(|(key, value)| (key.as_str(), value))
    }

    pub fn values(&self) -> impl Iterator<Item = &T> {
        self.iter().map(|(_, value)| value)
    }

    /// Repeated keys preserve position.
    pub fn insert(&mut self, key: impl Into<String>, value: T) {
        let key = key.into();
        if let Some(index) = array_index(&key) {
            self.indexed.insert(index, (key, value));
            return;
        }
        if let Some(at) = self.position(&key) {
            self.named[at].1 = value;
            return;
        }
        if let Some(lookup) = &mut self.lookup {
            lookup.insert(key.clone(), self.named.len());
        }
        self.named.push((key, value));
        if self.lookup.is_none() && self.named.len() >= INDEXED_FROM {
            self.lookup = Some(
                self.named
                    .iter()
                    .enumerate()
                    .map(|(at, (held, _))| (held.clone(), at))
                    .collect(),
            );
        }
    }

    /// Omits absent values.
    pub fn insert_some(&mut self, key: impl Into<String>, value: Option<T>) {
        if let Some(value) = value {
            self.insert(key, value);
        }
    }

    fn position(&self, key: &str) -> Option<usize> {
        match &self.lookup {
            Some(lookup) => lookup.get(key).copied(),
            None => self.named.iter().position(|(held, _)| held == key),
        }
    }
}

impl Value {
    pub fn get(&self, key: &str) -> Option<&Value> {
        match self {
            Value::Object(object) => object.get(key),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(text) => Some(text),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Number(number) => Some(*number),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(held) => Some(*held),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&[Value]> {
        match self {
            Value::Array(items) => Some(items),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&Object> {
        match self {
            Value::Object(object) => Some(object),
            _ => None,
        }
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }
}

impl From<&str> for Value {
    fn from(text: &str) -> Self {
        Value::String(text.to_owned())
    }
}

impl From<String> for Value {
    fn from(text: String) -> Self {
        Value::String(text)
    }
}

impl From<bool> for Value {
    fn from(held: bool) -> Self {
        Value::Bool(held)
    }
}

impl From<f64> for Value {
    fn from(number: f64) -> Self {
        Value::Number(number)
    }
}

impl From<usize> for Value {
    fn from(number: usize) -> Self {
        Value::Number(number as f64)
    }
}

impl From<Object> for Value {
    fn from(object: Object) -> Self {
        Value::Object(object)
    }
}

impl<T: Into<Value>> From<Vec<T>> for Value {
    fn from(items: Vec<T>) -> Self {
        Value::Array(items.into_iter().map(Into::into).collect())
    }
}

impl<T: Into<Value>> From<Option<T>> for Value {
    fn from(held: Option<T>) -> Self {
        held.map_or(Value::Null, Into::into)
    }
}

/// No leading zeros; below `u32::MAX`.
fn array_index(key: &str) -> Option<u32> {
    let bytes = key.as_bytes();
    if bytes.is_empty() || bytes.len() > 10 || !bytes.iter().all(u8::is_ascii_digit) {
        return None;
    }
    if bytes[0] == b'0' && bytes.len() > 1 {
        return None;
    }
    key.parse::<u64>()
        .ok()
        .filter(|index| *index < u64::from(u32::MAX))
        .map(|index| index as u32)
}

/// ECMAScript parsing semantics.
pub fn parse_json(text: &str) -> Option<Value> {
    let mut reader = Reader {
        bytes: text.as_bytes(),
        at: 0,
    };
    reader.skip_space();
    let value = reader.value(0)?;
    reader.skip_space();
    (reader.at == reader.bytes.len()).then_some(value)
}

struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl Reader<'_> {
    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.at).copied()
    }

    fn skip_space(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.at += 1;
        }
    }

    fn eat(&mut self, word: &[u8]) -> Option<()> {
        self.bytes[self.at..]
            .starts_with(word)
            .then(|| self.at += word.len())
    }

    fn value(&mut self, depth: usize) -> Option<Value> {
        match self.peek()? {
            b'{' => self.object(depth + 1),
            b'[' => self.array(depth + 1),
            b'"' => self.string().map(Value::String),
            b't' => self.eat(b"true").map(|()| Value::Bool(true)),
            b'f' => self.eat(b"false").map(|()| Value::Bool(false)),
            b'n' => self.eat(b"null").map(|()| Value::Null),
            _ => self.number(),
        }
    }

    fn object(&mut self, depth: usize) -> Option<Value> {
        if depth > MAX_DEPTH {
            return None;
        }
        self.at += 1;
        let mut object = Object::new();
        self.skip_space();
        if self.peek()? == b'}' {
            self.at += 1;
            return Some(Value::Object(object));
        }
        loop {
            self.skip_space();
            if self.peek()? != b'"' {
                return None;
            }
            let key = self.string()?;
            self.skip_space();
            self.eat(b":")?;
            self.skip_space();
            let value = self.value(depth)?;
            object.insert(key, value);
            self.skip_space();
            match self.peek()? {
                b',' => self.at += 1,
                b'}' => {
                    self.at += 1;
                    return Some(Value::Object(object));
                }
                _ => return None,
            }
        }
    }

    fn array(&mut self, depth: usize) -> Option<Value> {
        if depth > MAX_DEPTH {
            return None;
        }
        self.at += 1;
        let mut items = Vec::new();
        self.skip_space();
        if self.peek()? == b']' {
            self.at += 1;
            return Some(Value::Array(items));
        }
        loop {
            self.skip_space();
            items.push(self.value(depth)?);
            self.skip_space();
            match self.peek()? {
                b',' => self.at += 1,
                b']' => {
                    self.at += 1;
                    return Some(Value::Array(items));
                }
                _ => return None,
            }
        }
    }

    fn string(&mut self) -> Option<String> {
        self.at += 1;
        let mut text = String::new();
        loop {
            let from = self.at;
            while !matches!(self.peek()?, b'"' | b'\\' | 0..=0x1F) {
                self.at += 1;
            }
            // ASCII delimiters preserve boundaries.
            text.push_str(std::str::from_utf8(&self.bytes[from..self.at]).ok()?);
            match self.peek()? {
                b'"' => {
                    self.at += 1;
                    return Some(text);
                }
                b'\\' => {
                    self.at += 1;
                    self.escape(&mut text)?;
                }
                _ => return None,
            }
        }
    }

    fn escape(&mut self, text: &mut String) -> Option<()> {
        let kind = self.peek()?;
        self.at += 1;
        match kind {
            b'"' => text.push('"'),
            b'\\' => text.push('\\'),
            b'/' => text.push('/'),
            b'b' => text.push('\u{8}'),
            b'f' => text.push('\u{C}'),
            b'n' => text.push('\n'),
            b'r' => text.push('\r'),
            b't' => text.push('\t'),
            b'u' => {
                let unit = self.hex4()?;
                text.push(self.scalar(unit));
            }
            _ => return None,
        }
        Some(())
    }

    /// Joins pairs; replaces lone surrogates.
    fn scalar(&mut self, unit: u32) -> char {
        if (0xD800..0xDC00).contains(&unit) && self.bytes[self.at..].starts_with(b"\\u") {
            let back = self.at;
            self.at += 2;
            match self.hex4() {
                Some(low) if (0xDC00..0xE000).contains(&low) => {
                    let joined = 0x10000 + ((unit - 0xD800) << 10) + (low - 0xDC00);
                    return char::from_u32(joined).unwrap_or(char::REPLACEMENT_CHARACTER);
                }
                _ => self.at = back,
            }
        }
        char::from_u32(unit).unwrap_or(char::REPLACEMENT_CHARACTER)
    }

    fn hex4(&mut self) -> Option<u32> {
        let digits = self.bytes.get(self.at..self.at + 4)?;
        let mut unit = 0;
        for digit in digits {
            unit = unit * 16 + char::from(*digit).to_digit(16)?;
        }
        self.at += 4;
        Some(unit)
    }

    fn number(&mut self) -> Option<Value> {
        let from = self.at;
        if self.peek() == Some(b'-') {
            self.at += 1;
        }
        match self.peek()? {
            b'0' => self.at += 1,
            b'1'..=b'9' => self.digits(),
            _ => return None,
        }
        if self.peek() == Some(b'.') {
            self.at += 1;
            self.some_digits()?;
        }
        if matches!(self.peek(), Some(b'e' | b'E')) {
            self.at += 1;
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.at += 1;
            }
            self.some_digits()?;
        }
        let spelled = std::str::from_utf8(&self.bytes[from..self.at]).ok()?;
        spelled.parse::<f64>().ok().map(Value::Number)
    }

    fn digits(&mut self) {
        while matches!(self.peek(), Some(b'0'..=b'9')) {
            self.at += 1;
        }
    }

    fn some_digits(&mut self) -> Option<()> {
        let from = self.at;
        self.digits();
        (self.at > from).then_some(())
    }
}

/// Compact ECMAScript JSON.
pub fn stringify_json(value: &Value) -> String {
    let mut out = String::new();
    write_value(&mut out, value);
    out
}

fn write_value(out: &mut String, value: &Value) {
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(held) => out.push_str(if *held { "true" } else { "false" }),
        Value::Number(number) if number.is_finite() => out.push_str(&number_to_string(*number)),
        Value::Number(_) => out.push_str("null"),
        Value::String(text) => write_string(out, text),
        Value::Array(items) => {
            out.push('[');
            for (at, item) in items.iter().enumerate() {
                if at > 0 {
                    out.push(',');
                }
                write_value(out, item);
            }
            out.push(']');
        }
        Value::Object(object) => {
            out.push('{');
            for (at, (key, held)) in object.iter().enumerate() {
                if at > 0 {
                    out.push(',');
                }
                write_string(out, key);
                out.push(':');
                write_value(out, held);
            }
            out.push('}');
        }
    }
}

/// Code-unit sorted JSON for fingerprints.
pub fn canonical_json(value: &Value) -> String {
    let mut out = String::new();
    write_canonical(&mut out, value);
    out
}

fn write_canonical(out: &mut String, value: &Value) {
    match value {
        Value::Array(items) => {
            out.push('[');
            for (at, item) in items.iter().enumerate() {
                if at > 0 {
                    out.push(',');
                }
                write_canonical(out, item);
            }
            out.push(']');
        }
        Value::Object(object) => {
            let mut fields: Vec<(&str, &Value)> = object.iter().collect();
            fields.sort_by(|(left, _), (right, _)| compare_text(left, right));
            out.push('{');
            for (at, (key, held)) in fields.into_iter().enumerate() {
                if at > 0 {
                    out.push(',');
                }
                write_string(out, key);
                out.push(':');
                write_canonical(out, held);
            }
            out.push('}');
        }
        scalar => write_value(out, scalar),
    }
}

/// ECMAScript string quoting.
pub fn quote_json(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    write_string(&mut out, text);
    out
}

fn write_string(out: &mut String, text: &str) {
    out.push('"');
    for char in text.chars() {
        match char {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{8}' => out.push_str("\\b"),
            '\u{C}' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            control if u32::from(control) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", u32::from(control));
            }
            other => out.push(other),
        }
    }
    out.push('"');
}

/// ECMAScript tie-to-even rounding.
fn even_on_a_tie(magnitude: f64, digits: String, exponent: i32) -> String {
    let Ok(held) = digits.parse::<u128>() else {
        return digits;
    };
    // magnitude = odd * 2^power.
    let bits = magnitude.to_bits();
    let biased = ((bits >> 52) & 0x7FF) as i32;
    let fraction = bits & ((1 << 52) - 1);
    let (mut odd, mut power) = match biased {
        0 => (fraction, -1074),
        _ => (fraction | (1 << 52), biased - 1075),
    };
    while odd.is_multiple_of(2) {
        odd >>= 1;
        power += 1;
    }
    // Detects a tie.
    let shift = digits.len() as i32 - 1 - exponent;
    let doubled: u128 = if (0..=24).contains(&shift) && power == -(shift + 1) {
        u128::from(odd) * 5u128.pow(shift as u32)
    } else if (-22..0).contains(&shift) && power == -shift - 1 {
        let scale = 5u64.pow(shift.unsigned_abs());
        if !odd.is_multiple_of(scale) {
            return digits;
        }
        u128::from(odd / scale)
    } else {
        return digits;
    };
    let below = (doubled - 1) / 2;
    let even = if below.is_multiple_of(2) {
        below
    } else {
        below + 1
    };
    let spelled = even.to_string();
    if even == held || spelled.len() != digits.len() {
        return digits;
    }
    // Keep round-tripping digits.
    let reads_back = format!("{spelled}e{}", exponent - (digits.len() as i32 - 1))
        .parse::<f64>()
        .is_ok_and(|read| read == magnitude);
    if reads_back { spelled } else { digits }
}

/// ECMAScript base-10 number text.
pub fn number_to_string(number: f64) -> String {
    if number.is_nan() {
        return "NaN".to_owned();
    }
    if number == 0.0 {
        return "0".to_owned();
    }
    if number.is_infinite() {
        return if number > 0.0 {
            "Infinity"
        } else {
            "-Infinity"
        }
        .to_owned();
    }
    // Find shortest round-trip digits.
    let magnitude = number.abs();
    let spelled = format!("{magnitude:e}");
    let (mantissa, exponent) = spelled.split_once('e').unwrap_or((&spelled, "0"));
    let exponent = exponent.parse::<i32>().unwrap_or(0);
    let digits: String = mantissa.chars().filter(char::is_ascii_digit).collect();
    let digits = even_on_a_tie(magnitude, digits, exponent);
    let count = digits.len() as i32;
    let point = exponent + 1;

    let mut out = String::new();
    if number < 0.0 {
        out.push('-');
    }
    if count <= point && point <= 21 {
        out.push_str(&digits);
        out.extend(std::iter::repeat_n('0', (point - count) as usize));
    } else if 0 < point && point <= 21 {
        out.push_str(&digits[..point as usize]);
        out.push('.');
        out.push_str(&digits[point as usize..]);
    } else if -6 < point && point <= 0 {
        out.push_str("0.");
        out.extend(std::iter::repeat_n('0', (-point) as usize));
        out.push_str(&digits);
    } else {
        let power = point - 1;
        out.push_str(&digits[..1]);
        if count > 1 {
            out.push('.');
            out.push_str(&digits[1..]);
        }
        out.push('e');
        out.push(if power < 0 { '-' } else { '+' });
        out.push_str(&power.abs().to_string());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn round(text: &str) -> Option<String> {
        parse_json(text).map(|value| stringify_json(&value))
    }

    #[test]
    fn orders_keys_as_ecmascript_does() {
        assert_eq!(
            round(r#"{"b":1,"10":2,"2":3,"a":4,"b":5,"01":6}"#).as_deref(),
            Some(r#"{"2":3,"10":2,"b":5,"a":4,"01":6}"#)
        );
    }

    #[test]
    fn spells_numbers_as_ecmascript_does() {
        let spelled = |text: &str| round(text).unwrap();
        assert_eq!(spelled("1e21"), "1e+21");
        assert_eq!(spelled("123456789012345680000"), "123456789012345680000");
        assert_eq!(spelled("0.000001"), "0.000001");
        assert_eq!(spelled("1e-7"), "1e-7");
        assert_eq!(spelled("-0"), "0");
        assert_eq!(spelled("1.50"), "1.5");
        assert_eq!(spelled("5e-324"), "5e-324");
        assert_eq!(spelled("1e999"), "null");
    }

    #[test]
    fn refuses_what_json_parse_refuses() {
        for text in [
            "",
            "01",
            "1.",
            "[1,]",
            "{\"a\":1,}",
            "\"a\nb\"",
            "nul",
            "1 2",
            "'a'",
            "\u{FEFF}1",
        ] {
            assert!(parse_json(text).is_none(), "{text:?}");
        }
    }

    #[test]
    fn replaces_a_lone_surrogate_and_joins_a_pair() {
        assert_eq!(
            round("[\"\\ud83d\\ude00\",\"\\ud83d\",\"\\ude00x\",\"\\ud83d\\u0041\"]").as_deref(),
            Some("[\"\u{1F600}\",\"\u{FFFD}\",\"\u{FFFD}x\",\"\u{FFFD}A\"]")
        );
    }

    #[test]
    fn refuses_nesting_past_the_cap() {
        let nested = |depth: usize| format!("{}{}", "[".repeat(depth), "]".repeat(depth));
        assert!(parse_json(&nested(MAX_DEPTH)).is_some());
        assert!(parse_json(&nested(MAX_DEPTH + 1)).is_none());
    }
}
