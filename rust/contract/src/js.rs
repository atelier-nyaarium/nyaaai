//! Ports use JavaScript stand-ins.

use crate::json::{Value, number_to_string};

/// `None` maps to `undefined`.
pub fn to_js_string(value: Option<&Value>) -> String {
    match value {
        None => "undefined".to_owned(),
        Some(Value::Null) => "null".to_owned(),
        Some(Value::Bool(held)) => held.to_string(),
        Some(Value::Number(number)) => number_to_string(*number),
        Some(Value::String(text)) => text.clone(),
        Some(Value::Array(items)) => items
            .iter()
            .map(|item| match item {
                Value::Null => String::new(),
                other => to_js_string(Some(other)),
            })
            .collect::<Vec<_>>()
            .join(","),
        Some(Value::Object(_)) => "[object Object]".to_owned(),
    }
}

/// Compound values compare unequal.
pub fn strict_equal(left: Option<&Value>, right: Option<&Value>) -> bool {
    match (left, right) {
        (None, None) => true,
        (Some(Value::Null), Some(Value::Null)) => true,
        (Some(Value::Bool(a)), Some(Value::Bool(b))) => a == b,
        (Some(Value::Number(a)), Some(Value::Number(b))) => a == b,
        (Some(Value::String(a)), Some(Value::String(b))) => a == b,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compares_as_triple_equals_does() {
        let object = crate::json::parse_json("{}").unwrap();
        assert!(strict_equal(None, None));
        assert!(strict_equal(
            Some(&Value::Number(0.0)),
            Some(&Value::Number(-0.0))
        ));
        assert!(!strict_equal(Some(&Value::Null), None));
        assert!(!strict_equal(Some(&object), Some(&object)));
        assert!(!strict_equal(
            Some(&Value::from("1")),
            Some(&Value::Number(1.0))
        ));
    }
}
