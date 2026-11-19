//! One request over its candidate routes: admission, retries, fall-through, metering and a trace.

use std::collections::HashSet;
use std::error::Error;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use nyaaai_contract::cancel::CancelToken;
use nyaaai_contract::future::BoxFuture;
use nyaaai_contract::json::number_to_string;
use nyaaai_contract::random::SystemRandom;
use nyaaai_contract::shapes::health::{
    Admission, Blocked, ErrorStatus, GateOutcome, GateSettled, GateWaiting, Notice, RetryOptions,
    Ticket,
};
use nyaaai_contract::shapes::lifecycle::{
    TraceEntry, TraceEntryAdmitted, TraceEntryAttempt, TraceEntryBilling, TraceEntryBillingReply,
    TraceEntryFailure, TraceEntryGateFailed, TraceEntryHeld, TraceEntryKept, TraceEntryLimits,
    TraceEntryMetered, TraceEntryNotice, TraceEntryReleased, TraceEntryRetry, TraceEntryRetryWhy,
    TraceEntryServed, TraceEntrySettled, TraceEntrySkip, TraceEntrySkipCause, TraceEntryStopped,
    TraceEntryThrew,
};
use nyaaai_contract::shapes::provider::{Disposition, NoticeKind, Passed, Skip, SkipReason};
use nyaaai_contract::shapes::step::{
    Failure, FailureBilled, FailureCategory, FailureStatus, NormalizedUsage, StepResult,
};
use nyaaai_contract::shapes::usage::{CallTag, MeteredCall};
use nyaaai_contract::time::Scheduler;

use super::retry::RetryPolicy;
use crate::failure::failure_of;
use crate::health::health::{Gate, RouteRef};
use crate::notice_text::{overloaded_text, unavailable_text};
use crate::provider::disposition::skip_reason_of;
use crate::provider::resolver::{Resolved, Routes};
use crate::usage::cost::{step_cost, wants_long_cache};
use crate::usage::meter::Meter;

/// What a send throws: a provider's failure, or any other error.
pub type SendError = Box<dyn Error + Send + Sync>;

/// How a request ended, short of a reply.
#[derive(Debug, Clone, PartialEq)]
pub enum Ended {
    Stopped,
    Notice(Notice),
}

/// A request's end.
#[allow(clippy::large_enum_variant)]
pub enum Attempt<'a, T> {
    Served {
        value: T,
        route: &'a Resolved,
    },
    Failed {
        outcome: Ended,
        failure: Option<Failure>,
    },
}

/// Which ended a call's token.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallEnd {
    Deadline,
    Caller,
}

/// Told what a request does. With no trace wanted no entry is built.
pub trait Observer: Send + Sync {
    /// False after streamed output.
    fn fresh(&self) -> bool {
        true
    }

    fn on_retry(&self, _attempt: i64, _wait_ms: f64, _status: &ErrorStatus) {}

    fn on_wait(&self, _waiting: Option<&GateWaiting>) {}

    fn on_skip(&self, _skip: &Skip) {}

    /// Whether `on_trace` is given entries.
    fn traces(&self) -> bool {
        false
    }

    /// Told one entry per effect, in order.
    fn on_trace(&self, _entry: TraceEntry) {}
}

/// What a reply adds to the meter's call.
#[derive(Debug, Clone, Default)]
pub struct Metered {
    pub usage: Option<NormalizedUsage>,
    pub provider: Option<String>,
    pub generation_id: Option<String>,
    pub finish_reason: Option<String>,
    pub failure: Option<String>,
    pub cancelled: Option<bool>,
}

/// One call to send over the routes.
pub trait Call: Send + Sync {
    type Value: Send;

    fn tag(&self) -> &CallTag;

    /// Ends with the caller's cancel or its deadline.
    fn cancel(&self) -> &CancelToken;

    /// Which ended the token, when it ended: the caller's deadline or a cancel.
    fn ended(&self) -> Option<CallEnd> {
        None
    }

    /// Two nonblocking tries.
    fn live(&self) -> bool {
        false
    }

    fn send<'a>(
        &'a self,
        route: &'a Resolved,
        first_token: &'a (dyn Fn() + Send + Sync),
    ) -> BoxFuture<'a, Result<Self::Value, SendError>>;

    fn read(&self, value: &Self::Value) -> Metered;

    /// The usage a reply carries.
    fn usage(value: &Self::Value) -> Option<&NormalizedUsage>;

    /// Usage billed before failure.
    fn billed(&self) -> Option<FailureBilled> {
        None
    }
}

pub struct LifecycleDeps {
    pub gate: Option<Arc<dyn Gate>>,
    pub meter: Option<Arc<dyn Meter>>,
    pub retry: Option<RetryPolicy>,
    /// The time and the waits of the retries.
    pub scheduler: Arc<dyn Scheduler>,
}

/// A refusal before a route is tried.
enum Refusal {
    Skip(SkipReason),
    Blocked(Blocked),
}

/// A route's tries, short of an answer.
enum Tried<'a, T> {
    Done(Attempt<'a, T>),
    /// The failure to fall through on.
    Onward(Failure),
    /// Deferred by ZDR.
    Kept,
}

enum Gated {
    Go(Option<Ticket>),
    Stopped,
    GaveUp,
}

const LIVE_TRIES: i64 = 2;

fn locked<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn status_of(status: &FailureStatus) -> ErrorStatus {
    match status {
        FailureStatus::Number(held) => ErrorStatus::Number(*held),
        FailureStatus::Text(held) => ErrorStatus::Text(held.clone()),
    }
}

fn status_text(status: &FailureStatus) -> String {
    match status {
        FailureStatus::Number(held) => number_to_string(*held),
        FailureStatus::Text(held) => held.clone(),
    }
}

/// Adds route context.
pub fn notice(kind: NoticeKind, failure: Option<&Failure>, route: Option<&Resolved>) -> Ended {
    Ended::Notice(Notice {
        kind,
        status: failure.map(|held| status_of(&held.status)),
        message: failure.map(|held| held.message.clone()),
        reasons: None,
        limit: None,
        reset: None,
        resets_at: None,
        next_try_at: None,
        model: None,
        entry: route.map(|held| held.entry.label.clone()),
    })
}

fn overloaded(route: &Resolved) -> Ended {
    let model = route.view.row.label.clone();
    Ended::Notice(Notice {
        kind: NoticeKind::ProviderOverloaded,
        message: Some(overloaded_text(Some(&model))),
        model: Some(model),
        ..empty_notice(NoticeKind::ProviderOverloaded)
    })
}

fn empty_notice(kind: NoticeKind) -> Notice {
    Notice {
        kind,
        status: None,
        message: None,
        reasons: None,
        limit: None,
        reset: None,
        resets_at: None,
        next_try_at: None,
        model: None,
        entry: None,
    }
}

/// Tells the sink one entry; nothing is built without a sink, and a throwing sink changes nothing.
fn trace(o: &Arc<dyn Observer>, entry: impl FnOnce() -> TraceEntry) {
    if o.traces() {
        let _ = catch_unwind(AssertUnwindSafe(|| o.on_trace(entry())));
    }
}

/// Distinguishes a deadline from a cancel.
fn abandoned(ended: Option<CallEnd>) -> (Option<String>, Option<bool>) {
    if ended == Some(CallEnd::Deadline) {
        (Some("timedOut".to_owned()), None)
    } else {
        (None, Some(true))
    }
}

fn reason_of(error: &SendError, route: &Resolved) -> String {
    let Some(failure) = failure_of(&**error) else {
        return "error".to_owned();
    };
    if failure.category == FailureCategory::Overloaded {
        return "overloaded".to_owned();
    }
    match route.kind.failure(failure, Some(&route.view)) {
        Disposition::Retry(_) => status_text(&failure.status),
        Disposition::FallThrough(held) => held.notice.as_str().to_owned(),
        Disposition::Stop(held) => held.notice.as_str().to_owned(),
    }
}

/// What a step's result adds to the meter's call.
pub fn metered_of(result: &StepResult) -> Metered {
    Metered {
        usage: result.usage.clone(),
        provider: result.serving.host.clone(),
        generation_id: result.serving.generation_id.clone(),
        finish_reason: Some(result.finish.as_str().to_owned()),
        failure: None,
        cancelled: None,
    }
}

struct Inner {
    gate: Option<Arc<dyn Gate>>,
    meter: Option<Arc<dyn Meter>>,
    retry: RetryPolicy,
    scheduler: Arc<dyn Scheduler>,
}

/// Runs candidates in order.
#[derive(Clone)]
pub struct RequestLifecycle {
    d: Arc<Inner>,
}

impl RequestLifecycle {
    pub fn new(d: LifecycleDeps) -> Self {
        let retry = d.retry.unwrap_or_else(|| {
            RetryPolicy::new(
                &RetryOptions {
                    tries: None,
                    budget_ms: None,
                    cap_ms: None,
                    base_ms: None,
                },
                Arc::new(SystemRandom),
            )
        });
        Self {
            d: Arc::new(Inner {
                gate: d.gate,
                meter: d.meter,
                retry,
                scheduler: d.scheduler,
            }),
        }
    }

    pub async fn run<'a, C: Call>(
        &self,
        candidates: &'a [Resolved],
        call: &C,
        o: &Arc<dyn Observer>,
    ) -> Result<Attempt<'a, C::Value>, SendError> {
        self.d.attempt(candidates, &[], None, call, o).await
    }

    /// Creates a stateful route.
    pub fn begin(&self) -> RunRoute {
        RunRoute {
            d: Arc::clone(&self.d),
            keep: Mutex::new(HashSet::new()),
            skipped: Arc::new(Mutex::new(HashSet::new())),
        }
    }
}

impl Inner {
    /// Balance replies extend `keep`; the run's end is the last entry told.
    async fn attempt<'a, C: Call>(
        &self,
        candidates: &'a [Resolved],
        passed: &[Passed],
        keep: Option<&Mutex<HashSet<String>>>,
        call: &C,
        o: &Arc<dyn Observer>,
    ) -> Result<Attempt<'a, C::Value>, SendError> {
        let result = self.walk(candidates, passed, keep, call, o).await?;
        trace(o, || match &result {
            Attempt::Served { route, .. } => TraceEntry::Served(TraceEntryServed {
                entry: route.entry.id.clone(),
                label: route.entry.label.clone(),
                model: route.key.clone(),
            }),
            Attempt::Failed {
                outcome: Ended::Stopped,
                ..
            } => TraceEntry::Stopped(TraceEntryStopped {}),
            Attempt::Failed {
                outcome: Ended::Notice(held),
                ..
            } => TraceEntry::Notice(TraceEntryNotice {
                notice: held.clone(),
            }),
        });
        Ok(result)
    }

    fn skip(
        &self,
        o: &Arc<dyn Observer>,
        route: &Resolved,
        reason: SkipReason,
        cause: TraceEntrySkipCause,
    ) {
        trace(o, || {
            TraceEntry::Skip(TraceEntrySkip {
                cause,
                entry: route.entry.id.clone(),
                label: route.entry.label.clone(),
                reason,
            })
        });
        o.on_skip(&Skip {
            entry: route.entry.id.clone(),
            label: route.entry.label.clone(),
            reason,
        });
    }

    fn ahead(&self, o: &Arc<dyn Observer>, passed: &[Passed], at: usize) {
        for each in passed {
            if each.before != at as f64 {
                continue;
            }
            trace(o, || {
                TraceEntry::Skip(TraceEntrySkip {
                    cause: TraceEntrySkipCause::Passed,
                    entry: each.entry.clone(),
                    label: each.label.clone(),
                    reason: each.reason,
                })
            });
            o.on_skip(&Skip {
                entry: each.entry.clone(),
                label: each.label.clone(),
                reason: each.reason,
            });
        }
    }

    async fn walk<'a, C: Call>(
        &self,
        candidates: &'a [Resolved],
        passed: &[Passed],
        keep: Option<&Mutex<HashSet<String>>>,
        call: &C,
        o: &Arc<dyn Observer>,
    ) -> Result<Attempt<'a, C::Value>, SendError> {
        let count = candidates.len();
        for (at, route) in candidates[..count - 1].iter().enumerate() {
            self.ahead(o, passed, at);
            if let Some(Refusal::Skip(reason)) = self.refusal(route, false, keep) {
                self.skip(o, route, reason, TraceEntrySkipCause::Deferred);
                continue;
            }
            match self.tries(route, call, o, keep, true).await? {
                Tried::Kept => continue,
                Tried::Done(done) => return Ok(done),
                Tried::Onward(failure) => {
                    self.failed(route, &failure, true, o);
                    self.skip(
                        o,
                        route,
                        skip_reason_of(&failure),
                        TraceEntrySkipCause::FellThrough,
                    );
                }
            }
        }
        self.ahead(o, passed, count - 1);
        let last = &candidates[count - 1];
        if let Some(Refusal::Blocked(blocked)) = self.refusal(last, true, keep) {
            let mut held = empty_notice(NoticeKind::KeyRejected);
            match blocked {
                Blocked::KeyRejected(_) => {}
                Blocked::SpendLimit(spend) => {
                    held.kind = NoticeKind::SpendLimit;
                    held.resets_at = spend.resets_at;
                    held.next_try_at = spend.next_try_at;
                }
            }
            held.entry = Some(last.entry.label.clone());
            return Ok(Attempt::Failed {
                outcome: Ended::Notice(held),
                failure: None,
            });
        }
        Ok(match self.tries(last, call, o, keep, false).await? {
            Tried::Kept => Attempt::Failed {
                outcome: notice(NoticeKind::NoZdrHost, None, None),
                failure: None,
            },
            Tried::Done(done) => done,
            Tried::Onward(_) => {
                return Err("The last route never falls through.".into());
            }
        })
    }

    /// Earlier defer; last blocks.
    fn refusal(
        &self,
        route: &Resolved,
        last: bool,
        keep: Option<&Mutex<HashSet<String>>>,
    ) -> Option<Refusal> {
        let gate = self.gate.as_ref()?;
        let held = keep.map(locked);
        let kept = held.as_deref();
        if last {
            gate.blocked(RouteRef::from(route), kept)
                .map(Refusal::Blocked)
        } else {
            gate.deferred(RouteRef::from(route), kept)
                .map(Refusal::Skip)
        }
    }

    async fn tries<'a, C: Call>(
        &self,
        route: &'a Resolved,
        call: &C,
        o: &Arc<dyn Observer>,
        run: Option<&Mutex<HashSet<String>>>,
        onward: bool,
    ) -> Result<Tried<'a, C::Value>, SendError> {
        // Backoff consumes budget.
        let mut waited = 0.0_f64;
        let mut again = false;
        let mut attempt: i64 = 1;
        loop {
            let readmitted = again;
            let admission = if call.live() {
                self.admit_live(route, call, o)
            } else {
                self.admit(route, call, o, again).await
            };
            again = false;
            let ticket = match admission {
                Gated::Stopped => {
                    return Ok(Tried::Done(Attempt::Failed {
                        outcome: Ended::Stopped,
                        failure: None,
                    }));
                }
                Gated::GaveUp => {
                    return Ok(Tried::Done(Attempt::Failed {
                        outcome: overloaded(route),
                        failure: None,
                    }));
                }
                Gated::Go(ticket) => ticket,
            };
            // Recheck ZDR on admission.
            if route.zdr_only() && route.view.data_kept.is_some() {
                trace(o, || {
                    TraceEntry::Kept(TraceEntryKept {
                        entry: route.entry.id.clone(),
                        model: route.key.clone(),
                    })
                });
                self.settle(route, ticket.as_ref(), GateOutcome::Cancelled, None, o);
                return Ok(Tried::Kept);
            }
            trace(o, || {
                TraceEntry::Attempt(TraceEntryAttempt {
                    entry: route.entry.id.clone(),
                    model: route.key.clone(),
                    attempt: attempt as u64,
                    live: call.live(),
                    again: readmitted,
                })
            });
            let error = match self.metered(route, call, o).await {
                Ok(value) => {
                    self.settle(route, ticket.as_ref(), GateOutcome::Ok, None, o);
                    self.served(route, C::usage(&value), run, TraceEntryBillingReply::Ok, o);
                    return Ok(Tried::Done(Attempt::Served { value, route }));
                }
                Err(error) => error,
            };
            if call.cancel().is_cancelled() {
                self.settle(route, ticket.as_ref(), GateOutcome::Cancelled, None, o);
                return Ok(Tried::Done(Attempt::Failed {
                    outcome: Ended::Stopped,
                    failure: None,
                }));
            }
            let Some(failure) = failure_of(&*error).cloned() else {
                self.settle(route, ticket.as_ref(), GateOutcome::Failed, None, o);
                trace(o, || {
                    TraceEntry::Threw(TraceEntryThrew {
                        entry: route.entry.id.clone(),
                        message: error.to_string(),
                    })
                });
                return Err(error);
            };
            let owed = call
                .billed()
                .and_then(|held| held.usage)
                .or_else(|| failure.billed.as_ref().and_then(|held| held.usage.clone()));
            self.served(route, owed.as_ref(), run, TraceEntryBillingReply::Failed, o);
            // Empty replies stay fresh.
            let fresh = failure.category == FailureCategory::Empty || o.fresh();
            let failed_at = |disposition: Option<Disposition>| {
                trace(o, || {
                    TraceEntry::Failure(TraceEntryFailure {
                        entry: route.entry.id.clone(),
                        model: route.key.clone(),
                        failure: failure.clone(),
                        disposition,
                        fresh,
                        onward,
                    })
                });
            };
            if failure.category == FailureCategory::Overloaded {
                let settled = self.settle(
                    route,
                    ticket.as_ref(),
                    GateOutcome::Overloaded,
                    failure.retry_after_ms,
                    o,
                );
                failed_at(None);
                if onward {
                    return Ok(Tried::Onward(failure));
                }
                if call.live() || settled.is_none() || settled == Some(GateSettled::GaveUp) {
                    return Ok(Tried::Done(Attempt::Failed {
                        outcome: overloaded(route),
                        failure: Some(failure),
                    }));
                }
                if !fresh {
                    return Ok(Tried::Done(Attempt::Failed {
                        outcome: notice(NoticeKind::StreamFailed, Some(&failure), None),
                        failure: Some(failure),
                    }));
                }
                again = true;
                continue;
            }
            self.settle(route, ticket.as_ref(), GateOutcome::Failed, None, o);
            let disposition = route.kind.failure(&failure, Some(&route.view));
            failed_at(Some(disposition.clone()));
            if !fresh {
                if onward && !matches!(disposition, Disposition::Stop(_)) {
                    return Ok(Tried::Onward(failure));
                }
                return Ok(Tried::Done(self.ended(
                    route,
                    failure.clone(),
                    notice(NoticeKind::StreamFailed, Some(&failure), None),
                    o,
                )));
            }
            let (falls, kind) = match &disposition {
                Disposition::Retry(_) => (false, None),
                Disposition::FallThrough(held) => (true, Some(held.notice)),
                Disposition::Stop(held) => (false, Some(held.notice)),
            };
            if let Some(kind) = kind {
                if onward && falls {
                    return Ok(Tried::Onward(failure));
                }
                let outcome = self
                    .notice_for(kind, &failure, route, call.cancel(), o)
                    .await;
                return Ok(Tried::Done(self.ended(route, failure, outcome, o)));
            }
            let wait_ms = if call.live() && attempt >= LIVE_TRIES {
                None
            } else {
                self.retry.wait(failure.retry_after_ms, attempt, waited)
            };
            let Some(wait_ms) = wait_ms else {
                if onward {
                    return Ok(Tried::Onward(failure));
                }
                let outcome = notice(NoticeKind::RetriesExhausted, Some(&failure), None);
                return Ok(Tried::Done(self.ended(route, failure, outcome, o)));
            };
            trace(o, || {
                TraceEntry::Retry(TraceEntryRetry {
                    entry: route.entry.id.clone(),
                    attempt: attempt as u64,
                    wait_ms: wait_ms as u64,
                    why: if failure.retry_after_ms.is_none() {
                        TraceEntryRetryWhy::Backoff
                    } else {
                        TraceEntryRetryWhy::RetryAfter
                    },
                    status: status_of(&failure.status),
                })
            });
            o.on_retry(attempt, wait_ms, &status_of(&failure.status));
            attempt += 1;
            waited += wait_ms;
            let slept = self.scheduler.sleep(wait_ms as i64, call.cancel()).await;
            if slept == nyaaai_contract::time::Slept::Cancelled {
                return Ok(Tried::Done(Attempt::Failed {
                    outcome: Ended::Stopped,
                    failure: None,
                }));
            }
        }
    }

    fn ended<'a, T>(
        &self,
        route: &Resolved,
        failure: Failure,
        outcome: Ended,
        o: &Arc<dyn Observer>,
    ) -> Attempt<'a, T> {
        self.failed(route, &failure, false, o);
        Attempt::Failed {
            outcome,
            failure: Some(failure),
        }
    }

    /// Tells the gate a route failed.
    fn failed(&self, route: &Resolved, failure: &Failure, onward: bool, o: &Arc<dyn Observer>) {
        let Some(gate) = &self.gate else {
            return;
        };
        gate.failed(RouteRef::from(route), failure, onward);
        trace(o, || {
            TraceEntry::GateFailed(TraceEntryGateFailed {
                entry: route.entry.id.clone(),
                failure: failure.clone(),
                onward,
            })
        });
    }

    /// Tells the gate a reply's billing.
    fn served(
        &self,
        route: &Resolved,
        usage: Option<&NormalizedUsage>,
        run: Option<&Mutex<HashSet<String>>>,
        reply: TraceEntryBillingReply,
        o: &Arc<dyn Observer>,
    ) {
        let Some(gate) = &self.gate else {
            return;
        };
        {
            let mut held = run.map(locked);
            gate.served(RouteRef::from(route), usage, held.as_deref_mut());
        }
        trace(o, || {
            TraceEntry::Billing(TraceEntryBilling {
                entry: route.entry.id.clone(),
                reply,
                usage: usage.cloned(),
            })
        });
    }

    async fn metered<C: Call>(
        &self,
        route: &Resolved,
        call: &C,
        o: &Arc<dyn Observer>,
    ) -> Result<C::Value, SendError> {
        let Some(meter) = &self.meter else {
            return call.send(route, &|| {}).await;
        };
        let view = &route.view;
        let at = self.scheduler.now_ms();
        let first: Mutex<Option<f64>> = Mutex::new(None);
        let record = |of: &dyn Fn() -> Metered| {
            let _ = catch_unwind(AssertUnwindSafe(|| {
                let fields = of();
                let done_at = self.scheduler.now_ms();
                let metered = MeteredCall {
                    tag: call.tag().clone(),
                    model: route.key.clone(),
                    entry: Some(route.entry.id.clone()),
                    label: Some(route.entry.label.clone()),
                    at: at as f64,
                    ms: (done_at - at) as f64,
                    first_token_ms: *locked(&first),
                    cost: step_cost(
                        fields.usage.as_ref(),
                        view.price.as_ref(),
                        wants_long_cache(call.tag(), view.cache_write1h.is_some()),
                    ),
                    usage: fields.usage,
                    provider: fields.provider,
                    generation_id: fields.generation_id,
                    finish_reason: fields.finish_reason,
                    failure: fields.failure,
                    cancelled: fields.cancelled,
                };
                trace(o, || {
                    TraceEntry::Metered(TraceEntryMetered {
                        call: metered.clone(),
                    })
                });
                meter.called(&metered);
            }));
        };
        let first_token = || {
            let mut held = locked(&first);
            if held.is_none() {
                *held = Some((self.scheduler.now_ms() - at) as f64);
            }
        };
        let sent = call.send(route, &first_token).await;
        match sent {
            Err(error) => {
                record(&|| {
                    let spent = call.billed();
                    let owed = failure_of(&*error).and_then(|held| held.billed.clone());
                    let pick = |from: fn(&FailureBilled) -> Option<String>| {
                        spent
                            .as_ref()
                            .and_then(from)
                            .or_else(|| owed.as_ref().and_then(from))
                    };
                    let (failure, cancelled) = if call.cancel().is_cancelled() {
                        abandoned(call.ended())
                    } else {
                        (Some(reason_of(&error, route)), None)
                    };
                    Metered {
                        usage: spent
                            .as_ref()
                            .and_then(|held| held.usage.clone())
                            .or_else(|| owed.as_ref().and_then(|held| held.usage.clone())),
                        provider: pick(|held| held.host.clone()),
                        generation_id: pick(|held| held.generation_id.clone()),
                        finish_reason: None,
                        failure,
                        cancelled,
                    }
                });
                Err(error)
            }
            Ok(value) => {
                record(&|| call.read(&value));
                Ok(value)
            }
        }
    }

    /// Live calls never wait.
    fn admit_live<C: Call>(&self, route: &Resolved, call: &C, o: &Arc<dyn Observer>) -> Gated {
        if call.cancel().is_cancelled() {
            return Gated::Stopped;
        }
        let Some(gate) = &self.gate else {
            return Gated::Go(None);
        };
        let ticket = gate.try_admit(RouteRef::from(route));
        let admission = match &ticket {
            None => Admission::GaveUp(nyaaai_contract::shapes::health::AdmissionGaveUp {}),
            Some(held) => Admission::Go(nyaaai_contract::shapes::health::AdmissionGo {
                ticket: held.clone(),
            }),
        };
        trace(o, || {
            TraceEntry::Admitted(TraceEntryAdmitted {
                entry: route.entry.id.clone(),
                admission,
            })
        });
        match ticket {
            Some(held) => Gated::Go(Some(held)),
            None => Gated::GaveUp,
        }
    }

    async fn admit<C: Call>(
        &self,
        route: &Resolved,
        call: &C,
        o: &Arc<dyn Observer>,
        again: bool,
    ) -> Gated {
        let cancel = call.cancel();
        if cancel.is_cancelled() {
            return Gated::Stopped;
        }
        let Some(gate) = &self.gate else {
            return Gated::Go(None);
        };
        let held = Arc::new(AtomicBool::new(false));
        let on_wait: crate::health::gate::WaitFn = {
            let (held, o, entry) = (Arc::clone(&held), Arc::clone(o), route.entry.id.clone());
            Arc::new(move |waiting| {
                held.store(true, Ordering::SeqCst);
                trace(&o, || {
                    TraceEntry::Held(TraceEntryHeld {
                        entry: entry.clone(),
                        waiting: waiting.clone(),
                    })
                });
                o.on_wait(Some(&waiting));
            })
        };
        let admission = gate
            .admit(RouteRef::from(route), cancel, Some(on_wait), again)
            .await;
        trace(o, || {
            TraceEntry::Admitted(TraceEntryAdmitted {
                entry: route.entry.id.clone(),
                admission: admission.clone(),
            })
        });
        if held.load(Ordering::SeqCst) {
            trace(o, || {
                TraceEntry::Released(TraceEntryReleased {
                    entry: route.entry.id.clone(),
                })
            });
            o.on_wait(None);
        }
        match admission {
            Admission::Go(go) => {
                if cancel.is_cancelled() {
                    self.settle(route, Some(&go.ticket), GateOutcome::Cancelled, None, o);
                    Gated::Stopped
                } else {
                    Gated::Go(Some(go.ticket))
                }
            }
            Admission::GaveUp(_) => Gated::GaveUp,
            Admission::Stopped(_) => Gated::Stopped,
        }
    }

    fn settle(
        &self,
        route: &Resolved,
        ticket: Option<&Ticket>,
        outcome: GateOutcome,
        retry_after_ms: Option<i64>,
        o: &Arc<dyn Observer>,
    ) -> Option<GateSettled> {
        let (Some(ticket), Some(gate)) = (ticket, &self.gate) else {
            return None;
        };
        let answer = gate.settle(ticket, outcome, retry_after_ms);
        trace(o, || {
            TraceEntry::Settled(TraceEntrySettled {
                entry: route.entry.id.clone(),
                outcome,
                retry_after_ms,
                answer,
            })
        });
        Some(answer)
    }

    async fn notice_for(
        &self,
        kind: NoticeKind,
        failure: &Failure,
        route: &Resolved,
        cancel: &CancelToken,
        o: &Arc<dyn Observer>,
    ) -> Ended {
        let Ended::Notice(mut held) = notice(kind, Some(failure), Some(route)) else {
            return Ended::Stopped;
        };
        if kind == NoticeKind::Refused && failure.reasons.is_some() {
            held.reasons.clone_from(&failure.reasons);
        }
        if kind == NoticeKind::ModelUnavailable {
            let model = route.view.row.label.clone();
            held.message = Some(unavailable_text(Some(&model), held.entry.as_deref()));
            held.model = Some(model);
        }
        if kind == NoticeKind::SpendLimit {
            let has = route.kind.has_limits();
            let limits = if has {
                route.kind.limits(route.transport.transport(), cancel).await
            } else {
                None
            };
            if has {
                trace(o, || {
                    TraceEntry::Limits(TraceEntryLimits {
                        entry: route.entry.id.clone(),
                        read: limits.is_some(),
                        limit: limits.as_ref().and_then(|held| held.limit),
                        reset: limits.as_ref().and_then(|held| held.reset.clone()),
                    })
                });
            }
            if let Some(limit) = limits.as_ref().and_then(|held| held.limit) {
                held.limit = Some(limit);
            }
            if let Some(reset) = limits.as_ref().and_then(|held| held.reset.clone()) {
                held.reset = Some(reset);
            }
            let waited = failure
                .retry_after_ms
                .map(|after| (self.scheduler.now_ms() + after) as f64);
            let resets_at = if failure.category == FailureCategory::Allowance {
                failure.resets_at.map(|at| at as f64).or(waited)
            } else {
                None
            };
            if let Some(at) = resets_at {
                held.resets_at = Some(at);
            }
        }
        Ended::Notice(held)
    }
}

/// Tracks per-run route state.
pub struct RunRoute {
    d: Arc<Inner>,
    keep: Mutex<HashSet<String>>,
    skipped: Arc<Mutex<HashSet<String>>>,
}

/// An observer that tells an entry's skip once.
struct Once {
    inner: Arc<dyn Observer>,
    skipped: Arc<Mutex<HashSet<String>>>,
}

impl Observer for Once {
    fn fresh(&self) -> bool {
        self.inner.fresh()
    }

    fn on_retry(&self, attempt: i64, wait_ms: f64, status: &ErrorStatus) {
        self.inner.on_retry(attempt, wait_ms, status);
    }

    fn on_wait(&self, waiting: Option<&GateWaiting>) {
        self.inner.on_wait(waiting);
    }

    fn on_skip(&self, skip: &Skip) {
        if !locked(&self.skipped).insert(skip.entry.clone()) {
            return;
        }
        self.inner.on_skip(skip);
    }

    fn traces(&self) -> bool {
        self.inner.traces()
    }

    fn on_trace(&self, entry: TraceEntry) {
        self.inner.on_trace(entry);
    }
}

impl RunRoute {
    pub async fn run<'a, C: Call>(
        &self,
        routes: &'a Routes,
        call: &C,
        o: &Arc<dyn Observer>,
    ) -> Result<Attempt<'a, C::Value>, SendError> {
        let once: Arc<dyn Observer> = Arc::new(Once {
            inner: Arc::clone(o),
            skipped: Arc::clone(&self.skipped),
        });
        let result = self
            .d
            .attempt(
                &routes.candidates,
                &routes.passed,
                Some(&self.keep),
                call,
                &once,
            )
            .await?;
        if let Attempt::Served { route, .. } = &result {
            locked(&self.keep).insert(route.entry.id.clone());
        }
        Ok(result)
    }

    /// Eligible routes, or all.
    pub fn sizing<'a>(&self, candidates: &'a [Resolved]) -> Vec<&'a Resolved> {
        let last = candidates.len().saturating_sub(1);
        let eligible: Vec<&Resolved> = candidates
            .iter()
            .enumerate()
            .filter(|(at, route)| {
                self.d
                    .refusal(route, *at == last, Some(&self.keep))
                    .is_none()
            })
            .map(|(_, route)| route)
            .collect();
        if eligible.is_empty() {
            candidates.iter().collect()
        } else {
            eligible
        }
    }
}
