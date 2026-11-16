//! Decoded provider failures.

use std::error::Error;
use std::fmt;

use nyaaai_contract::json::{Object, Value};
use nyaaai_contract::shapes::step::{Failure, FailureBilled, FailureCategory, FailureStatus};
use nyaaai_contract::text::trim_text;

/// A number or a text.
#[derive(Debug, Clone, PartialEq)]
pub enum NumberOrText {
    Number(f64),
    Text(String),
}

/// Decoded API error.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct WireError {
    pub code: Option<NumberOrText>,
    pub type_: Option<String>,
    pub message: Option<String>,
    pub metadata: Option<Object>,
    /// Limit reset time.
    pub resets_at: Option<NumberOrText>,
}

/// What `failed` takes beside the category and message.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FailureExtra {
    pub status: Option<FailureStatus>,
    pub retry_after_ms: Option<i64>,
    pub resets_at: Option<i64>,
    pub reasons: Option<Vec<String>>,
    pub billed: Option<FailureBilled>,
}

/// A provider request's decoded failure.
#[derive(Debug, Clone, PartialEq)]
pub struct ProviderError {
    pub failure: Box<Failure>,
}

impl ProviderError {
    pub fn new(failure: Failure) -> Self {
        Self {
            failure: Box::new(failure),
        }
    }
}

impl fmt::Display for ProviderError {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        out.write_str(&self.failure.message)
    }
}

impl Error for ProviderError {}

/// Status is the category, for failures with no HTTP status.
pub fn failed(category: FailureCategory, message: &str, extra: FailureExtra) -> ProviderError {
    ProviderError::new(Failure {
        category,
        status: extra
            .status
            .unwrap_or_else(|| FailureStatus::Text(category.as_str().to_owned())),
        retry_after_ms: extra.retry_after_ms,
        resets_at: extra.resets_at,
        message: message.to_owned(),
        reasons: extra.reasons,
        billed: extra.billed,
    })
}

/// `None` for errors no codec decoded.
pub fn failure_of<'a>(error: &'a (dyn Error + 'static)) -> Option<&'a Failure> {
    error
        .downcast_ref::<ProviderError>()
        .map(|provider| &*provider.failure)
}

/// Blank counts as missing.
pub fn message_of(value: &Value) -> Option<&str> {
    value.as_str().filter(|text| !trim_text(text).is_empty())
}
