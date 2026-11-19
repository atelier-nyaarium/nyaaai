//! Provider gate cases.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};
use std::task::{Context, Poll, Waker};

use nyaaai_agentic::health::gate::{GateBook, GateDeps, GateState, Phase, ProviderGate, WaitFn};
use nyaaai_contract::cancel::CancelToken;
use nyaaai_contract::future::BoxFuture;
use nyaaai_contract::id::CountedIds;
use nyaaai_contract::shapes::health::{Admission, GateOutcome, GateWaiting, Ticket};
use nyaaai_contract::time::Clock;
use nyaaai_testkit::cases::{Outcome, block_on};
use nyaaai_testkit::scheduler::{FakeScheduler, TimerWheel};

use super::shapes::*;

fn locked<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// The books a script names; a book's era is counted from 1 as books are first used.
#[derive(Default)]
struct Books {
    named: HashMap<String, Arc<GateBook>>,
    by_era: HashMap<u64, Arc<GateBook>>,
    eras: u64,
    /// The book each model was first used in.
    homes: HashMap<String, String>,
}

const MAIN: &str = "main";

impl Books {
    fn book_of(&mut self, name: Option<&str>) -> Arc<GateBook> {
        let name = name.unwrap_or(MAIN);
        if let Some(book) = self.named.get(name) {
            return Arc::clone(book);
        }
        self.eras += 1;
        let book = Arc::new(GateBook::new(self.eras, name.to_owned()));
        self.named.insert(name.to_owned(), Arc::clone(&book));
        self.by_era.insert(self.eras, Arc::clone(&book));
        book
    }

    fn use_in(&mut self, model: &str, name: Option<&str>) {
        self.homes
            .insert(model.to_owned(), name.unwrap_or(MAIN).to_owned());
    }

    /// The book a ticket's era names, else the main one.
    fn book_for(&mut self, ticket: &Ticket) -> Arc<GateBook> {
        match self.by_era.get(&ticket.era) {
            Some(book) => Arc::clone(book),
            None => self.book_of(None),
        }
    }

    fn live(&self, name: Option<&str>) -> Option<Arc<GateBook>> {
        self.named.get(name.unwrap_or(MAIN)).cloned()
    }

    fn home(&self, model: &str) -> Option<String> {
        self.homes.get(model).cloned()
    }
}

fn state_in(
    gate: &ProviderGate,
    books: &Mutex<Books>,
    model: &str,
    name: Option<&str>,
) -> Option<GateState> {
    let book = locked(books).live(name)?;
    gate.state_of(&book, model)
}

fn state_of_model(gate: &ProviderGate, books: &Mutex<Books>, model: &str) -> Option<GateState> {
    let home = locked(books).home(model);
    state_in(gate, books, model, home.as_deref())
}

/// What the script knows of its requests: queues and who just stopped.
#[derive(Default)]
struct Mirror {
    tokens: HashMap<String, CancelToken>,
    /// Each model's waiting requests in queue order, models in first-queued order.
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

/// What a waiter's callback saw of its gate, by request.
type Inside = Arc<Mutex<Vec<(String, Option<GateState>)>>>;

fn inside_of(state: GateState) -> GateRunResultStepsItemInsideItemGate {
    GateRunResultStepsItemInsideItemGate {
        phase: match state.phase {
            Phase::Open => GateRunResultStepsItemInsideItemGatePhase::Open,
            Phase::Paused => GateRunResultStepsItemInsideItemGatePhase::Paused,
            Phase::Probing => GateRunResultStepsItemInsideItemGatePhase::Probing,
        },
        generation: state.generation as i64,
        pauses: state.pauses as i64,
        resume_at: state.resume_at,
        probe: state.probe,
        queued: state.queued as i64,
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

/// Takes a settled request's answer; the request ends here.
fn resolve(
    pending: &mut HashMap<String, BoxFuture<'static, Admission>>,
    request: &str,
    resolved: &mut Vec<(String, Admission)>,
    tickets: &Mutex<HashMap<String, Ticket>>,
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
        locked(tickets).insert(request.to_owned(), go.ticket.clone());
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
    let gate = Arc::new(ProviderGate::new(
        GateDeps {
            runtime: nyaaai_testkit::cases::runtime(),
            scheduler,
            pause_ms: input.pause_ms.map(|held| held as i64),
            max_pauses: input.max_pauses,
            on_ticket: None,
            on_withdrawn: None,
        },
        &CountedIds::new("gate-"),
    ));
    let mut resolved: Vec<(String, Admission)> = Vec::new();
    let waits: Arc<Mutex<Vec<(String, GateWaiting)>>> = Arc::default();
    let told: Arc<Mutex<Vec<String>>> = Arc::default();
    let tickets: Arc<Mutex<HashMap<String, Ticket>>> = Arc::default();
    let inside: Inside = Arc::default();
    let mirror: Arc<Mutex<Mirror>> = Arc::default();
    // Each request's admission, until it resolves.
    let mut pending: HashMap<String, BoxFuture<'static, Admission>> = HashMap::new();
    let mut order: Vec<String> = Vec::new();
    {
        let told = Arc::clone(&told);
        drop(gate.on_change(Box::new(move |_owner, model| {
            locked(&told).push(model.to_owned())
        })));
    }
    let mut models: Vec<String> = Vec::new();
    let mut steps = Vec::new();
    let mut disposed = false;
    let books: Arc<Mutex<Books>> = Arc::default();
    let queued_of = |model: &str, name: Option<&str>| {
        state_in(&gate, &books, model, name).map_or(0, |held| held.queued)
    };
    let queued_home =
        |model: &str| state_of_model(&gate, &books, model).map_or(0, |held| held.queued);
    for step in &input.steps {
        let mut settled = None;
        let mut ticket = None;
        match step {
            GateRunInputStepsItem::Admit(held) => {
                if !models.contains(&held.model) {
                    models.push(held.model.clone());
                }
                locked(&books).use_in(&held.model, held.book.as_deref());
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
                let before = queued_of(&held.model, held.book.as_deref());
                let on_wait: WaitFn = {
                    let (waits, request) = (Arc::clone(&waits), held.request.clone());
                    let (gate, inside, tickets, books) = (
                        Arc::clone(&gate),
                        Arc::clone(&inside),
                        Arc::clone(&tickets),
                        Arc::clone(&books),
                    );
                    let (model, action, name) =
                        (held.model.clone(), held.inside, held.book.clone());
                    Arc::new(move |waiting| {
                        locked(&waits).push((request.clone(), waiting));
                        locked(&inside)
                            .push((request.clone(), state_of_model(&gate, &books, &model)));
                        let book = locked(&books).book_of(name.as_deref());
                        match action {
                            Some(GateRunInputStepsItemAdmitInside::Cool) => {
                                gate.cool(&book, &model, None);
                            }
                            Some(GateRunInputStepsItemAdmitInside::TryAdmit) => {
                                let _ = gate.try_admit(&book, &model);
                            }
                            Some(GateRunInputStepsItemAdmitInside::SettleTrigger) => {
                                let held = locked(&tickets).get("trigger").cloned();
                                if let Some(held) = held {
                                    let book = locked(&books).book_for(&held);
                                    gate.settle(&book, &held, GateOutcome::Overloaded, None);
                                }
                            }
                            None => {}
                        }
                    })
                };
                let again = held.again == Some(true);
                let book = locked(&books).book_of(held.book.as_deref());
                let admission =
                    Box::pin(gate.admit(&book, &held.model, &token, Some(on_wait), again));
                pending.insert(held.request.clone(), admission);
                order.push(held.request.clone());
                if queued_of(&held.model, held.book.as_deref()) > before {
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
                    resolve(&mut pending, &held.request, &mut resolved, &tickets)?;
                }
            }
            GateRunInputStepsItem::TryAdmit(held) => {
                if !models.contains(&held.model) {
                    models.push(held.model.clone());
                }
                locked(&books).use_in(&held.model, held.book.as_deref());
                let book = locked(&books).book_of(held.book.as_deref());
                ticket = gate.try_admit(&book, &held.model);
                if let Some(ticket) = &ticket {
                    locked(&tickets).insert(held.request.clone(), ticket.clone());
                }
            }
            GateRunInputStepsItem::Settle(held) => {
                let ticket = locked(&tickets)
                    .get(&held.request)
                    .cloned()
                    .ok_or_else(|| format!("No ticket for {}.", held.request))?;
                if !models.contains(&ticket.model) {
                    models.push(ticket.model.clone());
                }
                {
                    let mut books = locked(&books);
                    let home = books.home(&ticket.model);
                    books.use_in(&ticket.model, home.as_deref());
                }
                let book = locked(&books).book_for(&ticket);
                settled = Some(gate.settle(
                    &book,
                    &ticket,
                    held.outcome,
                    held.retry_after_ms.map(|after| after as i64),
                ));
            }
            GateRunInputStepsItem::SettleTicket(held) => {
                if !models.contains(&held.ticket.model) {
                    models.push(held.ticket.model.clone());
                }
                {
                    let mut books = locked(&books);
                    let home = books.home(&held.ticket.model);
                    books.use_in(&held.ticket.model, home.as_deref());
                }
                let book = locked(&books).book_for(&held.ticket);
                settled = Some(gate.settle(&book, &held.ticket, held.outcome, None));
            }
            GateRunInputStepsItem::Cool(held) => {
                if !models.contains(&held.model) {
                    models.push(held.model.clone());
                }
                locked(&books).use_in(&held.model, held.book.as_deref());
                let book = locked(&books).book_of(held.book.as_deref());
                gate.cool(
                    &book,
                    &held.model,
                    held.retry_after_ms.map(|after| after as i64),
                );
            }
            GateRunInputStepsItem::Retire(held) => {
                let book = locked(&books).book_of(held.book.as_deref());
                gate.retire(&book);
                locked(&books)
                    .named
                    .remove(held.book.as_deref().unwrap_or(MAIN));
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
        // Released requests left their queue front; their promises are resolved.
        let held: Vec<String> = {
            let names: Vec<String> = locked(&mirror)
                .queues
                .iter()
                .map(|(model, _)| model.clone())
                .collect();
            let counts: Vec<usize> = names.iter().map(|model| queued_home(model)).collect();
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
            resolve(&mut pending, &request, &mut resolved, &tickets)?;
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
            inside: std::mem::take(&mut *locked(&inside))
                .into_iter()
                .map(|(request, state)| GateRunResultStepsItemInsideItem {
                    request,
                    gate: state.map(inside_of),
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
                    gate: state_of_model(&gate, &books, model).map(gate_of),
                })
                .collect(),
            holding: {
                let live: Vec<Arc<GateBook>> = locked(&books).named.values().cloned().collect();
                let held: Vec<String> = live.iter().flat_map(|book| gate.holding(book)).collect();
                nyaaai_contract::order::sorted_texts(&held)
            },
        });
    }
    if !disposed {
        gate.dispose();
    }
    let rest: Vec<String> = pending.keys().cloned().collect();
    for request in rest {
        resolve(&mut pending, &request, &mut resolved, &tickets)?;
    }
    for request in &order {
        wheel.cancel(&format!("abort-{request}"));
    }
    Ok(GateRunResult {
        steps,
        waiting: wheel.waiting(),
    })
}
