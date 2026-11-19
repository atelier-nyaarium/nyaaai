//! OpenRouter key check.

use nyaaai_contract::cancel::CancelToken;
use nyaaai_contract::json::Object;
use nyaaai_contract::shapes::step::FailureCategory;

use crate::kinds::readers::{field_of, fields_of, number_field, object_field, text_field};
use crate::transport::{KeyAuth, Transport, TransportError};

/// Why a key did not pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyFailure {
    Rejected,
    Unreachable,
}

/// USD; absent limit means unlimited.
#[derive(Debug, Clone, PartialEq)]
pub enum KeyCheck {
    Ok {
        usage: Option<f64>,
        limit: Option<f64>,
        reset: Option<String>,
    },
    Failed(KeyFailure),
}

/// `GET /key` validates keys and reports limits; a cancel of the caller is the error.
pub async fn check_key(
    transport: &Transport,
    auth: &KeyAuth,
    cancel: &CancelToken,
) -> Result<KeyCheck, TransportError> {
    let body = match transport.get_json("/key", cancel, auth).await {
        Ok(body) => body,
        Err(error) => {
            if cancel.is_cancelled() {
                return Err(error);
            }
            let rejected = matches!(
                &error,
                TransportError::Provider(held) if matches!(
                    held.failure.category,
                    FailureCategory::KeyRejected | FailureCategory::Refused | FailureCategory::KeyMissing
                )
            );
            return Ok(KeyCheck::Failed(if rejected {
                KeyFailure::Rejected
            } else {
                KeyFailure::Unreachable
            }));
        }
    };
    let empty = Object::new();
    let data = fields_of(&body)
        .and_then(|fields| object_field(fields, "data"))
        .unwrap_or(&empty);
    Ok(KeyCheck::Ok {
        usage: number_field(data, "usage"),
        limit: number_field(data, "limit"),
        reset: field_of(data, "limit_reset")
            .and_then(|_| text_field(data, "limit_reset"))
            .map(str::to_owned),
    })
}
