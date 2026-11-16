//! Readers of loose provider JSON fields.

use nyaaai_contract::json::{Object, Value};

use crate::native::{number_of, string_of};

/// A text field; anything else reads as empty.
pub fn text_at<'a>(object: &'a Object, key: &str) -> &'a str {
    object.get(key).map_or("", string_of)
}

/// A finite number field.
pub fn number_at(object: &Object, key: &str) -> Option<f64> {
    object.get(key).and_then(number_of)
}

/// A count or an amount: negative reads as mistyped, negative zero as zero.
pub fn amount_at(object: &Object, key: &str) -> Option<f64> {
    object.get(key).and_then(amount_of)
}

/// A finite number that is not negative.
pub fn amount_of(value: &Value) -> Option<f64> {
    number_of(value)
        .filter(|number| *number >= 0.0)
        .map(|number| number + 0.0)
}

/// Absent, null, false, zero and empty text are no error.
pub fn errored(raw: Option<&Value>) -> bool {
    match raw {
        None | Some(Value::Null | Value::Bool(false)) => false,
        Some(Value::Number(number)) => *number != 0.0,
        Some(Value::String(text)) => !text.is_empty(),
        Some(_) => true,
    }
}

pub fn object_at<'a>(object: &'a Object, key: &str) -> Option<&'a Object> {
    object.get(key).and_then(Value::as_object)
}

/// The elements of an array field; anything else reads as none.
pub fn items_at<'a>(object: &'a Object, key: &str) -> &'a [Value] {
    object.get(key).and_then(Value::as_array).unwrap_or(&[])
}

/// A text error is the message; a non-object is empty.
pub fn error_body(raw: Option<&Value>) -> Object {
    match raw {
        Some(Value::String(text)) => {
            let mut held = Object::new();
            held.insert("message", Value::from(text.as_str()));
            held
        }
        Some(Value::Object(object)) => object.clone(),
        _ => Object::new(),
    }
}

/// A non-empty text code is the type.
pub fn type_of(code: Option<&Value>) -> Option<String> {
    match code {
        Some(Value::String(text)) if !text.is_empty() => Some(text.clone()),
        _ => None,
    }
}

/// Text, else the fallback when empty.
pub fn text_or(text: &str, fallback: &str) -> String {
    if text.is_empty() { fallback } else { text }.to_owned()
}

/// An object from entries in order.
pub fn object_of(entries: Vec<(&str, Value)>) -> Value {
    let mut object = Object::new();
    for (key, value) in entries {
        object.insert(key, value);
    }
    Value::Object(object)
}
