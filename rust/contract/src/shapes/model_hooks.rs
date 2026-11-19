//! Handwritten model shape hooks.

use super::model::Ability;
use crate::json::Value;
use crate::text::trim_text;

pub fn trimmed(raw: &Value) -> Value {
    match raw {
        Value::String(text) => Value::from(trim_text(text)),
        other => other.clone(),
    }
}

/// Each ability once.
pub fn distinct_abilities(held: &[Ability]) -> bool {
    held.iter()
        .enumerate()
        .all(|(at, ability)| !held[..at].contains(ability))
}
