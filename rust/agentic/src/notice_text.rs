//! Sentences for model notices.

/// `None`: the label is unknown.
pub fn overloaded_text(model: Option<&str>) -> String {
    format!(
        "{} is overloaded at its provider.",
        model.unwrap_or("The model")
    )
}

/// `None`: the label is unknown.
pub fn unavailable_text(model: Option<&str>, entry: Option<&str>) -> String {
    format!(
        "{} is unavailable on {}.",
        model.unwrap_or("The model"),
        entry.unwrap_or("its provider")
    )
}
