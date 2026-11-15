//! JSON the tool prints.

use nyaaai_contract::json::{Object, Value};
use nyaaai_operator::claude::session_cut::{CutPlan, CutRefused};

pub fn object(pairs: Vec<(&str, Value)>) -> Value {
    let mut held = Object::new();
    for (key, value) in pairs {
        held.insert(key, value);
    }
    Value::Object(held)
}

pub fn plan_json(plan: &CutPlan) -> Value {
    let preserved = plan.preserved.as_ref().map_or(Value::Null, |held| {
        object(vec![
            ("head", Value::from(held.head.as_str())),
            ("tail", Value::from(held.tail.as_str())),
            ("uuids", Value::from(held.uuids.clone())),
        ])
    });
    let from = plan.from.as_ref().map_or(Value::Null, |held| {
        object(vec![
            ("at", Value::from(held.at.clone())),
            ("text", Value::from(held.text.as_str())),
        ])
    });
    object(vec![
        ("ok", Value::Bool(true)),
        ("mode", Value::from(plan.mode.as_str())),
        ("headerLines", Value::from(plan.header_lines.clone())),
        ("keptLines", Value::from(plan.kept_lines.clone())),
        ("preserved", preserved),
        ("foldLines", Value::from(plan.fold_lines.clone())),
        ("keptBytes", Value::from(plan.kept_bytes)),
        ("droppedBytes", Value::from(plan.dropped_bytes)),
        ("droppedRows", Value::from(plan.dropped_rows)),
        ("from", from),
    ])
}

pub fn refused_json(refused: &CutRefused) -> Value {
    object(vec![
        ("ok", Value::Bool(false)),
        ("reason", Value::from(refused.reason.as_str())),
        ("detail", Value::from(refused.detail.as_str())),
    ])
}
