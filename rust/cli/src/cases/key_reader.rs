//! Key reader cases.

use std::sync::{Arc, Mutex, MutexGuard};

use nyaaai_agentic::health::key_reader::{Check, Failed, KeyReader, Shown};
use nyaaai_contract::future::BoxFuture;
use nyaaai_contract::shapes::provider::KeyState;
use nyaaai_contract::time::{Clock, Deadline, Ended};
use nyaaai_testkit::cases::{Outcome, block_on};
use nyaaai_testkit::scheduler::{FakeScheduler, TimerWheel};
use tokio::sync::oneshot;

use super::shapes::*;

fn locked<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

type Answer = Result<KeyState, ()>;

/// A check the script holds open: only a step, an abort or the deadline ends it.
struct Held {
    settle: Option<oneshot::Sender<Answer>>,
    closed: bool,
    chain: Option<BoxFuture<'static, ()>>,
    awaited: bool,
    limit: Arc<Deadline>,
}

/// Ends the checks whose signal ended, as the script sees it.
fn signals(checks: &Mutex<Vec<Held>>) -> Vec<KeyReaderRunResultStepsItemSignalledItem> {
    let mut told = Vec::new();
    for (at, held) in locked(checks).iter_mut().enumerate() {
        if held.closed {
            continue;
        }
        let Some(ended) = held.limit.ended() else {
            continue;
        };
        told.push(KeyReaderRunResultStepsItemSignalledItem {
            check: at as i64 + 1,
            by: match ended {
                Ended::Deadline => KeyReaderRunResultStepsItemSignalledItemBy::Deadline,
                Ended::Caller => KeyReaderRunResultStepsItemSignalledItemBy::Caller,
            },
        });
        held.closed = true;
        if let Some(settle) = held.settle.take() {
            let _ = settle.send(Err(()));
        }
    }
    told
}

fn settle(checks: &Mutex<Vec<Held>>, check: u64, answer: Answer) {
    let mut held = locked(checks);
    let Some(held) = check
        .checked_sub(1)
        .and_then(|at| held.get_mut(at as usize))
    else {
        return;
    };
    if held.closed {
        return;
    }
    held.closed = true;
    if let Some(settle) = held.settle.take() {
        let _ = settle.send(answer);
    }
}

pub fn run(input: KeyReaderRunInput) -> Outcome<KeyReaderRunResult> {
    block_on(async move {
        let wheel = Arc::new(TimerWheel::new(input.start_ms));
        let scheduler = Arc::new(FakeScheduler::new(Arc::clone(&wheel)));
        let checks: Arc<Mutex<Vec<Held>>> = Arc::new(Mutex::new(Vec::new()));
        let shown: Arc<Mutex<Vec<KeyState>>> = Arc::new(Mutex::new(Vec::new()));
        let check: Check = {
            let checks = Arc::clone(&checks);
            Arc::new(move |limit| {
                let (settle, wait) = oneshot::channel::<Answer>();
                locked(&checks).push(Held {
                    settle: Some(settle),
                    closed: false,
                    chain: None,
                    awaited: false,
                    limit,
                });
                Box::pin(async move {
                    match wait.await {
                        Ok(Ok(state)) => Ok(state),
                        _ => Err(Failed),
                    }
                })
            })
        };
        let seen: Shown = {
            let shown = Arc::clone(&shown);
            Arc::new(move |state| locked(&shown).push(state))
        };
        let reader = KeyReader::new(scheduler, check, seen);
        let mut steps = Vec::new();
        let mut signalled = Vec::new();
        for step in &input.steps {
            match step {
                KeyReaderRunInputStepsItem::Read(_) => {
                    let chain = reader.read();
                    if let Some(held) = locked(&checks).last_mut() {
                        held.chain = Some(chain);
                    }
                }
                KeyReaderRunInputStepsItem::Answer(held) => {
                    settle(&checks, held.check, Ok(held.state.clone()));
                }
                KeyReaderRunInputStepsItem::Fail(held) => settle(&checks, held.check, Err(())),
                KeyReaderRunInputStepsItem::Advance(held) => {
                    wheel.advance(held.ms as i64);
                }
                KeyReaderRunInputStepsItem::Dispose(_) => reader.dispose(),
            }
            signalled.extend(signals(&checks));
            // A closed check's chain has run once awaited.
            let chains: Vec<BoxFuture<'static, ()>> = locked(&checks)
                .iter_mut()
                .filter(|held| held.closed && !held.awaited && held.chain.is_some())
                .filter_map(|held| {
                    held.awaited = true;
                    held.chain.take()
                })
                .collect();
            for chain in chains {
                chain.await;
            }
            steps.push(KeyReaderRunResultStepsItem {
                now_ms: wheel.now_ms(),
                shown: std::mem::take(&mut *locked(&shown)),
                signalled: std::mem::take(&mut signalled),
                open: locked(&checks)
                    .iter()
                    .enumerate()
                    .filter(|(_, held)| !held.closed)
                    .map(|(at, _)| at as i64 + 1)
                    .collect(),
                busy: reader.busy(),
            });
        }
        reader.dispose();
        signals(&checks);
        reader.idle().await;
        KeyReaderRunResult {
            steps,
            waiting: wheel.waiting(),
        }
    })
}
