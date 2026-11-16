//! Test kit cases.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use nyaaai_contract::cancel::CancelToken;
use nyaaai_contract::fetch::{Fetch, FetchRequest, Method};
use nyaaai_contract::id::Ids;
use nyaaai_contract::json::{Record, Value, parse_json, stringify_json};
use nyaaai_contract::random::{Random, SequenceRandom};
use nyaaai_contract::time::{Clock, Scheduled, Scheduler, Slept};
use nyaaai_testkit::cases::{CaseContext, Outcome, block_on, failed};
use nyaaai_testkit::claude_session;
use nyaaai_testkit::scheduler::{FakeScheduler, TimerWheel};
use nyaaai_testkit::trace::{read_trace, replay_fetch};

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

enum Stop {
    Sleep(CancelToken),
    Call(Scheduled),
}

fn drained<T>(held: &Mutex<Vec<T>>) -> Vec<T> {
    std::mem::take(&mut *held.lock().unwrap_or_else(|poisoned| poisoned.into_inner()))
}

pub fn scheduler(input: TestkitSchedulerInput) -> Outcome<TestkitSchedulerResult> {
    block_on(async move {
        let scheduler = FakeScheduler::new(Arc::new(TimerWheel::new(input.start_ms)));
        let ran = Arc::new(Mutex::new(Vec::<String>::new()));
        let ended = Arc::new(Mutex::new(Vec::<(String, &'static str)>::new()));
        let mut stops = HashMap::new();
        let mut steps = Vec::new();
        for step in &input.steps {
            let id = step.id.clone().unwrap_or_default();
            let ms = step.ms.unwrap_or(0);
            let tag = step.tag.clone().unwrap_or_default();
            let mut fired = Vec::new();
            match step.op {
                TestkitSchedulerInputStepsItemOp::Sleep => {
                    let token = match stops.get(&id) {
                        Some(Stop::Sleep(held)) => held.clone(),
                        _ => CancelToken::new(),
                    };
                    let slept = scheduler.sleep(ms, &token);
                    stops.insert(id.clone(), Stop::Sleep(token));
                    let ended = Arc::clone(&ended);
                    tokio::spawn(async move {
                        let outcome = match slept.await {
                            Slept::Done => "done",
                            Slept::Cancelled => "cancelled",
                        };
                        ended
                            .lock()
                            .unwrap_or_else(|poisoned| poisoned.into_inner())
                            .push((id, outcome));
                    });
                }
                TestkitSchedulerInputStepsItemOp::Schedule => {
                    let ran = Arc::clone(&ran);
                    let held = id.clone();
                    let call = scheduler.schedule(
                        ms,
                        Box::new(move || {
                            ran.lock()
                                .unwrap_or_else(|poisoned| poisoned.into_inner())
                                .push(held);
                        }),
                    );
                    stops.insert(id, Stop::Call(call));
                }
                TestkitSchedulerInputStepsItemOp::Cancel => match stops.get(&id) {
                    Some(Stop::Sleep(token)) => token.cancel(),
                    Some(Stop::Call(call)) => call.cancel(),
                    None => {}
                },
                TestkitSchedulerInputStepsItemOp::WheelCancel => {
                    scheduler.wheel().cancel(&tag);
                }
                TestkitSchedulerInputStepsItemOp::WheelAfter => scheduler.wheel().after(ms, &tag),
                TestkitSchedulerInputStepsItemOp::Settle => {
                    for _ in 0..4 {
                        tokio::task::yield_now().await;
                    }
                }
                TestkitSchedulerInputStepsItemOp::Advance => fired = scheduler.wheel().advance(ms),
            }
            steps.push(TestkitSchedulerResultStepsItem {
                now_ms: scheduler.now_ms(),
                waiting: scheduler.wheel().waiting(),
                fired: fired
                    .into_iter()
                    .map(|held| TestkitSchedulerResultStepsItemFiredItem {
                        tag: held.tag,
                        at_ms: held.at_ms,
                    })
                    .collect(),
                ran: drained(&ran),
                ended: drained(&ended)
                    .into_iter()
                    .map(|(id, outcome)| TestkitSchedulerResultStepsItemEndedItem {
                        id,
                        outcome: outcome.to_owned(),
                    })
                    .collect(),
            });
        }
        TestkitSchedulerResult { steps }
    })
}

pub fn random(input: TestkitRandomInput) -> Outcome<TestkitRandomResult> {
    let random = SequenceRandom::new(input.values).ok_or("a random sequence holds a value")?;
    let drawn = (0..input.draws).map(|_| random.next()).collect();
    Ok(TestkitRandomResult { drawn })
}

pub fn replay(input: TestkitReplayInput, context: &CaseContext) -> Outcome<TestkitReplayResult> {
    let trace = read_trace(&context.root().join(&input.file)).map_err(failed)?;
    let played = replay_fetch(&trace).map_err(failed)?;
    let cancel = CancelToken::new();
    let exchanges = block_on(async {
        let mut exchanges = Vec::new();
        for exchange in &trace.exchanges {
            let request = FetchRequest {
                method: Method::Post,
                url: "http://replay.invalid/".to_owned(),
                headers: Vec::new(),
                body: Some(stringify_json(&exchange.request)),
            };
            let mut response = played
                .fetch(request, cancel.clone())
                .await
                .map_err(|error| format!("{error:?}"))?;
            let mut chunks: Vec<Vec<u8>> = Vec::new();
            while let Some(chunk) = response
                .chunks
                .next()
                .await
                .map_err(|error| format!("{error:?}"))?
            {
                chunks.push(chunk);
            }
            let mut headers = Record::new();
            for (name, value) in response.headers {
                headers.insert(name, value);
            }
            exchanges.push(TestkitReplayResultExchangesItem {
                status: i64::from(response.status),
                headers,
                chunks: chunks.iter().map(|chunk| chunk.len() as i64).collect(),
                body: String::from_utf8_lossy(&chunks.concat()).into_owned(),
            });
        }
        Ok::<_, String>(exchanges)
    })??;
    let requests = played
        .requests()
        .iter()
        .map(|request| {
            request
                .body
                .as_deref()
                .and_then(parse_json)
                .unwrap_or(Value::Null)
        })
        .collect();
    Ok(TestkitReplayResult {
        requests,
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
