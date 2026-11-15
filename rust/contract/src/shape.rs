//! Contract shape runtime.

use std::cmp::Ordering;
use std::fmt;

use crate::json::{Object, Record, Value, parse_json};
use crate::pattern::Pattern;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathPart {
    Key(String),
    Index(usize),
}

pub type Path = Vec<PathPart>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refused {
    pub path: Path,
    pub message: &'static str,
    /// Marks an authored message.
    pub fixed: bool,
}

impl fmt::Display for Refused {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        out.write_str(self.message)?;
        for part in &self.path {
            match part {
                PathPart::Key(key) => write!(out, " .{key}")?,
                PathPart::Index(index) => write!(out, " [{index}]")?,
            }
        }
        Ok(())
    }
}

impl std::error::Error for Refused {}

pub trait Shape: Sized {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused>;

    fn encode(&self) -> Value;

    fn decode(value: &Value) -> Result<Self, Refused> {
        Self::decode_at(value, &mut Path::new())
    }

    fn decode_text(text: &str) -> Result<Self, Refused> {
        let value = parse_json(text).ok_or_else(|| refused(&Path::new(), "not JSON"))?;
        Self::decode(&value)
    }
}

/// Largest exact integer.
const SAFE: f64 = 9_007_199_254_740_991.0;

pub fn refused(at: &Path, message: &'static str) -> Refused {
    Refused {
        path: at.clone(),
        message,
        fixed: false,
    }
}

fn under<T>(
    at: &mut Path,
    part: PathPart,
    decode: impl FnOnce(&mut Path) -> Result<T, Refused>,
) -> Result<T, Refused> {
    at.push(part);
    let held = decode(at);
    at.pop();
    held
}

pub fn object<'a>(value: &'a Value, at: &Path) -> Result<&'a Object, Refused> {
    value
        .as_object()
        .ok_or_else(|| refused(at, "not an object"))
}

pub fn required<T>(
    object: &Object,
    key: &str,
    at: &mut Path,
    decode: impl FnOnce(&Value, &mut Path) -> Result<T, Refused>,
) -> Result<T, Refused> {
    under(at, PathPart::Key(key.to_owned()), |at| {
        let value = object.get(key).ok_or_else(|| refused(at, "absent"))?;
        decode(value, at)
    })
}

pub fn optional<T>(
    object: &Object,
    key: &str,
    at: &mut Path,
    decode: impl FnOnce(&Value, &mut Path) -> Result<T, Refused>,
) -> Result<Option<T>, Refused> {
    match object.get(key) {
        Some(value) => under(at, PathPart::Key(key.to_owned()), |at| decode(value, at)).map(Some),
        None => Ok(None),
    }
}

/// Default JSON text.
pub fn defaulted<T>(
    object: &Object,
    key: &str,
    at: &mut Path,
    fallback: &str,
    decode: impl FnOnce(&Value, &mut Path) -> Result<T, Refused>,
) -> Result<T, Refused> {
    under(at, PathPart::Key(key.to_owned()), |at| {
        match object.get(key) {
            Some(value) => decode(value, at),
            None => {
                let value =
                    parse_json(fallback).ok_or_else(|| refused(at, "a default is not JSON"))?;
                decode(&value, at)
            }
        }
    })
}

pub fn any(value: &Value, _at: &mut Path) -> Result<Value, Refused> {
    Ok(value.clone())
}

pub fn text(value: &Value, at: &mut Path) -> Result<String, Refused> {
    value
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| refused(at, "not text"))
}

/// Counts Unicode code points.
pub fn text_within(
    value: &Value,
    at: &mut Path,
    least: Option<usize>,
    most: Option<usize>,
) -> Result<String, Refused> {
    let held = text(value, at)?;
    let points = held.chars().count();
    if least.is_some_and(|least| points < least) {
        return Err(refused(at, "too short"));
    }
    if most.is_some_and(|most| points > most) {
        return Err(refused(at, "too long"));
    }
    Ok(held)
}

pub fn number(value: &Value, at: &mut Path) -> Result<f64, Refused> {
    value
        .as_f64()
        .filter(|held| held.is_finite())
        .ok_or_else(|| refused(at, "not a number"))
}

fn whole(value: &Value, at: &mut Path) -> Result<f64, Refused> {
    let held = number(value, at)?;
    if held.fract() != 0.0 {
        return Err(refused(at, "not an integer"));
    }
    if held.abs() > SAFE {
        return Err(refused(at, "past the safe integers"));
    }
    Ok(held)
}

pub fn integer(value: &Value, at: &mut Path) -> Result<i64, Refused> {
    whole(value, at).map(|held| held as i64)
}

pub fn natural(value: &Value, at: &mut Path) -> Result<u64, Refused> {
    let held = whole(value, at)?;
    if held < 0.0 {
        return Err(refused(at, "below zero"));
    }
    Ok(held as u64)
}

pub fn at_least<T: PartialOrd>(held: T, bound: T, at: &Path) -> Result<T, Refused> {
    if held >= bound {
        Ok(held)
    } else {
        Err(refused(at, "too small"))
    }
}

pub fn above<T: PartialOrd>(held: T, bound: T, at: &Path) -> Result<T, Refused> {
    if held > bound {
        Ok(held)
    } else {
        Err(refused(at, "too small"))
    }
}

pub fn at_most<T: PartialOrd>(held: T, bound: T, at: &Path) -> Result<T, Refused> {
    if held <= bound {
        Ok(held)
    } else {
        Err(refused(at, "too big"))
    }
}

pub fn below<T: PartialOrd>(held: T, bound: T, at: &Path) -> Result<T, Refused> {
    if held < bound {
        Ok(held)
    } else {
        Err(refused(at, "too big"))
    }
}

pub fn flag(value: &Value, at: &mut Path) -> Result<bool, Refused> {
    value.as_bool().ok_or_else(|| refused(at, "not a flag"))
}

pub fn null(value: &Value, at: &mut Path) -> Result<(), Refused> {
    if value.is_null() {
        Ok(())
    } else {
        Err(refused(at, "not null"))
    }
}

pub fn is_text(value: &Value, at: &mut Path, expected: &str) -> Result<(), Refused> {
    if value.as_str() == Some(expected) {
        Ok(())
    } else {
        Err(refused(at, "not the expected text"))
    }
}

pub fn is_number(value: &Value, at: &mut Path, expected: f64) -> Result<(), Refused> {
    if value.as_f64() == Some(expected) {
        Ok(())
    } else {
        Err(refused(at, "not the expected number"))
    }
}

pub fn is_flag(value: &Value, at: &mut Path, expected: bool) -> Result<(), Refused> {
    if value.as_bool() == Some(expected) {
        Ok(())
    } else {
        Err(refused(at, "not the expected flag"))
    }
}

pub fn list<T>(
    value: &Value,
    at: &mut Path,
    decode: impl Fn(&Value, &mut Path) -> Result<T, Refused>,
) -> Result<Vec<T>, Refused> {
    list_within(value, at, None, None, decode)
}

/// Excess before items; shortage after.
pub fn list_within<T>(
    value: &Value,
    at: &mut Path,
    least: Option<usize>,
    most: Option<usize>,
    decode: impl Fn(&Value, &mut Path) -> Result<T, Refused>,
) -> Result<Vec<T>, Refused> {
    let items = value.as_array().ok_or_else(|| refused(at, "not a list"))?;
    if most.is_some_and(|most| items.len() > most) {
        return Err(refused(at, "too many"));
    }
    let held = items
        .iter()
        .enumerate()
        .map(|(index, item)| under(at, PathPart::Index(index), |at| decode(item, at)))
        .collect::<Result<Vec<T>, Refused>>()?;
    if least.is_some_and(|least| held.len() < least) {
        return Err(refused(at, "too few"));
    }
    Ok(held)
}

/// Requires exactly `count` items.
pub fn tuple<'a>(value: &'a Value, at: &Path, count: usize) -> Result<&'a [Value], Refused> {
    let items = value.as_array().ok_or_else(|| refused(at, "not a list"))?;
    match items.len().cmp(&count) {
        Ordering::Less => Err(refused(at, "too few")),
        Ordering::Greater => Err(refused(at, "too many")),
        Ordering::Equal => Ok(items),
    }
}

pub fn place<T>(
    items: &[Value],
    index: usize,
    at: &mut Path,
    decode: impl FnOnce(&Value, &mut Path) -> Result<T, Refused>,
) -> Result<T, Refused> {
    under(at, PathPart::Index(index), |at| {
        let item = items.get(index).ok_or_else(|| refused(at, "absent"))?;
        decode(item, at)
    })
}

/// Drops JavaScript's "__proto__" key.
pub fn record<T>(
    value: &Value,
    at: &mut Path,
    decode: impl Fn(&Value, &mut Path) -> Result<T, Refused>,
) -> Result<Record<T>, Refused> {
    let mut held = Record::new();
    for (key, item) in object(value, at)?.iter() {
        if key == "__proto__" {
            continue;
        }
        let item = under(at, PathPart::Key(key.to_owned()), |at| decode(item, at))?;
        held.insert(key, item);
    }
    Ok(held)
}

pub fn record_value<T>(held: &Record<T>, encode: impl Fn(&T) -> Value) -> Value {
    let mut object = Object::new();
    for (key, item) in held.iter() {
        object.insert(key, encode(item));
    }
    Value::Object(object)
}

pub fn nullable<T>(
    value: &Value,
    at: &mut Path,
    decode: impl FnOnce(&Value, &mut Path) -> Result<T, Refused>,
) -> Result<Option<T>, Refused> {
    if value.is_null() {
        Ok(None)
    } else {
        decode(value, at).map(Some)
    }
}

pub fn tag<'a>(object: &'a Object, key: &str, at: &Path) -> Result<&'a str, Refused> {
    object
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| no_branch(at, key))
}

pub fn no_branch(at: &Path, key: &str) -> Refused {
    let mut path = at.clone();
    path.push(PathPart::Key(key.to_owned()));
    Refused {
        path,
        message: "names no branch",
        fixed: false,
    }
}

/// Authored messages override fallback.
fn checked<T>(
    verdict: bool,
    held: T,
    at: &Path,
    message: Option<&'static str>,
    plain: &'static str,
) -> Result<T, Refused> {
    if verdict {
        return Ok(held);
    }
    Err(Refused {
        path: at.clone(),
        message: message.unwrap_or(plain),
        fixed: message.is_some(),
    })
}

pub fn holds<T>(
    verdict: bool,
    held: T,
    at: &Path,
    message: Option<&'static str>,
) -> Result<T, Refused> {
    checked(verdict, held, at, message, "refined away")
}

/// Unsupported patterns match nothing.
pub fn matches(
    pattern: Option<&Pattern>,
    held: String,
    at: &Path,
    message: Option<&'static str>,
) -> Result<String, Refused> {
    let found = pattern.is_some_and(|pattern| pattern.test(&held));
    checked(found, held, at, message, "no match")
}

fn bare(part: &str) -> Option<&str> {
    if part.is_empty() || !part.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let trimmed = part.trim_start_matches('0');
    Some(if trimmed.is_empty() { "0" } else { trimmed })
}

fn version_parts(version: &str) -> Option<(&str, &str)> {
    let mut parts = version.split('.');
    let (major, minor) = (bare(parts.next()?)?, bare(parts.next()?)?);
    bare(parts.next()?)?;
    parts.next().is_none().then_some((major, minor))
}

/// Same major; pre-1.0 also needs the same minor.
pub fn compatible_versions(left: &str, right: &str) -> bool {
    match (version_parts(left), version_parts(right)) {
        (Some((left_major, left_minor)), Some((right_major, right_minor))) => {
            left_major == right_major && (left_major != "0" || left_minor == right_minor)
        }
        _ => false,
    }
}
