//! Segment flattening for chat content.

use nyaaai_contract::json::Value;
use nyaaai_contract::shapes::step::ThoughtPart;
use nyaaai_contract::text::trim_text;

/// Responses can hold any JSON value; `None` is no value.
pub fn normalize_response_value(value: &Value) -> Option<Value> {
    match value {
        Value::Array(items) => Some(Value::Array(
            items
                .iter()
                .map(|item| normalize_response_value(item).unwrap_or(Value::Null))
                .collect(),
        )),
        Value::Object(object) => match object.get("content") {
            Some(Value::Array(content)) => extract_text_from_segments(content).map(Value::String),
            _ => Some(value.clone()),
        },
        other => Some(other.clone()),
    }
}

/// Streaming segments can hold any entry.
pub fn extract_text_from_segments(segments: &[Value]) -> Option<String> {
    let mut parts: Vec<String> = Vec::new();
    for segment in segments {
        match segment {
            Value::String(text) => {
                if !trim_text(text).is_empty() {
                    parts.push(text.clone());
                }
            }
            Value::Object(object) => {
                if let Some(Value::String(text)) = object.get("text")
                    && !trim_text(text).is_empty()
                {
                    parts.push(text.clone());
                    continue;
                }
                let Some(Value::Array(nested)) = object.get("content") else {
                    continue;
                };
                if let Some(text) = extract_text_from_segments(nested)
                    && !trim_text(&text).is_empty()
                {
                    parts.push(text);
                }
            }
            _ => {}
        }
    }
    (!parts.is_empty()).then(|| parts.join(" "))
}

pub fn text_of(content: &Value) -> String {
    match content {
        Value::String(text) => text.clone(),
        Value::Array(items) => extract_text_from_segments(items).unwrap_or_default(),
        _ => String::new(),
    }
}

/// The visible text of a thought, in order.
pub fn thought_text(part: &ThoughtPart) -> String {
    part.segments
        .iter()
        .flatten()
        .map(|segment| segment.text.as_str())
        .collect()
}
