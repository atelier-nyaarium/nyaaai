//! JavaScript stand-ins. Ports call these, never their own copies.

use crate::json::{Value, number_to_string};

/// `String(value)` coercion. `None` is `undefined`.
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

/// JavaScript strict equality treats objects and arrays as unequal.
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

/// JavaScript whitespace.
pub fn is_space(char: char) -> bool {
    matches!(
        char,
        '\t' | '\n' | '\u{B}' | '\u{C}' | '\r' | ' ' | '\u{A0}' | '\u{1680}' | '\u{2000}'
            ..='\u{200A}'
                | '\u{2028}'
                | '\u{2029}'
                | '\u{202F}'
                | '\u{205F}'
                | '\u{3000}'
                | '\u{FEFF}'
    )
}

pub fn trim(text: &str) -> &str {
    text.trim_matches(is_space)
}

/// Collapses JavaScript whitespace.
pub fn collapse_space(text: &str) -> String {
    text.split(is_space)
        .filter(|word| !word.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trims_and_collapses_what_javascript_calls_space() {
        let text = "\u{FEFF}\u{A0} one\u{2028}\u{3000}two \t\n";
        assert_eq!(trim(text), "one\u{2028}\u{3000}two");
        assert_eq!(collapse_space(text), "one two");
        // Not JavaScript whitespace.
        assert_eq!(trim("\u{85}x\u{200B}"), "\u{85}x\u{200B}");
    }

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
