//! Step assembly cases.

use nyaaai_agentic::step::{StepAssembler, reply_calls, reply_text, reply_thoughts};
use nyaaai_contract::shape::Shape;
use nyaaai_contract::shapes::step::{ReplyPart, StepEvent};
use nyaaai_testkit::cases::Outcome;

use super::shapes::*;

pub fn assemble(input: StepAssembleInput) -> Outcome<StepAssembleResult> {
    let mut events = input.events;
    for text in input.more.iter().flatten() {
        events.push(StepEvent::decode_text(text).map_err(|_| "an event text is not an event")?);
    }
    let mut assembler = StepAssembler::new();
    let mut added = Vec::new();
    let mut failed_at = None;
    let mut failure = None;
    for (at, event) in events.iter().enumerate() {
        match assembler.add(event) {
            Ok(changed) => added.push(changed),
            Err(error) => {
                failure = Some(*error.failure);
                failed_at = Some(at as i64);
                break;
            }
        }
    }
    Ok(StepAssembleResult {
        added,
        failed_at,
        output_seen: assembler.output_seen(),
        billed: assembler.billed().map(|billed| StepBilled {
            usage: billed.usage,
            host: billed.host,
            generation_id: billed.generation_id,
        }),
        snapshot: assembler.snapshot(),
        result: failure.is_none().then(|| assembler.result()),
        failure,
    })
}

pub fn replies(input: StepRepliesInput) -> Outcome<StepRepliesResult> {
    Ok(StepRepliesResult {
        text: reply_text(&input.result),
        calls: reply_calls(&input.result)
            .into_iter()
            .map(ReplyPart::Call)
            .collect(),
        thoughts: reply_thoughts(&input.result),
    })
}
