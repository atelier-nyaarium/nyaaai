//! Request lifecycle cases.

use std::collections::{HashMap, VecDeque};
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::task::{Context, Wake, Waker};

use nyaaai_agentic::codec::Codec;
use nyaaai_agentic::failure::{FailureExtra, failed};
use nyaaai_agentic::health::health::{Billing, Gate, HealthDeps, ProviderHealth};
use nyaaai_agentic::kinds::fetcher::Aborted;
use nyaaai_agentic::lifecycle::lifecycle::{
    Attempt, Call, CallEnd, Ended, LifecycleDeps, Metered, Observer, RequestLifecycle, RunRoute,
    SendError, metered_of,
};
use nyaaai_agentic::lifecycle::retry::RetryPolicy;
use nyaaai_agentic::models::uses::CHAT;
use nyaaai_agentic::models::view::unknown_view;
use nyaaai_agentic::provider::incarnation::EntryRegistry;
use nyaaai_agentic::provider::kind::{
    CatalogDeps, EntryCatalog, KINDS, KindDeps, KindError, KindField, ProviderKind,
};
use nyaaai_agentic::provider::resolver::{ModelResolver, Planned, Resolved, ResolverDeps, Routes};
use nyaaai_agentic::step::{StepAssembler, reply_calls, reply_text};
use nyaaai_agentic::transport::{KeySource, Transport, TransportError};
use nyaaai_agentic::usage::meter::{MeterDeps, provider_meter};
use nyaaai_contract::cancel::CancelToken;
use nyaaai_contract::entry_keys::KeyAnswer;
use nyaaai_contract::fetch::Span;
use nyaaai_contract::future::BoxFuture;
use nyaaai_contract::id::CountedIds;
use nyaaai_contract::json::{parse_json, quote_json};
use nyaaai_contract::model_registry::ModelRegistry;
use nyaaai_contract::random::SequenceRandom;
use nyaaai_contract::shape::Shape;
use nyaaai_contract::shapes::health::{ErrorStatus, GateWaiting, RetryOptions};
use nyaaai_contract::shapes::lifecycle::TraceEntry;
use nyaaai_contract::shapes::model::{
    ModelRow, ModelUse, ModelView, ModelViewPrice, ModelViewStatus, RowIdentity,
};
use nyaaai_contract::shapes::provider::{
    EntryFields, KeyState, KeyStateOk, ProviderEntry, ProviderKindName, Skip,
};
use nyaaai_contract::shapes::step::{
    CanonicalRequest, Failure, FailureBilled, FailureCategory, FinishReason, NormalizedUsage,
    StepEvent, StepResult,
};
use nyaaai_contract::shapes::usage::{CallTag, MeteredCall};
use nyaaai_contract::text::trim_text;
use nyaaai_contract::time::{Clock, Deadline, Ended as DeadlineEnd, deadline};
use nyaaai_testkit::cases::{Outcome, block_on};
use nyaaai_testkit::scheduler::{FakeScheduler, TimerWheel};
use nyaaai_testkit::scripted_fetch::{ScriptedChunk, ScriptedEnd, ScriptedExchange, ScriptedFetch};
use nyaaai_testkit::scripted_ports::{
    ScriptedEntryKeys, ScriptedModelRegistry, ScriptedProviderList, ScriptedUsageSink,
};

use super::catalog::row_of_key;
use super::local::local_row_of;
use super::shapes::*;

const CALLER_REASON: &str = "caller aborted";

fn locked<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

////////////////////////////////
//  Script readers

fn row_for(key: &str) -> Outcome<ModelRow> {
    if key.starts_with("local:") {
        local_row_of(key)
    } else {
        row_of_key(key)
    }
}

/// Remembers an entry the script named, in the order first named.
fn remember(named: &Mutex<Vec<String>>, id: &str) {
    let mut named = locked(named);
    if !named.iter().any(|held| held == id) {
        named.push(id.to_owned());
    }
}

fn key_answer(held: &LifeKeyAnswer) -> KeyAnswer {
    match held {
        LifeKeyAnswer::Key(held) => KeyAnswer::Key(held.key.clone()),
        LifeKeyAnswer::None(_) => KeyAnswer::None,
        LifeKeyAnswer::Refused(_) => KeyAnswer::Refused,
        LifeKeyAnswer::Unready(_) => KeyAnswer::Unready,
    }
}

fn exchange_of(held: &LifeReply) -> ScriptedExchange {
    ScriptedExchange {
        status: held.status as u16,
        headers: held.headers.clone(),
        chunks: held
            .chunks
            .iter()
            .map(|text| ScriptedChunk {
                bytes: text.clone().into_bytes(),
                delay_ms: held.chunk_ms.unwrap_or(0) as i64,
            })
            .collect(),
        head_ms: held.head_ms.unwrap_or(0) as i64,
        end: match held.end {
            None | Some(LifeReplyEnd::Closed) => ScriptedEnd::Closed,
            Some(LifeReplyEnd::Dropped) => ScriptedEnd::Dropped,
            Some(LifeReplyEnd::Refused) => ScriptedEnd::Refused,
        },
        error: held.error.clone(),
    }
}

fn request(model: &str, zdr_only: bool) -> Outcome<CanonicalRequest> {
    let text = format!(
        "{{\"model\":{},\"system\":[],\"history\":[],\"cache\":{{\"kind\":\"none\"}},\"session\":\"session-1\",\"zdrOnly\":{zdr_only}}}",
        quote_json(model)
    );
    let value = parse_json(&text).ok_or_else(|| "the request does not parse".to_owned())?;
    CanonicalRequest::decode(&value).map_err(|refused| refused.to_string())
}

////////////////////////////////
//  The kinds

/// What the catalogs list, by entry and row.
type Listings = HashMap<String, HashMap<String, LifeListing>>;

struct Stub {
    rows: Arc<dyn ModelRegistry>,
    listings: Arc<Listings>,
    zdr: Arc<AtomicBool>,
}

struct StubCatalog {
    name: ProviderKindName,
    entry: ProviderEntry,
    rows: Arc<dyn ModelRegistry>,
    listings: Arc<Listings>,
}

impl EntryCatalog for StubCatalog {
    fn views(
        &self,
        _use: ModelUse,
        zdr_only: bool,
        _cancel: Option<&CancelToken>,
    ) -> BoxFuture<'_, Result<Vec<ModelView>, Aborted>> {
        Box::pin(async move {
            let mut views = Vec::new();
            for row in self.rows.rows() {
                let bound = row.bindings.get(self.name.as_str()).is_some();
                let elsewhere = matches!(
                    &row.identity,
                    RowIdentity::Local(local) if local.entry != self.entry.id
                );
                if !bound || elsewhere {
                    continue;
                }
                let held = self
                    .listings
                    .get(&self.entry.id)
                    .and_then(|rows| rows.get(&row.key));
                if held.is_some_and(|held| held.unlisted.is_some()) {
                    continue;
                }
                let mut view = unknown_view(&row);
                view.status = ModelViewStatus::Ok;
                if let Some(held) = held {
                    if held.keeps.is_some() {
                        if zdr_only {
                            view.status = ModelViewStatus::NoZdrHost;
                        } else {
                            view.data_kept = Some(());
                        }
                    }
                    view.price = held.price.as_ref().map(|price| match price {
                        LifeListingPrice::Price(price) => ModelViewPrice::Price(price.clone()),
                        LifeListingPrice::Free => ModelViewPrice::Free,
                        LifeListingPrice::Unknown => ModelViewPrice::Unknown,
                    });
                    if held.cache_write1h.is_some() {
                        view.cache_write1h = Some(());
                    }
                }
                views.push(view);
            }
            Ok(views)
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn arc_any(self: Arc<Self>) -> Arc<dyn std::any::Any + Send + Sync> {
        self
    }
}

/// The real kind, with a stub catalog or an always-passing key check.
struct Kind {
    real: Arc<dyn ProviderKind>,
    stub: Option<Stub>,
    checked: bool,
}

impl ProviderKind for Kind {
    fn kind(&self) -> ProviderKindName {
        self.real.kind()
    }

    fn label(&self) -> &str {
        self.real.label()
    }

    fn field(&self) -> KindField {
        self.real.field()
    }

    fn optional_key(&self) -> bool {
        self.real.optional_key()
    }

    fn check<'a>(
        &'a self,
        entry: &'a EntryFields,
        secret: Option<&'a str>,
        deps: &'a KindDeps,
        span: &'a Span,
    ) -> BoxFuture<'a, Result<KeyState, KindError>> {
        if self.checked {
            return Box::pin(async {
                Ok(KeyState::Ok(KeyStateOk {
                    usage: None,
                    limit: None,
                    models: None,
                }))
            });
        }
        self.real.check(entry, secret, deps, span)
    }

    fn transport(
        &self,
        entry: &EntryFields,
        key: Arc<dyn KeySource>,
        deps: &KindDeps,
    ) -> Result<Arc<Transport>, String> {
        self.real.transport(entry, key, deps)
    }

    fn catalog(
        &self,
        entry: &ProviderEntry,
        transport: Arc<Transport>,
        deps: CatalogDeps,
    ) -> Arc<dyn EntryCatalog> {
        match &self.stub {
            Some(stub) => {
                let _ = (&stub.zdr, &deps.refreshed);
                Arc::new(StubCatalog {
                    name: self.real.kind(),
                    entry: entry.clone(),
                    rows: Arc::clone(&stub.rows),
                    listings: Arc::clone(&stub.listings),
                })
            }
            None => self.real.catalog(entry, transport, deps),
        }
    }

    fn codec(
        &self,
        view: Option<&ModelView>,
        entry: &EntryFields,
        deps: &KindDeps,
    ) -> Box<dyn Codec> {
        self.real.codec(view, entry, deps)
    }

    fn failure(
        &self,
        failure: &Failure,
        view: Option<&ModelView>,
    ) -> nyaaai_contract::shapes::provider::Disposition {
        self.real.failure(failure, view)
    }

    fn has_limits(&self) -> bool {
        self.real.has_limits()
    }

    fn limits<'a>(
        &'a self,
        transport: &'a Transport,
        span: &'a Span,
    ) -> BoxFuture<'a, Option<nyaaai_contract::shapes::provider::Limits>> {
        self.real.limits(transport, span)
    }
}

fn real_of(name: ProviderKindName) -> Outcome<Arc<dyn ProviderKind>> {
    KINDS
        .iter()
        .find(|held| held.kind() == name)
        .map(Arc::clone)
        .ok_or_else(|| format!("No {} kind.", name.as_str()))
}

////////////////////////////////
//  What a call is told and does

/// What the callbacks and the trace of one call are given.
struct Rec {
    told: Arc<Mutex<Vec<LifeTold>>>,
    trace: Option<Arc<Mutex<Vec<TraceEntry>>>>,
    assembler: Arc<Mutex<StepAssembler>>,
}

impl Observer for Rec {
    fn fresh(&self) -> bool {
        !locked(&self.assembler).output_seen()
    }

    fn on_retry(&self, attempt: i64, wait_ms: f64, status: &ErrorStatus) {
        locked(&self.told).push(LifeTold::Retry(LifeToldRetry {
            attempt,
            wait_ms: wait_ms as i64,
            status: match status {
                ErrorStatus::Number(held) => LifeToldRetryStatus::Number(*held),
                ErrorStatus::Text(held) => LifeToldRetryStatus::Text(held.clone()),
            },
        }));
    }

    fn on_wait(&self, waiting: Option<&GateWaiting>) {
        locked(&self.told).push(LifeTold::Wait(LifeToldWait {
            waiting: waiting.cloned(),
        }));
    }

    fn on_skip(&self, skip: &Skip) {
        locked(&self.told).push(LifeTold::Skip(LifeToldSkip {
            entry: skip.entry.clone(),
            label: skip.label.clone(),
            reason: skip.reason,
        }));
    }

    fn traces(&self) -> bool {
        self.trace.is_some()
    }

    fn on_trace(&self, entry: TraceEntry) {
        if let Some(trace) = &self.trace {
            locked(trace).push(entry);
        }
    }
}

type Attempts = Arc<Mutex<Vec<LifeCallAttemptsItem>>>;

/// One call the script made, sent through a route.
struct StepCall {
    tag: CallTag,
    cancel: CancelToken,
    limit: Option<Arc<Deadline>>,
    live: bool,
    stream: bool,
    throws: Option<String>,
    assembler: Arc<Mutex<StepAssembler>>,
    attempts: Attempts,
    named: Arc<Mutex<Vec<String>>>,
}

fn send_error(error: TransportError) -> SendError {
    match error {
        TransportError::Provider(held) => Box::new(held),
        TransportError::Cancelled => CALLER_REASON.into(),
    }
}

impl Call for StepCall {
    type Value = StepResult;

    fn tag(&self) -> &CallTag {
        &self.tag
    }

    fn cancel(&self) -> &CancelToken {
        &self.cancel
    }

    fn ended(&self) -> Option<CallEnd> {
        self.limit.as_ref().and_then(|limit| {
            limit.ended().map(|ended| match ended {
                DeadlineEnd::Deadline => CallEnd::Deadline,
                DeadlineEnd::Caller => CallEnd::Caller,
            })
        })
    }

    fn live(&self) -> bool {
        self.live
    }

    fn send<'a>(
        &'a self,
        route: &'a Resolved,
        first_token: &'a (dyn Fn() + Send + Sync),
    ) -> BoxFuture<'a, Result<StepResult, SendError>> {
        Box::pin(async move {
            *locked(&self.assembler) = StepAssembler::new();
            let at = {
                let mut attempts = locked(&self.attempts);
                attempts.push(LifeCallAttemptsItem {
                    entry: route.entry.id.clone(),
                    events: Vec::new(),
                });
                attempts.len() - 1
            };
            {
                let mut named = locked(&self.named);
                if !named.contains(&route.entry.id) {
                    named.push(route.entry.id.clone());
                }
            }
            if let Some(message) = &self.throws {
                return Err(message.clone().into());
            }
            let asked = request(&route.binding.id, route.zdr_only())?;
            if !self.stream {
                return route
                    .transport
                    .complete(&*route.codec, &asked, &self.cancel)
                    .await
                    .map_err(send_error);
            }
            let mut stream = route.transport.stream(&*route.codec, asked, &self.cancel);
            loop {
                match stream.next().await {
                    Ok(Some(event)) => {
                        let event: StepEvent = event;
                        locked(&self.attempts)[at].events.push(event.clone());
                        let seen = {
                            let mut assembler = locked(&self.assembler);
                            assembler
                                .add(&event)
                                .map_err(|error| Box::new(error) as SendError)?;
                            assembler.output_seen()
                        };
                        if seen {
                            first_token();
                        }
                    }
                    Ok(None) => break,
                    Err(error) => return Err(send_error(error)),
                }
            }
            let result = locked(&self.assembler).result();
            if result.finish == FinishReason::Error {
                return Err(Box::new(failed(
                    FailureCategory::Stream,
                    "The provider ended the step.",
                    FailureExtra::default(),
                )));
            }
            let cut = matches!(
                result.finish,
                FinishReason::Length | FinishReason::ContentFilter
            );
            if !cut && trim_text(&reply_text(&result)).is_empty() && reply_calls(&result).is_empty()
            {
                return Err(Box::new(failed(
                    FailureCategory::Empty,
                    "The reply was empty.",
                    FailureExtra::default(),
                )));
            }
            Ok(result)
        })
    }

    fn read(&self, value: &StepResult) -> Metered {
        metered_of(value)
    }

    fn usage(value: &StepResult) -> Option<&NormalizedUsage> {
        value.usage.as_ref()
    }

    fn billed(&self) -> Option<FailureBilled> {
        locked(&self.assembler).billed()
    }
}

////////////////////////////////
//  The calls and their wakes

/// Which calls were woken, in the order they were.
type Woken = Arc<Mutex<VecDeque<usize>>>;

struct Wakes {
    index: usize,
    woken: Woken,
}

impl Wake for Wakes {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        let mut woken = locked(&self.woken);
        if !woken.contains(&self.index) {
            woken.push_back(self.index);
        }
    }
}

struct Slot {
    name: String,
    token: CancelToken,
    trace: Option<Arc<Mutex<Vec<TraceEntry>>>>,
    told: Arc<Mutex<Vec<LifeTold>>>,
    attempts: Attempts,
    result: Arc<Mutex<LifeResult>>,
    settled: Arc<AtomicBool>,
    future: Option<Pin<Box<dyn Future<Output = ()>>>>,
    waker: Waker,
}

/// Polls woken calls in order until none is.
fn drive(slots: &mut [Slot], woken: &Woken) {
    loop {
        let Some(index) = locked(woken).pop_front() else {
            return;
        };
        let slot = &mut slots[index];
        let Some(future) = slot.future.as_mut() else {
            continue;
        };
        let mut context = Context::from_waker(&slot.waker);
        if future.as_mut().poll(&mut context).is_ready() {
            slot.future = None;
        }
    }
}

type Plans = HashMap<String, Arc<Routes>>;

pub fn run(input: LifecycleRunInput) -> Outcome<LifecycleRunResult> {
    block_on(play(input))?
}

async fn play(input: LifecycleRunInput) -> Outcome<LifecycleRunResult> {
    let wheel = Arc::new(TimerWheel::new(input.start_ms));
    let scheduler = Arc::new(FakeScheduler::new(Arc::clone(&wheel)));
    let played = Arc::new(ScriptedFetch::new(
        input.exchanges.iter().map(exchange_of).collect(),
        Arc::clone(&scheduler),
    ));
    let scripted = Arc::new(ScriptedEntryKeys::new());
    for (id, answer) in input.keys.iter() {
        scripted.set(id, key_answer(answer));
    }
    let list = Arc::new(ScriptedProviderList::new(input.entries.clone()));
    let entries = Arc::new(EntryRegistry::new(list.clone(), &*scripted));
    let registry = Arc::new(ScriptedModelRegistry::empty());
    registry.set_rows(
        input
            .rows
            .iter()
            .map(|key| row_for(key))
            .collect::<Outcome<Vec<_>>>()?,
    );
    let usage = Arc::new(ScriptedUsageSink::new());
    usage.set_ids(input.usage_ids.clone().unwrap_or_default());
    let zdr = Arc::new(AtomicBool::new(input.zdr_only));
    let deps = KindDeps {
        runtime: nyaaai_testkit::cases::runtime(),
        fetch: played.clone(),
        scheduler: scheduler.clone(),
        log: Arc::new(|_| {}),
        user_agent: "cases/1.0".to_owned(),
        base_url: None,
        budgets: None,
    };
    let mut listings = Listings::new();
    for (entry, rows) in input.listings.iter() {
        let mut held = HashMap::new();
        for (key, listing) in rows.iter() {
            held.insert(key.to_owned(), listing.clone());
        }
        listings.insert(entry.to_owned(), held);
    }
    let listings = Arc::new(listings);
    let mut stubs: Vec<Arc<dyn ProviderKind>> = Vec::new();
    let mut checks: Vec<Arc<dyn ProviderKind>> = Vec::new();
    for name in &input.kinds {
        stubs.push(Arc::new(Kind {
            real: real_of(*name)?,
            stub: Some(Stub {
                rows: registry.clone(),
                listings: Arc::clone(&listings),
                zdr: Arc::clone(&zdr),
            }),
            checked: false,
        }));
        checks.push(Arc::new(Kind {
            real: real_of(*name)?,
            stub: None,
            checked: true,
        }));
    }
    let health = ProviderHealth::new(
        HealthDeps {
            runtime: nyaaai_testkit::cases::runtime(),
            scheduler: scheduler.clone(),
            entries: Arc::clone(&entries),
            keys: scripted.clone(),
            kinds: checks,
            kind_deps: {
                let deps = deps.clone();
                Arc::new(move |_| deps.clone())
            },
            budgets: None,
            pause_ms: input.pause_ms.map(|held| held as i64),
            max_pauses: input.max_pauses,
        },
        &CountedIds::new("gate-"),
    );
    let resolver = ModelResolver::new(ResolverDeps {
        entries: Arc::clone(&entries),
        keys: health.entry_keys(),
        registry: registry.clone() as Arc<dyn ModelRegistry>,
        kinds: stubs,
        kind_deps: {
            let deps = deps.clone();
            Arc::new(move |_| deps.clone())
        },
        zdr_only: {
            let live = Arc::clone(&zdr);
            Arc::new(move || live.load(Ordering::SeqCst))
        },
        refreshed: None,
    });
    let stats: Arc<Mutex<Vec<LifecycleRunResultStatsItem>>> = Arc::default();
    let meter = provider_meter(MeterDeps {
        usage: usage.clone(),
        health: Arc::new(health.clone()),
        stats: {
            let stats = Arc::clone(&stats);
            Arc::new(move |id, generation_id, call: &MeteredCall| {
                locked(&stats).push(LifecycleRunResultStatsItem {
                    id: id.to_owned(),
                    generation_id: generation_id.to_owned(),
                    call: call.clone(),
                });
            })
        },
    });
    let draws = SequenceRandom::new(input.draws.clone().unwrap_or_else(|| vec![1.0]))
        .ok_or_else(|| "no draws".to_owned())?;
    let lifecycle = RequestLifecycle::new(LifecycleDeps {
        gate: (input.gate != Some(false)).then(|| Arc::new(health.clone()) as Arc<dyn Gate>),
        meter: Some(meter),
        retry: Some(RetryPolicy::new(
            &input.retry.clone().unwrap_or(RetryOptions {
                tries: None,
                budget_ms: None,
                cap_ms: None,
                base_ms: None,
            }),
            Arc::new(draws),
        )),
        scheduler: scheduler.clone(),
        budgets: None,
    });

    let mut plans = Plans::new();
    let mut runs: HashMap<String, Arc<RunRoute>> = HashMap::new();
    let named: Arc<Mutex<Vec<String>>> = Arc::default();
    let name = |id: &str| {
        let mut named = locked(&named);
        if !named.iter().any(|held| held == id) {
            named.push(id.to_owned());
        }
    };
    for entry in &input.entries {
        name(&entry.id);
    }
    let woken: Woken = Arc::default();
    let mut laters = 0;
    let mut slots: Vec<Slot> = Vec::new();
    let mut steps = Vec::new();
    let health_of = |named: &Arc<Mutex<Vec<String>>>| -> Vec<String> { locked(named).clone() };

    // Lets the gate sweep waiters whose caller cancelled.
    let sweep = |health: &ProviderHealth, named: &Arc<Mutex<Vec<String>>>| {
        for id in health_of(named) {
            health.state(&id);
        }
    };

    for step in &input.steps {
        let before: Vec<bool> = slots
            .iter()
            .map(|slot| slot.settled.load(Ordering::SeqCst))
            .collect();
        let mut answer = LifeAnswer::None(LifeAnswerNone {});
        match step {
            LifeStep::Dispose(_) => entries.dispose(),
            LifeStep::Send(held) => {
                let routes = match &held.plan {
                    Some(plan) => plans
                        .get(plan)
                        .cloned()
                        .ok_or_else(|| format!("No plan {plan}."))?,
                    None => plan_of(&resolver, &held.model, held.use_.as_ref()).await?,
                };
                let run_route = match &held.run {
                    Some(run) => Some(
                        runs.get(run)
                            .cloned()
                            .ok_or_else(|| format!("No run {run}."))?,
                    ),
                    None => None,
                };
                let token = CancelToken::new();
                if held.aborted_before == Some(true) {
                    token.cancel();
                } else if let Some(after) = held.abort_after_ms {
                    let caller = token.clone();
                    wheel.after_run(
                        after as i64,
                        &format!("abort-{}", held.call),
                        Box::new(move || caller.cancel()),
                    );
                }
                let limit = held
                    .deadline_ms
                    .map(|ms| Arc::new(deadline(&*scheduler, ms as i64, &token)));
                let cancel = limit
                    .as_ref()
                    .map_or_else(|| token.clone(), |limit| limit.token().clone());
                let assembler = Arc::new(Mutex::new(StepAssembler::new()));
                let attempts: Attempts = Arc::default();
                let told: Arc<Mutex<Vec<LifeTold>>> = Arc::default();
                let trace =
                    (input.trace != Some(false)).then(Arc::<Mutex<Vec<TraceEntry>>>::default);
                let call = Arc::new(StepCall {
                    tag: held.tag.clone(),
                    cancel,
                    limit: limit.clone(),
                    live: held.live == Some(true),
                    stream: !matches!(held.mode, Some(LifeStepSendMode::Complete)),
                    throws: held.throws.clone(),
                    assembler: Arc::clone(&assembler),
                    attempts: Arc::clone(&attempts),
                    named: Arc::clone(&named),
                });
                let observer: Arc<dyn Observer> = Arc::new(Rec {
                    told: Arc::clone(&told),
                    trace: trace.clone(),
                    assembler,
                });
                let result = Arc::new(Mutex::new(LifeResult::Open(LifeResultOpen {})));
                let settled = Arc::new(AtomicBool::new(false));
                let index = slots.len();
                let future: Pin<Box<dyn Future<Output = ()>>> = {
                    let (lifecycle, result, settled, wheel) = (
                        lifecycle.clone(),
                        Arc::clone(&result),
                        Arc::clone(&settled),
                        Arc::clone(&wheel),
                    );
                    let call_name = held.call.clone();
                    Box::pin(async move {
                        let ended = match &run_route {
                            None => lifecycle.run(&routes.candidates, &*call, &observer).await,
                            Some(run) => run.run(&routes, &*call, &observer).await,
                        };
                        let outcome = match ended {
                            Ok(Attempt::Served { value, route }) => {
                                LifeResult::Served(LifeResultServed {
                                    entry: route.entry.id.clone(),
                                    reply: value,
                                })
                            }
                            Ok(Attempt::Failed {
                                outcome: Ended::Stopped,
                                ..
                            }) => LifeResult::Stopped(LifeResultStopped {}),
                            Ok(Attempt::Failed {
                                outcome: Ended::Notice(notice),
                                failure,
                            }) => LifeResult::Notice(LifeResultNotice { notice, failure }),
                            Err(error) => LifeResult::Thrown(LifeResultThrown {
                                message: error.to_string(),
                            }),
                        };
                        *locked(&result) = outcome;
                        if let Some(limit) = &limit {
                            limit.clear();
                        }
                        wheel.cancel(&format!("abort-{call_name}"));
                        settled.store(true, Ordering::SeqCst);
                    })
                };
                slots.push(Slot {
                    name: held.call.clone(),
                    token,
                    trace,
                    told,
                    attempts,
                    result,
                    settled,
                    future: Some(future),
                    waker: Waker::from(Arc::new(Wakes {
                        index,
                        woken: Arc::clone(&woken),
                    })),
                });
                locked(&woken).push_back(index);
            }
            LifeStep::Later(held) => {
                laters += 1;
                let (list, scripted, named, change) = (
                    Arc::clone(&list),
                    Arc::clone(&scripted),
                    Arc::clone(&named),
                    held.change.clone(),
                );
                wheel.after_run(
                    held.ms as i64,
                    &format!("later-{laters}"),
                    Box::new(move || match &change {
                        LifeChange::List(held) => {
                            for entry in &held.entries {
                                remember(&named, &entry.id);
                            }
                            list.set(held.entries.clone());
                        }
                        LifeChange::Key(held) => {
                            remember(&named, &held.id);
                            scripted.set(&held.id, key_answer(&held.answer));
                        }
                    }),
                );
            }
            LifeStep::Plan(held) => {
                answer = match resolver
                    .routes(&held.model, held.use_.as_ref().unwrap_or(&CHAT))
                    .await
                {
                    Planned::Routes(routes) => {
                        let line = LifeAnswer::Plan(LifeAnswerPlan {
                            unresolved: None,
                            candidates: routes
                                .candidates
                                .iter()
                                .map(|each| each.entry.id.clone())
                                .collect(),
                            passed: routes.passed.clone(),
                        });
                        plans.insert(held.name.clone(), Arc::new(routes));
                        line
                    }
                    Planned::Unresolved(unresolved) => LifeAnswer::Plan(LifeAnswerPlan {
                        unresolved: Some(unresolved),
                        candidates: Vec::new(),
                        passed: Vec::new(),
                    }),
                };
            }
            LifeStep::Begin(held) => {
                runs.insert(held.run.clone(), Arc::new(lifecycle.begin()));
            }
            LifeStep::Sizing(held) => {
                let routes = plans.get(&held.plan);
                let run = runs.get(&held.run);
                let (Some(routes), Some(run)) = (routes, run) else {
                    return Err("No plan or run to size.".to_owned());
                };
                answer = LifeAnswer::Sizing(LifeAnswerSizing {
                    entries: run
                        .sizing(&routes.candidates)
                        .iter()
                        .map(|each| each.entry.id.clone())
                        .collect(),
                });
            }
            LifeStep::Advance(held) => {
                wheel.advance(held.ms as i64);
            }
            LifeStep::Abort(held) => {
                if let Some(slot) = slots.iter().find(|slot| slot.name == held.call) {
                    slot.token.cancel();
                }
            }
            LifeStep::List(held) => {
                for entry in &held.entries {
                    name(&entry.id);
                }
                list.set(held.entries.clone());
            }
            LifeStep::Key(held) => {
                name(&held.id);
                scripted.set(&held.id, key_answer(&held.answer));
            }
            LifeStep::Zdr(held) => zdr.store(held.on, Ordering::SeqCst),
            LifeStep::Check(held) => {
                name(&held.id);
                health.check(&held.id);
            }
            LifeStep::Billed(held) => {
                name(&held.id);
                health.billed(
                    &held.id,
                    &Billing {
                        source: held.source.clone(),
                        allowance: None,
                        cost: held.cost,
                    },
                    None,
                    None,
                );
            }
        }
        health.idle().await;
        sweep(&health, &named);
        loop {
            drive(&mut slots, &woken);
            health.idle().await;
            sweep(&health, &named);
            if locked(&woken).is_empty() {
                break;
            }
        }
        if played.waited() > 0 {
            return Err(format!(
                "step {} gave two calls the network at once",
                steps.len() + 1
            ));
        }
        steps.push(LifecycleRunResultStepsItem {
            now_ms: wheel.now_ms(),
            answer,
            ended: slots
                .iter()
                .enumerate()
                .filter(|(at, slot)| {
                    slot.settled.load(Ordering::SeqCst)
                        && !before.get(*at).copied().unwrap_or(false)
                })
                .map(|(_, slot)| slot.name.clone())
                .collect(),
            parked: slots
                .iter()
                .filter(|slot| !slot.settled.load(Ordering::SeqCst))
                .map(|slot| slot.name.clone())
                .collect(),
            entries: health_of(&named)
                .iter()
                .map(|id| health.state(id))
                .collect(),
            requests: played.requests().len() as i64,
        });
    }

    let left: Vec<String> = slots
        .iter()
        .filter(|slot| !slot.settled.load(Ordering::SeqCst))
        .map(|slot| slot.name.clone())
        .collect();
    let calls: Vec<LifeCall> = slots
        .iter()
        .map(|slot| LifeCall {
            call: slot.name.clone(),
            result: if left.contains(&slot.name) {
                LifeResult::Open(LifeResultOpen {})
            } else {
                locked(&slot.result).clone()
            },
            trace: slot.trace.as_ref().map(|trace| locked(trace).clone()),
            told: locked(&slot.told).clone(),
            attempts: locked(&slot.attempts).clone(),
        })
        .collect();
    // Parked calls end by the caller's abort.
    for slot in &slots {
        if !slot.settled.load(Ordering::SeqCst) {
            slot.token.cancel();
        }
    }
    sweep(&health, &named);
    loop {
        drive(&mut slots, &woken);
        health.idle().await;
        sweep(&health, &named);
        if locked(&woken).is_empty() {
            break;
        }
    }
    if slots
        .iter()
        .any(|slot| !slot.settled.load(Ordering::SeqCst))
    {
        return Err("A call did not end after its abort.".to_owned());
    }
    health.idle().await;
    let seen = played
        .requests()
        .iter()
        .map(|sent| {
            let header = |name: &str| {
                sent.headers
                    .iter()
                    .find(|(each, _)| each == name)
                    .map(|(_, value)| value.clone())
            };
            LifeSeen {
                url: sent.url.clone(),
                authorization: header("Authorization"),
                accept: header("Accept"),
            }
        })
        .collect();
    resolver.dispose();
    health.dispose();
    entries.dispose();
    let waiting = wheel.waiting();
    let recorded = usage.recorded();
    let stats = std::mem::take(&mut *locked(&stats));
    Ok(LifecycleRunResult {
        steps,
        calls,
        requests: seen,
        recorded,
        stats,
        waiting,
    })
}

async fn plan_of(
    resolver: &ModelResolver,
    model: &str,
    use_: Option<&ModelUse>,
) -> Outcome<Arc<Routes>> {
    match resolver.routes(model, use_.unwrap_or(&CHAT)).await {
        Planned::Routes(routes) => Ok(Arc::new(routes)),
        Planned::Unresolved(_) => Err(format!("No routes for {model}.")),
    }
}

#[cfg(test)]
mod tests {
    use std::task::{Context, Waker};

    use nyaaai_agentic::failure::FailureExtra;
    use nyaaai_agentic::usage::meter::Meter;
    use nyaaai_contract::shapes::health::{
        Admission, AdmissionGaveUp, AdmissionGo, GateOutcome, GateSettled, Ticket,
    };
    use nyaaai_contract::shapes::lifecycle::TraceEntrySkipCause;
    use nyaaai_contract::shapes::provider::{NoticeKind, SkipReason};
    use nyaaai_contract::shapes::step::{FailureStatus, Serving};

    use super::*;

    const ROW: &str = "openai/gpt-6-luna";

    #[derive(Clone, Copy)]
    enum Admits {
        GaveUp,
        Go,
        Never,
    }

    /// A gate that answers as told and notes what it was asked.
    struct Fake {
        admits: Admits,
        tokens: Mutex<Vec<CancelToken>>,
        settled: Mutex<Vec<(Ticket, GateOutcome)>>,
        failed: Mutex<Vec<(String, Failure, bool)>>,
    }

    impl Fake {
        fn new(admits: Admits) -> Arc<Self> {
            Arc::new(Self {
                admits,
                tokens: Mutex::default(),
                settled: Mutex::default(),
                failed: Mutex::default(),
            })
        }
    }

    impl Gate for Fake {
        fn admit(
            &self,
            route: nyaaai_agentic::health::health::RouteRef<'_>,
            cancel: &CancelToken,
            _on_wait: Option<nyaaai_agentic::health::gate::WaitFn>,
            _again: bool,
        ) -> BoxFuture<'static, Admission> {
            locked(&self.tokens).push(cancel.clone());
            let model = route.key.to_owned();
            match self.admits {
                Admits::GaveUp => Box::pin(async { Admission::GaveUp(AdmissionGaveUp {}) }),
                Admits::Go => Box::pin(async move {
                    Admission::Go(AdmissionGo {
                        ticket: Ticket {
                            gate: "gate-1".to_owned(),
                            model,
                            era: 0,
                            generation: 0,
                            probe: false,
                            serial: 1,
                        },
                    })
                }),
                Admits::Never => Box::pin(std::future::pending()),
            }
        }

        fn try_admit(
            &self,
            _route: nyaaai_agentic::health::health::RouteRef<'_>,
        ) -> Option<Ticket> {
            None
        }

        fn settle(
            &self,
            ticket: &Ticket,
            outcome: GateOutcome,
            _retry_after_ms: Option<i64>,
        ) -> GateSettled {
            locked(&self.settled).push((ticket.clone(), outcome));
            GateSettled::Done
        }

        fn deferred(
            &self,
            _route: nyaaai_agentic::health::health::RouteRef<'_>,
            _keep: Option<&std::collections::HashSet<String>>,
        ) -> Option<SkipReason> {
            None
        }

        fn blocked(
            &self,
            _route: nyaaai_agentic::health::health::RouteRef<'_>,
            _keep: Option<&std::collections::HashSet<String>>,
        ) -> Option<nyaaai_contract::shapes::health::Blocked> {
            None
        }

        fn failed(
            &self,
            route: nyaaai_agentic::health::health::RouteRef<'_>,
            failure: &Failure,
            onward: bool,
            _ticket: Option<&Ticket>,
        ) -> bool {
            locked(&self.failed).push((route.entry.id.clone(), failure.clone(), onward));
            false
        }

        fn served(
            &self,
            _route: nyaaai_agentic::health::health::RouteRef<'_>,
            _usage: Option<&NormalizedUsage>,
            _run: Option<&mut std::collections::HashSet<String>>,
            _ticket: Option<&Ticket>,
        ) -> bool {
            false
        }
    }

    #[derive(Default)]
    struct Records(Mutex<Vec<MeteredCall>>);

    impl Meter for Records {
        fn called(
            &self,
            call: &MeteredCall,
            _ticket: Option<&Ticket>,
            _incarnation: Option<&nyaaai_agentic::provider::incarnation::Incarnation>,
        ) {
            locked(&self.0).push(call.clone());
        }
    }

    #[derive(Default)]
    struct Told(Mutex<Vec<TraceEntry>>);

    impl Observer for Told {
        fn traces(&self) -> bool {
            true
        }

        fn on_trace(&self, entry: TraceEntry) {
            locked(&self.0).push(entry);
        }
    }

    #[derive(Clone, Copy)]
    enum Sends {
        Reply,
        Never,
        Down,
    }

    struct Probe {
        cancel: CancelToken,
        sends: Sends,
    }

    impl Call for Probe {
        type Value = StepResult;

        fn tag(&self) -> &CallTag {
            static TAG: std::sync::LazyLock<CallTag> = std::sync::LazyLock::new(|| CallTag {
                feature: "chat".to_owned(),
                lane: None,
                folder: None,
            });
            &TAG
        }

        fn cancel(&self) -> &CancelToken {
            &self.cancel
        }

        fn send<'a>(
            &'a self,
            _route: &'a Resolved,
            _first_token: &'a (dyn Fn() + Send + Sync),
        ) -> BoxFuture<'a, Result<StepResult, SendError>> {
            match self.sends {
                Sends::Reply => Box::pin(async {
                    Ok(StepResult {
                        parts: Vec::new(),
                        finish: FinishReason::Stop,
                        usage: None,
                        serving: Serving {
                            host: None,
                            generation_id: None,
                        },
                    })
                }),
                Sends::Never => Box::pin(std::future::pending()),
                Sends::Down => Box::pin(async {
                    Err(Box::new(failed(
                        FailureCategory::Unavailable,
                        "down",
                        FailureExtra {
                            status: Some(FailureStatus::Number(503.0)),
                            ..FailureExtra::default()
                        },
                    )) as SendError)
                }),
            }
        }

        fn read(&self, value: &StepResult) -> Metered {
            metered_of(value)
        }

        fn usage(value: &StepResult) -> Option<&NormalizedUsage> {
            value.usage.as_ref()
        }
    }

    fn entry(id: &str) -> ProviderEntry {
        ProviderEntry {
            id: id.to_owned(),
            kind: ProviderKindName::Openrouter,
            label: id.to_owned(),
            plan: None,
            url: None,
            extra_usage: None,
        }
    }

    struct Rig {
        resolver: ModelResolver,
        wheel: Arc<TimerWheel>,
        scheduler: Arc<FakeScheduler>,
    }

    fn rig(ids: &[&str]) -> Rig {
        let wheel = Arc::new(TimerWheel::new(0));
        let scheduler = Arc::new(FakeScheduler::new(Arc::clone(&wheel)));
        let deps = KindDeps {
            runtime: nyaaai_testkit::cases::runtime(),
            fetch: Arc::new(ScriptedFetch::new(Vec::new(), Arc::clone(&scheduler))),
            scheduler: scheduler.clone(),
            log: Arc::new(|_| {}),
            user_agent: "cases/1.0".to_owned(),
            base_url: None,
            budgets: None,
        };
        let keys = Arc::new(ScriptedEntryKeys::new());
        for id in ids {
            keys.set(id, KeyAnswer::Key("fake-key-one".to_owned()));
        }
        let registry = Arc::new(ScriptedModelRegistry::empty());
        registry.set_rows(vec![row_of_key(ROW).expect("a curated row")]);
        let stub = Kind {
            real: real_of(ProviderKindName::Openrouter).expect("a kind"),
            stub: Some(Stub {
                rows: registry.clone(),
                listings: Arc::new(Listings::new()),
                zdr: Arc::new(AtomicBool::new(false)),
            }),
            checked: false,
        };
        let entries = Arc::new(EntryRegistry::new(
            Arc::new(ScriptedProviderList::new(
                ids.iter().map(|id| entry(id)).collect(),
            )),
            &*keys,
        ));
        let resolver = ModelResolver::new(ResolverDeps {
            entries,
            keys,
            registry: registry as Arc<dyn ModelRegistry>,
            kinds: vec![Arc::new(stub)],
            kind_deps: Arc::new(move |_| deps.clone()),
            zdr_only: Arc::new(|| false),
            refreshed: None,
        });
        Rig {
            resolver,
            wheel,
            scheduler,
        }
    }

    fn lifecycle_of(rig: &Rig, gate: &Arc<Fake>, records: &Arc<Records>) -> RequestLifecycle {
        RequestLifecycle::new(LifecycleDeps {
            gate: Some(gate.clone() as Arc<dyn Gate>),
            meter: Some(records.clone() as Arc<dyn Meter>),
            retry: None,
            scheduler: rig.scheduler.clone(),
            budgets: None,
        })
    }

    async fn routes_of(rig: &Rig) -> Routes {
        match rig.resolver.routes(ROW, &CHAT).await {
            Planned::Routes(routes) => routes,
            Planned::Unresolved(_) => panic!("no routes"),
        }
    }

    fn kinds_of(told: &Told) -> Vec<&'static str> {
        locked(&told.0)
            .iter()
            .map(|entry| match entry {
                TraceEntry::Admitted(_) => "admitted",
                TraceEntry::GateFailed(_) => "gateFailed",
                TraceEntry::Skip(_) => "skip",
                TraceEntry::Notice(_) => "notice",
                TraceEntry::Attempt(_) => "attempt",
                TraceEntry::Settled(_) => "settled",
                TraceEntry::Metered(_) => "metered",
                _ => "other",
            })
            .collect()
    }

    #[test]
    fn a_give_up_on_a_first_route_falls_through_and_on_the_last_ends_the_request() {
        block_on(async {
            let rig = rig(&["a", "b"]);
            let routes = routes_of(&rig).await;
            let gate = Fake::new(Admits::GaveUp);
            let records = Arc::new(Records::default());
            let told = Arc::new(Told::default());
            let observer: Arc<dyn Observer> = told.clone();
            let call = Probe {
                cancel: CancelToken::new(),
                sends: Sends::Reply,
            };
            let result = lifecycle_of(&rig, &gate, &records)
                .run(&routes.candidates, &call, &observer)
                .await;
            let Ok(Attempt::Failed {
                outcome: Ended::Notice(notice),
                ..
            }) = result
            else {
                panic!("the request should end on a notice");
            };
            assert_eq!(notice.kind, NoticeKind::ProviderOverloaded);
            assert_eq!(
                kinds_of(&told),
                ["admitted", "gateFailed", "skip", "admitted", "notice"]
            );
            let skip = locked(&told.0)
                .iter()
                .find_map(|entry| match entry {
                    TraceEntry::Skip(held) => Some((held.cause, held.reason)),
                    _ => None,
                })
                .expect("a skip");
            assert_eq!(
                skip,
                (TraceEntrySkipCause::FellThrough, SkipReason::Overloaded)
            );
            let failed = locked(&gate.failed).clone();
            assert_eq!(failed.len(), 1);
            assert_eq!(failed[0].0, "a");
            assert_eq!(failed[0].1.category, FailureCategory::Overloaded);
            assert_eq!(
                failed[0].1.status,
                FailureStatus::Text("overloaded".to_owned())
            );
            assert!(failed[0].2);
        })
        .expect("ran");
    }

    fn poll_once<F: Future>(run: &mut Pin<Box<F>>) -> bool {
        run.as_mut()
            .poll(&mut Context::from_waker(Waker::noop()))
            .is_pending()
    }

    #[test]
    fn a_call_dropped_at_the_admission_wait_leaves_the_queue() {
        block_on(async {
            let rig = rig(&["a"]);
            let routes = routes_of(&rig).await;
            let gate = Fake::new(Admits::Never);
            let records = Arc::new(Records::default());
            let observer: Arc<dyn Observer> = Arc::new(Told::default());
            let call = Probe {
                cancel: CancelToken::new(),
                sends: Sends::Reply,
            };
            let lifecycle = lifecycle_of(&rig, &gate, &records);
            let mut run = Box::pin(lifecycle.run(&routes.candidates, &call, &observer));
            assert!(poll_once(&mut run));
            let waiting = locked(&gate.tokens)[0].clone();
            assert!(!waiting.is_cancelled());
            drop(run);
            assert!(waiting.is_cancelled());
            assert!(!call.cancel.is_cancelled());
            assert!(locked(&gate.settled).is_empty());
            assert!(locked(&records.0).is_empty());
        })
        .expect("ran");
    }

    #[test]
    fn a_call_dropped_at_the_send_is_settled_cancelled_and_recorded() {
        block_on(async {
            let rig = rig(&["a"]);
            let routes = routes_of(&rig).await;
            let gate = Fake::new(Admits::Go);
            let records = Arc::new(Records::default());
            let observer: Arc<dyn Observer> = Arc::new(Told::default());
            let call = Probe {
                cancel: CancelToken::new(),
                sends: Sends::Never,
            };
            let lifecycle = lifecycle_of(&rig, &gate, &records);
            let mut run = Box::pin(lifecycle.run(&routes.candidates, &call, &observer));
            assert!(poll_once(&mut run));
            assert!(locked(&gate.settled).is_empty());
            drop(run);
            let settled = locked(&gate.settled).clone();
            assert_eq!(settled.len(), 1);
            assert_eq!(settled[0].1, GateOutcome::Cancelled);
            let records = locked(&records.0).clone();
            assert_eq!(records.len(), 1);
            assert_eq!(records[0].cancelled, Some(true));
            assert_eq!(records[0].failure, None);
        })
        .expect("ran");
    }

    #[test]
    fn a_call_dropped_at_a_retry_sleep_is_settled_and_recorded_once() {
        block_on(async {
            let rig = rig(&["a"]);
            let routes = routes_of(&rig).await;
            let gate = Fake::new(Admits::Go);
            let records = Arc::new(Records::default());
            let observer: Arc<dyn Observer> = Arc::new(Told::default());
            let call = Probe {
                cancel: CancelToken::new(),
                sends: Sends::Down,
            };
            let lifecycle = lifecycle_of(&rig, &gate, &records);
            let mut run = Box::pin(lifecycle.run(&routes.candidates, &call, &observer));
            assert!(poll_once(&mut run));
            assert_eq!(rig.wheel.waiting().len(), 1);
            drop(run);
            let settled = locked(&gate.settled).clone();
            assert_eq!(settled.len(), 1);
            assert_eq!(settled[0].1, GateOutcome::Failed);
            let records = locked(&records.0).clone();
            assert_eq!(records.len(), 1);
            assert_eq!(records[0].cancelled, None);
            assert!(records[0].failure.is_some());
        })
        .expect("ran");
    }
}
