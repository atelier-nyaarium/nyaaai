//! Provider gate cases.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};
use std::task::{Context, Poll, Waker};

use nyaaai_agentic::health::gate::{GateDeps, GateState, Phase, ProviderGate, WaitFn};
use nyaaai_contract::cancel::CancelToken;
use nyaaai_contract::future::BoxFuture;
use nyaaai_contract::id::CountedIds;
use nyaaai_contract::shapes::health::{Admission, GateWaiting, Ticket};
use nyaaai_contract::time::Clock;
use nyaaai_testkit::cases::{Outcome, block_on};
use nyaaai_testkit::scheduler::{FakeScheduler, TimerWheel};

use super::shapes::*;

fn locked<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// What the script knows of its requests: who waits in each model's queue, and who just stopped.
#[derive(Default)]
struct Mirror {
    tokens: HashMap<String, CancelToken>,
    /// Each model's waiting requests in the gate's queue order, models in the order first queued.
    queues: Vec<(String, Vec<String>)>,
    /// Requests whose abort just resolved them.
    stopped: Vec<String>,
}

fn abort_of(mirror: &Mutex<Mirror>, id: &str) -> Outcome<()> {
    let mut held = locked(mirror);
    let token = held
        .tokens
        .get(id)
        .cloned()
        .ok_or_else(|| format!("No request {id}."))?;
    for (_, queue) in &mut held.queues {
        queue.retain(|queued| queued != id);
    }
    held.stopped.push(id.to_owned());
    drop(held);
    token.cancel();
    Ok(())
}

fn phase_of(phase: Phase) -> GateRunResultStepsItemGatesItemGatePhase {
    match phase {
        Phase::Open => GateRunResultStepsItemGatesItemGatePhase::Open,
        Phase::Paused => GateRunResultStepsItemGatesItemGatePhase::Paused,
        Phase::Probing => GateRunResultStepsItemGatesItemGatePhase::Probing,
    }
}

fn gate_of(state: GateState) -> GateRunResultStepsItemGatesItemGate {
    GateRunResultStepsItemGatesItemGate {
        phase: phase_of(state.phase),
        generation: state.generation as i64,
        pauses: state.pauses as i64,
        resume_at: state.resume_at,
        probe: state.probe,
        queued: state.queued as i64,
    }
}

/// Takes the answer of a request the gate has settled; the request ends here.
fn resolve(
    pending: &mut HashMap<String, BoxFuture<'static, Admission>>,
    request: &str,
    resolved: &mut Vec<(String, Admission)>,
    tickets: &mut HashMap<String, Ticket>,
) -> Outcome<()> {
    let Some(mut admission) = pending.remove(request) else {
        return Ok(());
    };
    let Poll::Ready(admission) = admission
        .as_mut()
        .poll(&mut Context::from_waker(Waker::noop()))
    else {
        return Err(format!("The admission of {request} has not resolved."));
    };
    if let Admission::Go(go) = &admission {
        tickets.insert(request.to_owned(), go.ticket.clone());
    }
    resolved.push((request.to_owned(), admission));
    Ok(())
}

pub fn run(input: GateRunInput) -> Outcome<GateRunResult> {
    block_on(play(input))?
}

async fn play(input: GateRunInput) -> Outcome<GateRunResult> {
    let wheel = Arc::new(TimerWheel::new(input.start_ms));
    let scheduler = Arc::new(FakeScheduler::new(Arc::clone(&wheel)));
    let gate = ProviderGate::new(
        GateDeps {
            scheduler,
            pause_ms: input.pause_ms.map(|held| held as i64),
            max_pauses: input.max_pauses,
        },
        &CountedIds::new("gate-"),
    );
    let mut resolved: Vec<(String, Admission)> = Vec::new();
    let waits: Arc<Mutex<Vec<(String, GateWaiting)>>> = Arc::default();
    let told: Arc<Mutex<Vec<String>>> = Arc::default();
    let mut tickets: HashMap<String, Ticket> = HashMap::new();
    let mirror: Arc<Mutex<Mirror>> = Arc::default();
    // Each request's admission, until it resolves.
    let mut pending: HashMap<String, BoxFuture<'static, Admission>> = HashMap::new();
    let mut order: Vec<String> = Vec::new();
    {
        let told = Arc::clone(&told);
        drop(gate.on_change(Box::new(move |model| locked(&told).push(model.to_owned()))));
    }
    let mut models: Vec<String> = Vec::new();
    let mut steps = Vec::new();
    let mut disposed = false;
    let queued_of = |model: &str| gate.state_of(model).map_or(0, |held| held.queued);
    for step in &input.steps {
        let mut settled = None;
        let mut ticket = None;
        match step {
            GateRunInputStepsItem::Admit(held) => {
                if !models.contains(&held.model) {
                    models.push(held.model.clone());
                }
                let token = CancelToken::new();
                locked(&mirror)
                    .tokens
                    .insert(held.request.clone(), token.clone());
                if held.aborted_before == Some(true) {
                    token.cancel();
                } else if let Some(after) = held.abort_after_ms {
                    let (mirror, request) = (Arc::clone(&mirror), held.request.clone());
                    wheel.after_run(
                        after as i64,
                        &format!("abort-{}", held.request),
                        Box::new(move || {
                            let _ = abort_of(&mirror, &request);
                        }),
                    );
                }
                let before = queued_of(&held.model);
                let on_wait: WaitFn = {
                    let (waits, request) = (Arc::clone(&waits), held.request.clone());
                    Arc::new(move |waiting| locked(&waits).push((request.clone(), waiting)))
                };
                let again = held.again == Some(true);
                let admission = Box::pin(gate.admit(&held.model, &token, Some(on_wait), again));
                pending.insert(held.request.clone(), admission);
                order.push(held.request.clone());
                if queued_of(&held.model) > before {
                    let mut mirror = locked(&mirror);
                    let at = match mirror
                        .queues
                        .iter()
                        .position(|(model, _)| *model == held.model)
                    {
                        Some(at) => at,
                        None => {
                            mirror.queues.push((held.model.clone(), Vec::new()));
                            mirror.queues.len() - 1
                        }
                    };
                    let queue = &mut mirror.queues[at].1;
                    if again {
                        queue.insert(0, held.request.clone());
                    } else {
                        queue.push(held.request.clone());
                    }
                } else {
                    resolve(&mut pending, &held.request, &mut resolved, &mut tickets)?;
                }
            }
            GateRunInputStepsItem::TryAdmit(held) => {
                if !models.contains(&held.model) {
                    models.push(held.model.clone());
                }
                ticket = gate.try_admit(&held.model);
                if let Some(ticket) = &ticket {
                    tickets.insert(held.request.clone(), ticket.clone());
                }
            }
            GateRunInputStepsItem::Settle(held) => {
                let ticket = tickets
                    .get(&held.request)
                    .cloned()
                    .ok_or_else(|| format!("No ticket for {}.", held.request))?;
                if !models.contains(&ticket.model) {
                    models.push(ticket.model.clone());
                }
                settled = Some(gate.settle(
                    &ticket,
                    held.outcome,
                    held.retry_after_ms.map(|after| after as i64),
                ));
            }
            GateRunInputStepsItem::Cool(held) => {
                if !models.contains(&held.model) {
                    models.push(held.model.clone());
                }
                gate.cool(&held.model, held.retry_after_ms.map(|after| after as i64));
            }
            GateRunInputStepsItem::Clear(held) => {
                gate.clear(|model| model.starts_with(&held.prefix));
            }
            GateRunInputStepsItem::Advance(held) => {
                wheel.advance(held.ms as i64);
            }
            GateRunInputStepsItem::Abort(held) => abort_of(&mirror, &held.request)?,
            GateRunInputStepsItem::Dispose(_) => {
                gate.dispose();
                disposed = true;
            }
        }
        // Whatever the gate released left its queue from the front; its promises are resolved.
        let held: Vec<String> = {
            let names: Vec<String> = locked(&mirror)
                .queues
                .iter()
                .map(|(model, _)| model.clone())
                .collect();
            let counts: Vec<usize> = names.iter().map(|model| queued_of(model)).collect();
            let mut mirror = locked(&mirror);
            let mut released = Vec::new();
            for ((_, queue), queued) in mirror.queues.iter_mut().zip(counts) {
                let leaving = queue.len().saturating_sub(queued);
                released.extend(queue.drain(..leaving));
            }
            let mut held = std::mem::take(&mut mirror.stopped);
            held.extend(released);
            held
        };
        for request in held {
            resolve(&mut pending, &request, &mut resolved, &mut tickets)?;
        }
        steps.push(GateRunResultStepsItem {
            now_ms: wheel.now_ms(),
            settled,
            ticket,
            resolved: std::mem::take(&mut resolved)
                .into_iter()
                .map(|(request, admission)| GateRunResultStepsItemResolvedItem {
                    request,
                    admission,
                })
                .collect(),
            waits: std::mem::take(&mut *locked(&waits))
                .into_iter()
                .map(|(request, waiting)| GateRunResultStepsItemWaitsItem { request, waiting })
                .collect(),
            told: std::mem::take(&mut *locked(&told)),
            gates: models
                .iter()
                .map(|model| GateRunResultStepsItemGatesItem {
                    model: model.clone(),
                    gate: gate.state_of(model).map(gate_of),
                })
                .collect(),
            holding: gate.holding(),
        });
    }
    if !disposed {
        gate.dispose();
    }
    let rest: Vec<String> = pending.keys().cloned().collect();
    for request in rest {
        resolve(&mut pending, &request, &mut resolved, &mut tickets)?;
    }
    for request in &order {
        wheel.cancel(&format!("abort-{request}"));
    }
    Ok(GateRunResult {
        steps,
        waiting: wheel.waiting(),
    })
}
