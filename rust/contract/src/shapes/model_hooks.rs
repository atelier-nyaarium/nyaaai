//! Handwritten model shape hooks.

use crate::json::Value;
use crate::text::trim_text;

pub fn trimmed(raw: &Value) -> Value {
    match raw {
        Value::String(text) => Value::from(trim_text(text)),
        other => other.clone(),
    }
}
