//! A day file's lines: building, writing, reading, and the day a line belongs to.

use nyaaai_contract::json::{Object, Value, stringify_json};
use nyaaai_contract::shape::{Refused, Shape};
use nyaaai_contract::shapes::usage::{
    CallRecord, GenerationStats, MeteredCall, StatsLine, StatsRecord, UsageLine,
};
use nyaaai_contract::text::trim_text;
use nyaaai_contract::time::{Clock, day_key};

/// What a day file's text held.
#[derive(Debug, Clone, PartialEq)]
pub struct LinesRead {
    pub lines: Vec<UsageLine>,
    /// Lines with text that did not parse or fit.
    pub dropped: usize,
}

/// A value the way a script truthiness reads it.
fn truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(held) => *held,
        Value::Number(held) => *held != 0.0 && !held.is_nan(),
        Value::String(held) => !held.is_empty(),
        Value::Array(_) | Value::Object(_) => true,
    }
}

/// Fields that hold a value: no null, no NaN.
fn present(fields: Vec<(&str, Option<Value>)>) -> Value {
    let mut object = Object::new();
    for (key, value) in fields {
        match value {
            None | Some(Value::Null) => {}
            Some(Value::Number(held)) if held.is_nan() => {}
            Some(held) => object.insert(key, held),
        }
    }
    Value::Object(object)
}

fn without_separators(text: &str) -> &str {
    text.trim_end_matches(['/', '\\'])
}

/// The local calendar day of `at_ms` in the clock's zone; none outside years 0000 to 9999.
pub fn day_of(at_ms: i64, clock: &dyn Clock) -> Option<String> {
    day_key(at_ms, clock.utc_offset_minutes(at_ms))
}

/// A folder path with home as `~` and a closing separator.
pub fn app_of(folder: &str, home: &str) -> String {
    let separator = if folder.contains('\\') && !folder.contains('/') {
        '\\'
    } else {
        '/'
    };
    let trimmed = without_separators(folder);
    let home_trimmed = without_separators(home);
    let under_home = !home_trimmed.is_empty()
        && (trimmed == home_trimmed
            || trimmed
                .strip_prefix(home_trimmed)
                .is_some_and(|rest| rest.starts_with(separator)));
    if under_home {
        format!("~{}{separator}", &trimmed[home_trimmed.len()..])
    } else {
        format!("{trimmed}{separator}")
    }
}

fn field<'a>(object: Option<&'a Object>, key: &str) -> Option<&'a Value> {
    object.and_then(|held| held.get(key))
}

fn same(object: Option<&Object>, key: &str) -> Option<Value> {
    field(object, key).cloned()
}

/// A call as its day file keeps it; a refusal when the shape does not fit. The call is read as
/// the meter may hand it: a field may be missing, null or of another type.
pub fn call_record_of(call: &Value, id: &str, home: &str) -> Result<CallRecord, Refused> {
    let call = call.as_object();
    let tag = field(call, "tag").and_then(Value::as_object);
    let usage = field(call, "usage").and_then(Value::as_object);
    let cost = field(call, "cost").and_then(Value::as_object);
    let usd = field(cost, "usd").and_then(Value::as_f64);
    let priced = usd.is_some_and(f64::is_finite);
    let estimated = field(cost, "estimated").is_some_and(truthy) || !priced;
    let folder = field(tag, "folder")
        .and_then(Value::as_str)
        .map(|folder| Value::from(app_of(folder, home).as_str()));
    CallRecord::decode(&present(vec![
        ("id", Some(Value::from(id))),
        ("at", same(call, "at")),
        ("feature", same(tag, "feature")),
        ("lane", same(tag, "lane")),
        ("model", same(call, "model")),
        ("entry", same(call, "entry")),
        ("label", same(call, "label")),
        ("app", folder),
        ("ms", same(call, "ms")),
        ("first", same(call, "firstTokenMs")),
        ("in", same(usage, "input")),
        ("out", same(usage, "output")),
        ("cached", same(usage, "cached")),
        ("cacheWrite", same(usage, "cacheWrite")),
        ("reasoning", same(usage, "reasoning")),
        (
            "cost",
            Some(if priced {
                Value::Number(usd.unwrap_or(0.0))
            } else {
                Value::Number(0.0)
            }),
        ),
        ("accountCost", same(usage, "cost")),
        ("upstreamCost", same(usage, "upstreamCost")),
        ("byok", same(usage, "byok")),
        ("billingSource", same(usage, "billingSource")),
        ("estimated", estimated.then_some(Value::Bool(true))),
        ("unpriced", same(cost, "unpriced")),
        ("provider", same(call, "provider")),
        ("gen", same(call, "generationId")),
        ("finish", same(call, "finishReason")),
        ("failure", same(call, "failure")),
        ("cancelled", same(call, "cancelled")),
    ]))
}

/// The same for a call the shapes already hold.
pub fn call_record_for(call: &MeteredCall, id: &str, home: &str) -> Result<CallRecord, Refused> {
    call_record_of(&call.encode(), id, home)
}

/// A generation's stats as its day file keeps them; a refusal when the shape does not fit.
pub fn stats_line_of(id: &str, stats: &Value) -> Result<StatsLine, Refused> {
    let fields = stats
        .as_object()
        .map(|held| {
            held.iter()
                .map(|(key, value)| (key, Some(value.clone())))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let of = GenerationStats::decode(&present(fields))?;
    Ok(StatsLine {
        stats: StatsRecord {
            id: id.to_owned(),
            of,
        },
    })
}

/// The line as written to a day file, with its line feed.
pub fn line_text(line: &UsageLine) -> String {
    format!("{}\n", stringify_json(&line.encode()))
}

/// A day file's lines in order; text that does not parse or fit is dropped, blank lines are not
/// counted.
pub fn lines_of(text: &str) -> LinesRead {
    let mut lines = Vec::new();
    let mut dropped = 0;
    for raw in text.split('\n') {
        if trim_text(raw).is_empty() {
            continue;
        }
        match UsageLine::decode_text(raw) {
            Ok(line) => lines.push(line),
            Err(_) => dropped += 1,
        }
    }
    LinesRead { lines, dropped }
}
