//! Test kit cases.

use nyaaai_contract::id::Ids;
use nyaaai_contract::json::{Record, Value};
use nyaaai_contract::order::compare_text;
use nyaaai_contract::random::{Random, SequenceRandom};
use nyaaai_contract::time::Clock;
use nyaaai_testkit::cases::{CaseContext, Outcome, failed};
use nyaaai_testkit::claude_session;
use nyaaai_testkit::scheduler::TimerWheel;
use nyaaai_testkit::trace::{ReplayTransport, read_trace};

use super::shapes::*;

pub fn wheel(input: TestkitWheelInput) -> Outcome<TestkitWheelResult> {
    let wheel = TimerWheel::new(input.start_ms);
    let steps = input
        .steps
        .iter()
        .map(|step| {
            let ms = step.ms.unwrap_or(0);
            let tag = step.tag.as_deref().unwrap_or("");
            let mut fired = Vec::new();
            let mut cancelled = false;
            match step.op {
                TestkitWheelInputStepsItemOp::Advance => fired = wheel.advance(ms),
                TestkitWheelInputStepsItemOp::Cancel => cancelled = wheel.cancel(tag),
                TestkitWheelInputStepsItemOp::After => wheel.after(ms, tag),
            }
            TestkitWheelResultStepsItem {
                now_ms: wheel.now_ms(),
                waiting: wheel.waiting(),
                fired: fired
                    .into_iter()
                    .map(|held| TestkitWheelResultStepsItemFiredItem {
                        tag: held.tag,
                        at_ms: held.at_ms,
                    })
                    .collect(),
                cancelled,
            }
        })
        .collect();
    Ok(TestkitWheelResult { steps })
}

pub fn random(input: TestkitRandomInput) -> Outcome<TestkitRandomResult> {
    let random = SequenceRandom::new(input.values).ok_or("a random sequence holds a value")?;
    let drawn = (0..input.draws).map(|_| random.next()).collect();
    Ok(TestkitRandomResult { drawn })
}

pub fn replay(input: TestkitReplayInput, context: &CaseContext) -> Outcome<TestkitReplayResult> {
    let trace = read_trace(&context.root().join(&input.file)).map_err(failed)?;
    let asked: Vec<Value> = trace
        .exchanges
        .iter()
        .map(|exchange| exchange.request.clone())
        .collect();
    let mut transport = ReplayTransport::new(trace);
    let mut exchanges = Vec::new();
    for request in asked {
        let replayed = transport.fetch(request).map_err(failed)?;
        let mut named = replayed.headers;
        named.sort_by(|(left, _), (right, _)| compare_text(left, right));
        let mut headers = Record::new();
        for (name, value) in named {
            headers.insert(name, value);
        }
        exchanges.push(TestkitReplayResultExchangesItem {
            status: i64::from(replayed.status),
            headers,
            chunks: replayed
                .chunks
                .iter()
                .map(|chunk| chunk.len() as i64)
                .collect(),
            body: String::from_utf8_lossy(&replayed.chunks.concat()).into_owned(),
        });
    }
    Ok(TestkitReplayResult {
        requests: transport.requests,
        exchanges,
    })
}

pub fn compacted_session(
    _input: TestkitCompactedSessionInput,
) -> Outcome<TestkitCompactedSessionResult> {
    Ok(TestkitCompactedSessionResult {
        lines: claude_session::compacted_session().session.lines,
    })
}

pub fn context(input: TestkitContextInput, context: &CaseContext) -> Outcome<TestkitContextResult> {
    let tmp = context.tmp()?;
    Ok(TestkitContextResult {
        ids: vec![context.ids.new_id(), context.ids.new_id()],
        times: vec![context.clock.now_ms(), context.clock.now_ms()],
        tmp: tmp.join(&input.folder).to_string_lossy().into_owned(),
        root: context
            .root()
            .join(&input.folder)
            .to_string_lossy()
            .into_owned(),
        planted: tmp.exists(),
    })
}
