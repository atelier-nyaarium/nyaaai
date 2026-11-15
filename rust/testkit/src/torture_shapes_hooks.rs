//! Handwritten torture shape hooks.

use nyaaai_contract::js::trim;
use nyaaai_contract::json::Value;

use super::torture_shapes::Span;

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
        Value::String(text) => Value::from(trim(text)),
        other => other.clone(),
    }
}

pub fn ordered(span: &Span) -> bool {
    span.from <= span.to
}
