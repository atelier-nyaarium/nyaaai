//! Usage record cases.

use nyaaai_agentic::usage::record::{
    app_of, call_record_of, day_of, line_text, lines_of, stats_line_of,
};
use nyaaai_contract::json::{Object, Value};
use nyaaai_contract::shape::{PathPart, Refused};
use nyaaai_contract::shapes::usage::{CallLine, UsageLine};
use nyaaai_contract::time::{OffsetFrom, SteppedClock};
use nyaaai_testkit::cases::Outcome;

use super::shapes::*;

/// The path of a refusal, joined by dots.
fn path_text(refused: &Refused) -> String {
    refused
        .path
        .iter()
        .map(|part| match part {
            PathPart::Key(key) => key.clone(),
            PathPart::Index(index) => index.to_string(),
        })
        .collect::<Vec<_>>()
        .join(".")
}

/// A copy of `value` with `to` set where the path says; a dot goes one level in.
fn set_at(value: &Value, path: &[&str], to: f64) -> Value {
    let Some((first, rest)) = path.split_first() else {
        return Value::Number(to);
    };
    let Value::Object(object) = value else {
        return value.clone();
    };
    let mut copy = Object::new();
    let mut found = false;
    for (key, held) in object.iter() {
        if key == *first {
            found = true;
            copy.insert(key, set_at(held, rest, to));
        } else {
            copy.insert(key, held.clone());
        }
    }
    if !found && rest.is_empty() {
        copy.insert(*first, Value::Number(to));
    }
    Value::Object(copy)
}

fn with_non_finite(value: &Value, set: &[(String, f64)]) -> Value {
    let mut held = value.clone();
    for (path, to) in set {
        let keys: Vec<&str> = path.split('.').collect();
        held = set_at(&held, &keys, *to);
    }
    held
}

fn float_of(held: &UsageRecordInputNonFiniteItemIs) -> f64 {
    match held {
        UsageRecordInputNonFiniteItemIs::NotANumber => f64::NAN,
        UsageRecordInputNonFiniteItemIs::PlusInfinity => f64::INFINITY,
        UsageRecordInputNonFiniteItemIs::MinusInfinity => f64::NEG_INFINITY,
    }
}

fn stats_float_of(held: &UsageStatsInputNonFiniteItemIs) -> f64 {
    match held {
        UsageStatsInputNonFiniteItemIs::NotANumber => f64::NAN,
        UsageStatsInputNonFiniteItemIs::PlusInfinity => f64::INFINITY,
        UsageStatsInputNonFiniteItemIs::MinusInfinity => f64::NEG_INFINITY,
    }
}

fn clock_of(offsets: impl Iterator<Item = (i64, i64)>) -> SteppedClock {
    SteppedClock::with_offsets(
        0,
        0,
        offsets
            .map(|(from_ms, minutes)| OffsetFrom { from_ms, minutes })
            .collect(),
    )
}

pub fn record(input: UsageRecordInput) -> Outcome<UsageRecordResult> {
    let set: Vec<(String, f64)> = input
        .non_finite
        .iter()
        .map(|each| (each.path.clone(), float_of(&each.is)))
        .collect();
    let call = with_non_finite(&input.call, &set);
    let clock = clock_of(
        input
            .offsets
            .iter()
            .map(|each| (each.from_ms, each.minutes)),
    );
    let day = day_of(input.now_ms, &clock);
    Ok(match call_record_of(&call, &input.id, &input.home) {
        Err(refused) => UsageRecordResult {
            record: None,
            refused_at: Some(path_text(&refused)),
            line: None,
            day,
        },
        Ok(record) => UsageRecordResult {
            line: Some(line_text(&UsageLine::CallLine(CallLine {
                call: record.clone(),
            }))),
            record: Some(record),
            refused_at: None,
            day,
        },
    })
}

pub fn stats(input: UsageStatsInput) -> Outcome<UsageStatsResult> {
    let set: Vec<(String, f64)> = input
        .non_finite
        .iter()
        .map(|each| (each.path.clone(), stats_float_of(&each.is)))
        .collect();
    let stats = with_non_finite(&input.stats, &set);
    Ok(match stats_line_of(&input.id, &stats) {
        Err(refused) => UsageStatsResult {
            line: None,
            refused_at: Some(path_text(&refused)),
            text: None,
        },
        Ok(line) => UsageStatsResult {
            text: Some(line_text(&UsageLine::StatsLine(line.clone()))),
            line: Some(line),
            refused_at: None,
        },
    })
}

pub fn lines(input: UsageLinesInput) -> Outcome<UsageLinesResult> {
    let read = lines_of(&input.text);
    Ok(UsageLinesResult {
        lines: read.lines,
        dropped: read.dropped as i64,
    })
}

pub fn app(input: UsageAppInput) -> Outcome<UsageAppResult> {
    Ok(UsageAppResult {
        apps: input
            .rows
            .iter()
            .map(|row| app_of(&row.folder, &row.home))
            .collect(),
    })
}

pub fn day(input: UsageDayInput) -> Outcome<UsageDayResult> {
    let clock = clock_of(
        input
            .offsets
            .iter()
            .map(|each| (each.from_ms, each.minutes)),
    );
    Ok(UsageDayResult {
        days: input
            .instants
            .iter()
            .map(|at| day_of(*at, &clock))
            .collect(),
    })
}
