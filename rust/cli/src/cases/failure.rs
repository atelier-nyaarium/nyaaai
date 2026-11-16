//! Failure decoding cases.

use std::error::Error;

use nyaaai_agentic::failure::{
    FailureExtra as Extra, ProviderError, failed, failure_of, message_of,
};
use nyaaai_contract::shapes::step::{FailureBilled, FailureStatus};
use nyaaai_testkit::cases::Outcome;

use super::shapes::*;

pub fn message(input: FailureMessageInput) -> Outcome<FailureMessageResult> {
    Ok(FailureMessageResult {
        messages: input
            .values
            .iter()
            .map(|value| message_of(value).map(str::to_owned))
            .collect(),
    })
}

fn extra_of(held: FailureExtra) -> Extra {
    Extra {
        status: held.status.map(|status| match status {
            FailureExtraStatus::Number(number) => FailureStatus::Number(number),
            FailureExtraStatus::Text(text) => FailureStatus::Text(text),
        }),
        retry_after_ms: held.retry_after_ms,
        resets_at: held.resets_at,
        reasons: held.reasons,
        billed: held.billed.map(|billed| FailureBilled {
            usage: billed.usage,
            host: billed.host,
            generation_id: billed.generation_id,
        }),
    }
}

pub fn of(input: FailureOfInput) -> Outcome<FailureOfResult> {
    let failures = input
        .thrown
        .into_iter()
        .map(|held| {
            let thrown: Box<dyn Error> = match held {
                FailureOfInputThrownItem::Failed(made) => Box::new(failed(
                    made.category,
                    &made.message,
                    made.extra.map(extra_of).unwrap_or_default(),
                )),
                FailureOfInputThrownItem::Provider(made) => {
                    Box::new(ProviderError::new(made.failure))
                }
                FailureOfInputThrownItem::Error(made) => Box::from(made.message),
            };
            failure_of(thrown.as_ref()).cloned()
        })
        .collect();
    Ok(FailureOfResult { failures })
}
