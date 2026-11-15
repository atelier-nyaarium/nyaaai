//! Handwritten torture shape hooks.

use nyaaai_contract::json::Value;
use nyaaai_contract::text::trim_text;

use super::torture_shapes::{NotFive, Span};

pub fn rename_legacy(raw: &Value) -> Value {
    let Value::Object(held) = raw else {
        return raw.clone();
    };
    let mut held = held.clone();
    if held.get("id").is_none()
        && let Some(name) = held.get("name").cloned()
    {
        held.insert("id", name);
    }
    if let Some(Value::String(tag)) = held.get("tags").cloned() {
        held.insert("tags", Value::Array(vec![Value::String(tag)]));
    }
    Value::Object(held)
}

pub fn trimmed(raw: &Value) -> Value {
    match raw {
        Value::String(text) => Value::from(trim_text(text)),
        other => other.clone(),
    }
}

pub fn ordered(span: &Span) -> bool {
    span.from <= span.to
}

pub fn even(value: &i64) -> bool {
    value % 2 == 0
}

pub fn small(value: &i64) -> bool {
    *value < 100
}

pub fn ascending(range: &(i64, i64)) -> bool {
    range.0 <= range.1
}

pub fn not_five(value: &NotFive) -> bool {
    !matches!(value, NotFive::Integer(5))
}
