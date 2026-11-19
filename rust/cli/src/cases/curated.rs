//! Curated list cases.

use nyaaai_agentic::models::curated::MODELS;
use nyaaai_testkit::cases::Outcome;

use super::shapes::*;

pub fn rows(_input: CuratedRowsInput) -> Outcome<CuratedRowsResult> {
    Ok(CuratedRowsResult {
        rows: MODELS.clone(),
    })
}
