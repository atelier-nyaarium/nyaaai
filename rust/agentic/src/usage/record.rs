//! A day file's lines: building, writing, reading, and the day a line belongs to.

use nyaaai_contract::json::{Object, Value, stringify_json};
use nyaaai_contract::shape::{Refused, Shape};
use nyaaai_contract::shapes::usage::{
    CallRecord, GenerationStats, MeteredCall, StatsLine, StatsRecord, UsageLine,
};
use nyaaai_contract::text::{ascii_lower, trim_text};
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

/// The local day of `at_ms`; none outside years 0000 to 9999.
pub fn day_of(at_ms: i64, clock: &dyn Clock) -> Option<String> {
    day_key(at_ms, clock.utc_offset_minutes(at_ms))
}

/// A path part with the separator before it; none for the first.
struct Segment<'a> {
    sep: &'a str,
    seg: &'a str,
}

fn segments_of(text: &str) -> Vec<Segment<'_>> {
    let mut out = Vec::new();
    let mut sep = "";
    let mut start = 0;
    for (at, char) in text.char_indices() {
        if char == '/' || char == '\\' {
            out.push(Segment {
                sep,
                seg: &text[start..at],
            });
            sep = &text[at..at + 1];
            start = at + 1;
        }
    }
    out.push(Segment {
        sep,
        seg: &text[start..],
    });
    out
}

/// A letter and a colon first.
fn has_drive(text: &str) -> bool {
    let mut chars = text.chars();
    matches!(chars.next(), Some(first) if first.is_ascii_alphabetic()) && chars.next() == Some(':')
}

/// `.` and `..` segments resolved, never past the root or the drive.
fn resolved(text: &str) -> String {
    let parts = segments_of(text);
    if !parts.iter().any(|part| part.seg == "." || part.seg == "..") {
        return text.to_owned();
    }
    let first = parts.first().map_or("", |part| part.seg);
    let kept = usize::from(first.is_empty() || (first.chars().count() == 2 && has_drive(first)));
    let mut out: Vec<Segment<'_>> = Vec::new();
    for part in parts {
        if part.seg == "." {
            continue;
        }
        if part.seg == ".." {
            if out.len() > kept {
                out.pop();
            }
            continue;
        }
        out.push(if out.is_empty() {
            Segment {
                sep: "",
                seg: part.seg,
            }
        } else {
            part
        });
    }
    out.iter()
        .map(|part| format!("{}{}", part.sep, part.seg))
        .collect()
}

fn plain_path(text: &str) -> String {
    without_separators(&resolved(without_separators(text))).to_owned()
}

/// A folder path with home as `~` and a trailing separator; none if blank.
pub fn app_of(folder: &str, home: &str) -> Option<String> {
    if trim_text(folder).is_empty() {
        return None;
    }
    let separator = if folder.contains('\\') && !folder.contains('/') {
        '\\'
    } else {
        '/'
    };
    let trimmed = plain_path(folder);
    let home_trimmed = plain_path(home);
    let fold = has_drive(&trimmed) && has_drive(&home_trimmed);
    let (held, held_home) = if fold {
        (ascii_lower(&trimmed), ascii_lower(&home_trimmed))
    } else {
        (trimmed.clone(), home_trimmed.clone())
    };
    let under_home = !home_trimmed.is_empty()
        && (held == held_home
            || held
                .strip_prefix(held_home.as_str())
                .is_some_and(|rest| rest.starts_with(separator)));
    Some(if under_home {
        format!("~{}{separator}", &trimmed[home_trimmed.len()..])
    } else {
        format!("{trimmed}{separator}")
    })
}

fn field<'a>(object: Option<&'a Object>, key: &str) -> Option<&'a Value> {
    object.and_then(|held| held.get(key))
}

fn same(object: Option<&Object>, key: &str) -> Option<Value> {
    field(object, key).cloned()
}

/// A call as its day file keeps it; fields may be missing, null or mistyped, else refused.
pub fn call_record_of(call: &Value, id: &str, home: &str) -> Result<CallRecord, Refused> {
    let call = call.as_object();
    let tag = field(call, "tag").and_then(Value::as_object);
    let usage = field(call, "usage").and_then(Value::as_object);
    let costed = field(call, "cost").is_some_and(|held| !held.is_null());
    let cost = field(call, "cost").and_then(Value::as_object);
    let usd = field(cost, "usd").and_then(Value::as_f64);
    let priced = usd.is_some_and(f64::is_finite);
    let estimated = field(cost, "estimated").is_some_and(truthy) || !priced;
    let folder = field(tag, "folder")
        .and_then(Value::as_str)
        .and_then(|folder| app_of(folder, home))
        .map(|app| Value::from(app.as_str()));
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
            costed.then(|| Value::Number(if priced { usd.unwrap_or(0.0) } else { 0.0 })),
        ),
        ("accountCost", same(usage, "cost")),
        ("upstreamCost", same(usage, "upstreamCost")),
        ("byok", same(usage, "byok")),
        ("billingSource", same(usage, "billingSource")),
        (
            "estimated",
            (costed && estimated).then_some(Value::Bool(true)),
        ),
        (
            "unpriced",
            if costed { same(cost, "unpriced") } else { None },
        ),
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

/// A generation's stats as its day file keeps them; refused when the shape misfits.
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

/// A day file's lines in order; unparseable text is dropped, blank lines not counted.
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
