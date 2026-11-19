//! Disposition cases.

use nyaaai_agentic::models::curated::MODELS;
use nyaaai_agentic::models::view::unknown_view;
use nyaaai_agentic::provider::disposition::{SKIP_REASONS, disposition_of, skip_reason_of};
use nyaaai_contract::shapes::model::ModelViewStatus;
use nyaaai_testkit::cases::Outcome;

use super::shapes::*;

pub fn of(input: DispositionOfInput) -> Outcome<DispositionOfResult> {
    let first = MODELS
        .first()
        .ok_or_else(|| "The curated list is empty.".to_owned())?;
    let dispositions = input
        .rows
        .iter()
        .map(|row| {
            let view = row.status.map(|status| {
                let mut view = unknown_view(first);
                view.status =
                    ModelViewStatus::parse(status.as_str()).unwrap_or(ModelViewStatus::Unknown);
                view
            });
            disposition_of(&row.failure, view.as_ref())
        })
        .collect();
    Ok(DispositionOfResult { dispositions })
}

pub fn skip_reason(input: DispositionSkipReasonInput) -> Outcome<DispositionSkipReasonResult> {
    Ok(DispositionSkipReasonResult {
        reasons: input.failures.iter().map(skip_reason_of).collect(),
        all: SKIP_REASONS.to_vec(),
    })
}
