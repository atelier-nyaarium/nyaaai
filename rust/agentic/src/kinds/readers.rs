//! Readers of loose catalog JSON: a field of the wrong type is missing.

use nyaaai_contract::json::{Object, Value};

/// An object that is no list.
pub fn fields_of(value: &Value) -> Option<&Object> {
    value.as_object()
}

/// Own fields only.
pub fn field_of<'a>(fields: &'a Object, key: &str) -> Option<&'a Value> {
    fields.get(key)
}

pub fn text_field<'a>(fields: &'a Object, key: &str) -> Option<&'a str> {
    field_of(fields, key).and_then(Value::as_str)
}

/// Finite numbers only.
pub fn number_field(fields: &Object, key: &str) -> Option<f64> {
    field_of(fields, key)
        .and_then(Value::as_f64)
        .filter(|number| number.is_finite())
}

/// A length: a finite number above zero.
pub fn positive_field(fields: &Object, key: &str) -> Option<f64> {
    number_field(fields, key).filter(|value| *value > 0.0)
}

pub fn boolean_field(fields: &Object, key: &str) -> Option<bool> {
    field_of(fields, key).and_then(Value::as_bool)
}

/// A list of texts; one other item makes it missing.
pub fn texts_field(fields: &Object, key: &str) -> Option<Vec<String>> {
    field_of(fields, key)?
        .as_array()?
        .iter()
        .map(|item| item.as_str().map(str::to_owned))
        .collect()
}

/// A nested object, or none.
pub fn object_field<'a>(fields: &'a Object, key: &str) -> Option<&'a Object> {
    field_of(fields, key).and_then(fields_of)
}

/// The items of `{ data: [...] }`.
pub fn data_list(raw: &Value) -> Option<&[Value]> {
    field_of(fields_of(raw)?, "data")?.as_array()
}
