//! Effort ladder cases.

use nyaaai_agentic::effort::{EFFORTS, resolve_effort};
use nyaaai_testkit::cases::Outcome;

use super::shapes::*;

pub fn ladder(_input: EffortLadderInput) -> Outcome<EffortLadderResult> {
    Ok(EffortLadderResult {
        efforts: EFFORTS.to_vec(),
    })
}

pub fn resolve(input: EffortResolveInput) -> Outcome<EffortResolveResult> {
    Ok(EffortResolveResult {
        values: input
            .wanted
            .iter()
            .map(|wanted| resolve_effort(&input.offered, wanted).cloned())
            .collect(),
    })
}
