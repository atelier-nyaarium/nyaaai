//! Budget cases: the table, one budgeted exchange, one span's reads.

use std::sync::Arc;

use nyaaai_agentic::dialects::openrouter::OPENROUTER_CHAT;
use nyaaai_agentic::transport::{
    Auth, Forbidden, KeyAuth, KeySource, Transport, TransportDeps, TransportError,
};
use nyaaai_contract::cancel::CancelToken;
use nyaaai_contract::fetch::{
    BUDGET_CEILINGS, BUDGET_NAMES, BudgetBound, Fetch, FetchError, FetchRequest, Method, Span,
    budget_table,
};
use nyaaai_contract::future::BoxFuture;
use nyaaai_contract::json::Value;
use nyaaai_contract::time::{Clock, Ended, Scheduler};
use nyaaai_testkit::cases::{Outcome, block_on};
use nyaaai_testkit::scheduler::{FakeScheduler, TimerWheel};
use nyaaai_testkit::scripted_fetch::ScriptedFetch;

use super::shapes::*;
use super::transport::exchange_of;

const BASE_URL: &str = "https://lab.invalid/api";

/// A span of the ceilings on the caller's token, so a case measures the kind alone.
pub fn ceiling_span(scheduler: &Arc<FakeScheduler>, caller: &CancelToken) -> Span {
    let shared: Arc<dyn Scheduler> = scheduler.clone();
    nyaaai_contract::fetch::span(&shared, &BUDGET_CEILINGS, Some(caller))
}

/// A host's changes as the table function takes them.
pub fn changes_of(
    held: Option<&BudgetChanges>,
) -> Vec<(&str, nyaaai_contract::fetch::BudgetChange)> {
    held.map(|changes| {
        changes
            .iter()
            .map(|(name, change)| {
                (
                    name,
                    nyaaai_contract::fetch::BudgetChange {
                        total_ms: change.total_ms,
                        idle_ms: change.idle_ms,
                        max_bytes: change.max_bytes,
                    },
                )
            })
            .collect()
    })
    .unwrap_or_default()
}

pub fn table(input: BudgetTableInput) -> Outcome<BudgetTableResult> {
    Ok(match budget_table(&changes_of(Some(&input.changes))) {
        Ok(table) => BudgetTableResult {
            presets: Some(
                BUDGET_NAMES
                    .iter()
                    .map(|name| BudgetTableResultPresetsItem {
                        name: name.text().to_owned(),
                        budget: table.get(*name).clone(),
                    })
                    .collect(),
            ),
            refused: None,
        },
        Err(refused) => BudgetTableResult {
            presets: None,
            refused: Some(refused),
        },
    })
}

/// How an exchange ended, as the consumer saw it.
fn ended(at_ms: i64, error: FetchError) -> BudgetSeen {
    let end = match error {
        FetchError::Budget(exceeded) => BudgetEnd::Budget(BudgetEndBudget {
            bound: match exceeded.bound {
                BudgetBound::Total => BudgetEndBudgetBound::Total,
                BudgetBound::Idle => BudgetEndBudgetBound::Idle,
                BudgetBound::Bytes => BudgetEndBudgetBound::Bytes,
            },
            limit: exceeded.limit as i64,
        }),
        FetchError::Cancelled => BudgetEnd::Caller(BudgetEndCaller {}),
        FetchError::Failed(message) => BudgetEnd::Failed(BudgetEndFailed { message }),
    };
    BudgetSeen::Ended(BudgetSeenEnded { at_ms, end })
}

fn chunk_seen(at_ms: i64, step: Option<Vec<u8>>) -> BudgetSeen {
    match step {
        None => BudgetSeen::Done(BudgetSeenDone { at_ms }),
        Some(chunk) => BudgetSeen::Chunk(BudgetSeenChunk {
            at_ms,
            size: chunk.len() as i64,
        }),
    }
}

pub fn fetch(input: BudgetFetchInput) -> Outcome<BudgetFetchResult> {
    let wheel = Arc::new(TimerWheel::new(0));
    let scheduler = Arc::new(FakeScheduler::new(Arc::clone(&wheel)));
    let played = ScriptedFetch::new(vec![exchange_of(&input.exchange)?], scheduler);
    let caller = CancelToken::new();
    match (input.abort_after_ms, input.abort_armed_at_ms) {
        (Some(at), Some(armed)) => {
            // The abort is armed later, after the clocks.
            let (held, later) = (caller.clone(), Arc::clone(&wheel));
            let arm =
                move || later.after_run(at - armed, "caller", Box::new(move || held.cancel()));
            wheel.after_run(armed, "arm", Box::new(arm));
        }
        (Some(at), None) => {
            let held = caller.clone();
            wheel.after_run(at, "caller", Box::new(move || held.cancel()));
        }
        (None, _) => {}
    }
    let holds = input.holds.unwrap_or_default();
    let seen = block_on(async {
        let mut seen = Vec::new();
        let request = FetchRequest {
            method: Method::Get,
            url: format!("{BASE_URL}/x"),
            headers: Vec::new(),
            body: None,
            budget: input.budget.clone(),
        };
        let response = match played.fetch(request, caller.clone()).await {
            Ok(response) => response,
            Err(error) => {
                seen.push(ended(wheel.now_ms(), error));
                return seen;
            }
        };
        seen.push(BudgetSeen::Head(BudgetSeenHead {
            at_ms: wheel.now_ms(),
            status: i64::from(response.status),
        }));
        let mut chunks = response.chunks;
        let mut index = 0;
        let failure = loop {
            match chunks.next().await {
                Ok(None) => {
                    seen.push(chunk_seen(wheel.now_ms(), None));
                    break None;
                }
                Ok(Some(chunk)) => {
                    seen.push(chunk_seen(wheel.now_ms(), Some(chunk)));
                    if let Some(hold) = holds.get(index) {
                        wheel.advance(*hold as i64);
                    }
                    index += 1;
                }
                Err(error) => break Some(error),
            }
        };
        if let Some(error) = failure {
            seen.push(ended(wheel.now_ms(), error));
            if let Some(after) = input.after_ms {
                wheel.advance(after as i64);
                match chunks.next().await {
                    Ok(step) => seen.push(chunk_seen(wheel.now_ms(), step)),
                    Err(later) => seen.push(ended(wheel.now_ms(), later)),
                }
            }
        }
        // Letting go of the body is its return.
        drop(chunks);
        seen
    })?;
    wheel.cancel("arm");
    wheel.cancel("caller");
    Ok(BudgetFetchResult {
        seen,
        served: played.read().first().copied().unwrap_or(0) as i64,
        waiting: wheel.waiting(),
        now_ms: wheel.now_ms(),
    })
}

/// A key read that spends wheel time.
struct SlowKey {
    wheel: Arc<TimerWheel>,
    ms: i64,
}

impl KeySource for SlowKey {
    fn get(&self) -> BoxFuture<'_, Result<Option<String>, Forbidden>> {
        Box::pin(async move {
            self.wheel.advance(self.ms);
            Ok(Some("fake-key-one".to_owned()))
        })
    }
}

pub fn span(input: BudgetSpanInput) -> Outcome<BudgetSpanResult> {
    let wheel = Arc::new(TimerWheel::new(0));
    let scheduler = Arc::new(FakeScheduler::new(Arc::clone(&wheel)));
    let exchanges = input
        .exchanges
        .iter()
        .map(exchange_of)
        .collect::<Outcome<Vec<_>>>()?;
    let played = Arc::new(ScriptedFetch::new(exchanges, Arc::clone(&scheduler)));
    let shared: Arc<dyn Scheduler> = scheduler;
    let transport = Transport::new(TransportDeps {
        fetch: played.clone(),
        scheduler: Arc::clone(&shared),
        key: Arc::new(SlowKey {
            wheel: Arc::clone(&wheel),
            ms: input.key_ms.unwrap_or(0) as i64,
        }),
        log: Box::new(|_| {}),
        base_url: BASE_URL.to_owned(),
        label: "Lab".to_owned(),
        user_agent: "cases/1.0".to_owned(),
        errors: Arc::new(OPENROUTER_CHAT),
        auth: Auth::Bearer,
        budgets: None,
    });
    let clock = nyaaai_contract::fetch::span(&shared, &input.budget, None);
    let reads = block_on(async {
        let mut reads = Vec::new();
        for step in &input.steps {
            let path = match step {
                BudgetSpanStep::Advance(held) => {
                    wheel.advance(held.ms as i64);
                    continue;
                }
                BudgetSpanStep::Read(held) => &held.path,
            };
            if !path.starts_with('/') {
                return Err(format!("a path starts with a slash: {path}"));
            }
            let before = played.requests().len();
            let mut read = BudgetSpanRead {
                start_ms: wheel.now_ms(),
                end_ms: 0,
                sent: None,
                answer: Value::Null,
                failure: None,
                thrown: None,
            };
            match transport.get_json(path, &clock, &KeyAuth::Stored).await {
                Ok(answer) => read.answer = answer,
                Err(TransportError::Provider(error)) => read.failure = Some(*error.failure),
                Err(TransportError::Cancelled) => read.thrown = Some("cancelled".to_owned()),
            }
            read.end_ms = wheel.now_ms();
            read.sent = played
                .requests()
                .get(before)
                .map(|sent| sent.budget.clone());
            reads.push(read);
        }
        Ok(reads)
    })??;
    let ended = clock.ended().map(|by| match by {
        Ended::Deadline => BudgetSpanResultEnded::Deadline,
        Ended::Caller => BudgetSpanResultEnded::Caller,
    });
    clock.clear();
    Ok(BudgetSpanResult {
        reads,
        ended,
        requests: played.requests().len() as i64,
        waiting: wheel.waiting(),
    })
}
