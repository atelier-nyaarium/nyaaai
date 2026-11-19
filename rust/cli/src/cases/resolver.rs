//! Resolver cases.

use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::task::Poll;

use nyaaai_agentic::codec::{Codec, Mode, StreamDecoder};
use nyaaai_agentic::failure::ProviderError;
use nyaaai_agentic::kinds::fetcher::Aborted;
use nyaaai_agentic::models::view::unknown_view;
use nyaaai_agentic::provider::kind::{
    CatalogDeps, EntryCatalog, KINDS, KindDeps, KindError, KindField, ProviderKind,
};
use nyaaai_agentic::provider::resolver::{
    Catalog, ModelResolver, Planned, Resolution, Resolved, ResolverDeps, ServedByDeps,
    key_source_of, reach_of, served_by as plan_served,
};
use nyaaai_agentic::transport::{KeySource, Transport, TransportError, static_key};
use nyaaai_contract::cancel::{CancelToken, Unsubscribe};
use nyaaai_contract::entry_keys::{EntryKeys, KeyAnswer};
use nyaaai_contract::fetch::{Chunks, Fetch, FetchError, FetchRequest, FetchResponse};
use nyaaai_contract::future::BoxFuture;
use nyaaai_contract::json::{Value, parse_json, stringify_json};
use nyaaai_contract::model_registry::ModelRegistry;
use nyaaai_contract::provider_list::ProviderList;
use nyaaai_contract::shape::Shape;
use nyaaai_contract::shapes::model::{ModelRow, ModelUse, ModelView, ModelViewStatus, Via};
use nyaaai_contract::shapes::provider::{EntryFields, ProviderEntry, ProviderKindName};
use nyaaai_contract::shapes::step::{
    CanonicalRequest, Failure, HeaderList, SseItem, StepEvent, StepResult,
};
use nyaaai_contract::time::Scheduler;
use nyaaai_testkit::cases::{Outcome, block_on};
use nyaaai_testkit::scheduler::{FakeScheduler, TimerWheel};
use nyaaai_testkit::scripted_ports::{
    ScriptedEntryKeys, ScriptedModelRegistry, ScriptedProviderList,
};
use tokio::sync::oneshot;

use super::catalog::{convert, row_of_key};
use super::local::local_row_of;
use super::shapes::*;

const KEY: &str = "fake-key-one";

////////////////////////////////
//  Shared pieces

/// A row by key: curated, hosted `provider:id`, or local `local:<entry>/<tag>`.
fn row_for(key: &str) -> Outcome<ModelRow> {
    if key.starts_with("local:") {
        local_row_of(key)
    } else {
        row_of_key(key)
    }
}

fn rows_for(keys: &[String]) -> Outcome<Vec<ModelRow>> {
    keys.iter().map(|key| row_for(key)).collect()
}

/// A key answer, from any of the generated spellings of one.
fn answer_of<T: Shape>(held: &T) -> Outcome<KeyAnswer> {
    let Value::Object(fields) = held.encode() else {
        return Err("a key answer is an object".to_owned());
    };
    let kind = fields.get("kind").and_then(Value::as_str);
    Ok(match kind {
        Some("key") => KeyAnswer::Key(
            fields
                .get("key")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned(),
        ),
        Some("none") => KeyAnswer::None,
        Some("refused") => KeyAnswer::Refused,
        Some("unready") => KeyAnswer::Unready,
        _ => return Err("an unknown key answer".to_owned()),
    })
}

/// The name a route is told by.
fn entry_label(entry: &ProviderEntry) -> String {
    format!(
        "{}{}{}",
        entry.label,
        entry
            .plan
            .map_or(String::new(), |plan| format!("/{}", plan.as_str())),
        entry
            .url
            .as_deref()
            .map_or(String::new(), |url| format!(" {url}")),
    )
}

fn fields_of(entry: &ProviderEntry) -> EntryFields {
    EntryFields {
        kind: entry.kind,
        label: entry.label.clone(),
        plan: entry.plan,
        url: entry.url.clone(),
        extra_usage: entry.extra_usage,
    }
}

fn request(model: &str) -> Outcome<CanonicalRequest> {
    let text = format!(
        "{{\"model\":{},\"system\":[],\"history\":[],\"cache\":{{\"kind\":\"none\"}},\"session\":\"session-1\",\"zdrOnly\":true}}",
        nyaaai_contract::json::quote_json(model)
    );
    let value = parse_json(&text).ok_or_else(|| "the request does not parse".to_owned())?;
    CanonicalRequest::decode(&value).map_err(|refused| refused.to_string())
}

fn locked<T>(held: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    held.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

////////////////////////////////
//  The host

struct Body(Option<Vec<u8>>);

impl Chunks for Body {
    fn next(&mut self) -> BoxFuture<'_, Result<Option<Vec<u8>>, FetchError>> {
        let held = self.0.take();
        Box::pin(async move { Ok(held) })
    }
}

/// A host that answers success or the told failure, recording each URL and `Authorization`.
#[derive(Default)]
struct Host {
    seen: Mutex<Vec<(String, Option<String>)>>,
    next: Mutex<Option<(i64, Value)>>,
}

impl Host {
    /// The next request answers this.
    fn fail(&self, failure: Option<(i64, Value)>) {
        *locked(&self.next) = failure;
    }
}

impl Fetch for Host {
    fn fetch(
        &self,
        request: FetchRequest,
        _cancel: CancelToken,
    ) -> BoxFuture<'_, Result<FetchResponse, FetchError>> {
        let header = |name: &str| {
            request
                .headers
                .iter()
                .find(|(each, _)| each == name)
                .map(|(_, value)| value.clone())
        };
        locked(&self.seen).push((request.url.clone(), header("Authorization")));
        let asked = header("Accept").unwrap_or_default();
        let failure = locked(&self.next).take();
        let stream = failure.is_none() && asked == "text/event-stream";
        let (status, body) = match &failure {
            None => (
                200,
                if stream {
                    "data: {}\n\n".to_owned()
                } else {
                    "{}".to_owned()
                },
            ),
            Some((status, body)) => (*status, stringify_json(body)),
        };
        Box::pin(async move {
            Ok(FetchResponse {
                status: status as u16,
                headers: vec![(
                    "content-type".to_owned(),
                    if stream {
                        "text/event-stream"
                    } else {
                        "application/json"
                    }
                    .to_owned(),
                )],
                chunks: Box::new(Body(Some(body.into_bytes()))),
            })
        })
    }
}

/// A decoder that reads items and says nothing.
struct Silent;

impl StreamDecoder for Silent {
    fn push(&mut self, _item: &SseItem, _events: &mut Vec<StepEvent>) -> Result<(), ProviderError> {
        Ok(())
    }

    fn over(&self) -> bool {
        false
    }

    fn end(&mut self, _events: &mut Vec<StepEvent>) -> Result<(), ProviderError> {
        Ok(())
    }
}

/// A candidate's codec, noting its switch; its reads carry nothing.
struct Probe<'a> {
    inner: &'a dyn Codec,
    zdr_only: Mutex<Option<bool>>,
}

impl Codec for Probe<'_> {
    fn encode(
        &self,
        request: &CanonicalRequest,
        mode: Mode,
    ) -> Result<nyaaai_contract::shapes::step::Encoded, String> {
        *locked(&self.zdr_only) = Some(request.zdr_only);
        self.inner.encode(request, mode)
    }

    fn stream(&self) -> Box<dyn StreamDecoder + '_> {
        Box::new(Silent)
    }

    fn complete(&self, _body: &Value) -> Result<StepResult, ProviderError> {
        Ok(StepResult {
            parts: Vec::new(),
            finish: nyaaai_contract::shapes::step::FinishReason::Stop,
            usage: None,
            serving: nyaaai_contract::shapes::step::Serving {
                host: None,
                generation_id: None,
            },
        })
    }

    fn failure(&self, status: i64, body: &Value, headers: &HeaderList) -> Failure {
        self.inner.failure(status, body, headers)
    }
}

type Sent = ResolverRoutesResultStepsItemAnswerSent;

/// A request through a route: its codec's input and what the host saw.
async fn send_through(
    found: &Resolved,
    host: &Host,
    mode: Option<bool>,
    fail: Option<(i64, Value)>,
) -> Outcome<Sent> {
    let probe = Probe {
        inner: &*found.codec,
        zdr_only: Mutex::new(None),
    };
    let before = locked(&host.seen).len();
    host.fail(fail);
    let token = CancelToken::new();
    let asked = request(&found.key)?;
    let outcome: Result<(), TransportError> = if mode == Some(true) {
        let mut stream = found.transport.stream(&probe, asked, &token);
        loop {
            match stream.next().await {
                Ok(Some(_)) => {}
                Ok(None) => break Ok(()),
                Err(error) => break Err(error),
            }
        }
    } else {
        found
            .transport
            .complete(&probe, &asked, &token)
            .await
            .map(|_| ())
    };
    host.fail(None);
    let (failure, thrown) = match outcome {
        Ok(()) => (None, None),
        Err(TransportError::Provider(held)) => (Some((*held.failure).clone()), None),
        Err(error) => (None, Some(error.to_string())),
    };
    let sent = locked(&host.seen)
        .get(before..)
        .and_then(|each| each.last().cloned());
    let zdr_only = *locked(&probe.zdr_only);
    Ok(Sent {
        zdr_only,
        url: sent.as_ref().map(|(url, _)| url.clone()),
        authorization: sent.and_then(|(_, authorization)| authorization),
        failure,
        thrown,
    })
}

////////////////////////////////
//  The stubs

/// What a catalog lists of a row for an entry.
#[derive(Clone, Copy)]
enum Listed {
    Unlisted,
    Status(ModelViewStatus),
}

#[derive(Clone, Copy)]
struct Listing {
    status: Listed,
    when_zdr_off: Option<Listed>,
}

#[derive(Default)]
struct Hub {
    /// Routes made in the current step, by entry id and label.
    made: Mutex<Vec<(String, String)>>,
    stopped: Mutex<Vec<String>>,
    refreshes: Mutex<HashMap<String, Arc<dyn Fn() + Send + Sync>>>,
    listings: HashMap<String, HashMap<String, Listing>>,
    dispose_fails: Vec<String>,
    /// Entries whose catalog answers no view until a `warm` step.
    cold: Mutex<HashSet<String>>,
    /// The views waiting on a cold catalog.
    waiting: Mutex<Vec<oneshot::Sender<()>>>,
    /// Told when a cold catalog is first asked.
    asked: Mutex<Option<oneshot::Sender<()>>>,
    /// Entries whose catalog answers after this many milliseconds on the wheel.
    slow: HashMap<String, u64>,
    scheduler: Option<Arc<FakeScheduler>>,
}

impl Hub {
    /// Tells a first ask, then waits on the wheel.
    async fn delayed(&self, ms: Option<u64>) {
        let Some(ms) = ms else {
            return;
        };
        if let Some(told) = locked(&self.asked).take() {
            let _ = told.send(());
        }
        if let Some(scheduler) = &self.scheduler {
            let _ = scheduler.sleep(ms as i64, &CancelToken::new()).await;
        }
    }
}

struct StubCatalog {
    name: ProviderKindName,
    entry: ProviderEntry,
    rows: Arc<dyn ModelRegistry>,
    hub: Arc<Hub>,
}

impl EntryCatalog for StubCatalog {
    fn views(
        &self,
        _use: ModelUse,
        zdr_only: bool,
        _cancel: Option<&CancelToken>,
    ) -> BoxFuture<'_, Result<Vec<ModelView>, Aborted>> {
        Box::pin(async move {
            if locked(&self.hub.cold).contains(&self.entry.id) {
                if let Some(told) = locked(&self.hub.asked).take() {
                    let _ = told.send(());
                }
                let (release, wait) = oneshot::channel();
                locked(&self.hub.waiting).push(release);
                let _ = wait.await;
            }
            self.hub
                .delayed(self.hub.slow.get(&self.entry.id).copied())
                .await;
            let mut views = Vec::new();
            for row in self.rows.rows() {
                let bound = row.bindings.get(self.name.as_str()).is_some();
                let elsewhere = matches!(
                    &row.identity,
                    nyaaai_contract::shapes::model::RowIdentity::Local(local) if local.entry != self.entry.id
                );
                if !bound || elsewhere {
                    continue;
                }
                let held = self
                    .hub
                    .listings
                    .get(&self.entry.id)
                    .and_then(|rows| rows.get(&row.key));
                let status = match held {
                    None => Listed::Status(ModelViewStatus::Ok),
                    Some(held) if zdr_only => held.status,
                    Some(held) => held.when_zdr_off.unwrap_or(held.status),
                };
                if let Listed::Status(status) = status {
                    let mut view = unknown_view(&row);
                    view.status = status;
                    views.push(view);
                }
            }
            Ok(views)
        })
    }

    fn dispose(&self) {
        locked(&self.hub.stopped).push(entry_label(&self.entry));
        if self.hub.dispose_fails.contains(&self.entry.id) {
            std::panic::resume_unwind(Box::new("Stopping failed."));
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn arc_any(self: Arc<Self>) -> Arc<dyn std::any::Any + Send + Sync> {
        self
    }
}

/// The real kind with a stub catalog.
struct Stub {
    real: Arc<dyn ProviderKind>,
    hub: Arc<Hub>,
}

impl ProviderKind for Stub {
    fn kind(&self) -> ProviderKindName {
        self.real.kind()
    }

    fn label(&self) -> &str {
        self.real.label()
    }

    fn field(&self) -> KindField {
        self.real.field()
    }

    fn check<'a>(
        &'a self,
        entry: &'a EntryFields,
        secret: Option<&'a str>,
        deps: &'a KindDeps,
        cancel: &'a CancelToken,
    ) -> BoxFuture<'a, Result<nyaaai_contract::shapes::provider::KeyState, KindError>> {
        self.real.check(entry, secret, deps, cancel)
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
        _transport: Arc<Transport>,
        deps: CatalogDeps,
    ) -> Arc<dyn EntryCatalog> {
        locked(&self.hub.made).push((entry.id.clone(), entry_label(entry)));
        locked(&self.hub.refreshes).insert(entry.id.clone(), Arc::clone(&deps.refreshed));
        Arc::new(StubCatalog {
            name: self.real.kind(),
            entry: entry.clone(),
            rows: deps.rows,
            hub: Arc::clone(&self.hub),
        })
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
}

fn real_of(name: ProviderKindName) -> Outcome<Arc<dyn ProviderKind>> {
    KINDS
        .iter()
        .find(|held| held.kind() == name)
        .map(Arc::clone)
        .ok_or_else(|| format!("No {} kind.", name.as_str()))
}

fn stub_of(name: ProviderKindName, hub: &Arc<Hub>) -> Outcome<Arc<dyn ProviderKind>> {
    Ok(Arc::new(Stub {
        real: real_of(name)?,
        hub: Arc::clone(hub),
    }))
}

fn deps_of(host: &Arc<Host>, scheduler: Arc<FakeScheduler>) -> KindDeps {
    KindDeps {
        runtime: nyaaai_testkit::cases::runtime(),
        fetch: host.clone(),
        scheduler,
        log: Arc::new(|_| {}),
        user_agent: "cases/1.0".to_owned(),
        base_url: None,
    }
}

/// A key read held at its first call until released.
struct Gate {
    reached: oneshot::Sender<()>,
    release: oneshot::Receiver<()>,
}

/// The scripted keys; a failing store answers `unready`.
struct Keys {
    inner: Arc<ScriptedEntryKeys>,
    failing: Vec<String>,
    gate: Mutex<Option<Gate>>,
    hub: Arc<Hub>,
    /// Entries whose key read answers after this many milliseconds on the wheel.
    slow: HashMap<String, u64>,
}

impl EntryKeys for Keys {
    fn get(&self, id: &str) -> BoxFuture<'_, KeyAnswer> {
        let gate = locked(&self.gate).take();
        let id = id.to_owned();
        Box::pin(async move {
            if let Some(gate) = gate {
                let _ = gate.reached.send(());
                let _ = gate.release.await;
            }
            self.hub.delayed(self.slow.get(&id).copied()).await;
            if self.failing.contains(&id) {
                KeyAnswer::Unready
            } else {
                self.inner.get(&id).await
            }
        })
    }

    fn on_change(&self, listener: Box<dyn Fn(&str) + Send + Sync>) -> Unsubscribe {
        self.inner.on_change(listener)
    }
}

////////////////////////////////
//  Cases

pub fn reach(input: ResolverReachInput) -> Outcome<ResolverReachResult> {
    let mut reaches = Vec::new();
    for row in &input.rows {
        let field = match row.field {
            ResolverReachInputRowsItemField::Key => KindField::Key,
            ResolverReachInputRowsItemField::Url => KindField::Url,
        };
        reaches.push(reach_of(field, &answer_of(&row.answer)?));
    }
    Ok(ResolverReachResult { reaches })
}

pub fn route_key(input: ResolverRouteKeyInput) -> Outcome<ResolverRouteKeyResult> {
    Ok(ResolverRouteKeyResult {
        keys: input
            .entries
            .iter()
            .map(nyaaai_agentic::provider::resolver::route_key)
            .collect(),
    })
}

pub fn key_source(input: ResolverKeySourceInput) -> Outcome<ResolverKeySourceResult> {
    let keys = Arc::new(ScriptedEntryKeys::new());
    for (id, answer) in input.answers.iter() {
        keys.set(id, answer_of(answer)?);
    }
    let list = Arc::new(ScriptedProviderList::new(input.list.clone()));
    let source = key_source_of(keys.clone(), list.clone(), input.entry.clone());
    let mut gets = Vec::new();
    for step in &input.steps {
        match step {
            ResolverKeySourceInputStepsItem::Get(_) => {
                gets.push(block_on(source.get())?);
            }
            ResolverKeySourceInputStepsItem::List(held) => list.set(held.entries.clone()),
            ResolverKeySourceInputStepsItem::Answer(held) => {
                keys.set(&held.id, answer_of(&held.answer)?);
            }
        }
    }
    Ok(ResolverKeySourceResult { gets })
}

struct Standing {
    status: Option<ModelViewStatus>,
}

impl Catalog for Standing {
    fn view<'a>(&'a self, key: &'a str, _use: ModelUse) -> BoxFuture<'a, Option<ModelView>> {
        let status = self.status;
        let row = row_for(key);
        Box::pin(async move {
            let status = status?;
            let mut view = unknown_view(&row.ok()?);
            view.status = status;
            Some(view)
        })
    }
}

pub fn served_by(input: ResolverServedByInput) -> Outcome<ResolverServedByResult> {
    let named = real_of(input.kind)?;
    let host = Arc::new(Host::default());
    let deps = deps_of(
        &host,
        Arc::new(FakeScheduler::new(Arc::new(TimerWheel::new(0)))),
    );
    let zdr = Arc::new(AtomicBool::new(input.zdr_only));
    let transport = named.transport(&fields_of(&input.entry), Arc::new(static_key(KEY)), &deps)?;
    let live = Arc::clone(&zdr);
    let status = match &input.view {
        None => None,
        Some(held) => Some(convert::<ResolverServedByInputView, ModelViewStatus>(held)?),
    };
    let served = ServedByDeps {
        entry: input.entry.clone(),
        kind: Arc::clone(&named),
        transport,
        catalog: Arc::new(Standing { status }),
        deps,
        zdr_only: Arc::new(move || live.load(Ordering::SeqCst)),
    };
    let routes = block_on(plan_served(&served, &input.key, &input.use_))?;
    let first = routes
        .candidates
        .first()
        .ok_or_else(|| "A plan serves its entry.".to_owned())?;
    let zdr_on_resolve = first.zdr_only();
    zdr.store(input.flipped_to, Ordering::SeqCst);
    let sent = block_on(send_through(first, &host, None, None))??;
    let path = first.codec.encode(&request("m")?, Mode::Complete)?.path;
    Ok(ResolverServedByResult {
        key: routes.key.clone(),
        entry: first.entry.id.clone(),
        binding: first.binding.id.clone(),
        view: first.view.clone(),
        path: path.clone(),
        passed: routes.passed.clone(),
        candidates: routes.candidates.len() as i64,
        zdr_on_resolve,
        sent: ResolverServedByResultSent {
            zdr_only: sent.zdr_only.unwrap_or(false),
            path,
        },
    })
}

fn listed_of<T: Shape>(held: &T) -> Outcome<Listed> {
    match held.encode() {
        Value::String(text) if text == "unlisted" => Ok(Listed::Unlisted),
        Value::String(text) => ModelViewStatus::parse(&text)
            .map(Listed::Status)
            .ok_or_else(|| format!("an unknown status {text}")),
        _ => Err("a listing status is text".to_owned()),
    }
}

/// A route as a request sees it.
struct Candidate {
    entry: String,
    kind: ProviderKindName,
    binding: String,
    status: ModelViewStatus,
    zdr_only: bool,
    path: String,
}

fn candidate_of(resolved: &Resolved) -> Outcome<Candidate> {
    Ok(Candidate {
        entry: resolved.entry.id.clone(),
        kind: resolved.kind.kind(),
        binding: resolved.binding.id.clone(),
        status: resolved.view.status,
        zdr_only: resolved.zdr_only(),
        path: resolved.codec.encode(&request("m")?, Mode::Complete)?.path,
    })
}

type Answer = ResolverRoutesResultStepsItemAnswer;

fn changed() -> Answer {
    Answer::Changed(ResolverRoutesResultStepsItemAnswerChanged {})
}

fn candidate_line(
    held: &Candidate,
) -> Outcome<ResolverRoutesResultStepsItemAnswerRoutesCandidatesItem> {
    Ok(ResolverRoutesResultStepsItemAnswerRoutesCandidatesItem {
        entry: held.entry.clone(),
        kind: held.kind,
        binding: held.binding.clone(),
        status: convert(&held.status)?,
        zdr_only: held.zdr_only,
        path: held.path.clone(),
    })
}

/// A routes answer as the answer of a call that waited.
fn waited_of(
    held: ResolverRoutesResultStepsItemAnswerRoutes,
) -> ResolverRoutesResultStepsItemAnswerWaitedRoutes {
    ResolverRoutesResultStepsItemAnswerWaitedRoutes {
        unresolved: held.unresolved,
        candidates: held
            .candidates
            .into_iter()
            .map(|each| ResolverRoutesResultStepsItemAnswerWaitedRoutesCandidatesItem {
                entry: each.entry,
                kind: each.kind,
                binding: each.binding,
                status: match each.status {
                    ResolverRoutesResultStepsItemAnswerRoutesCandidatesItemStatus::Ok => {
                        ResolverRoutesResultStepsItemAnswerWaitedRoutesCandidatesItemStatus::Ok
                    }
                    ResolverRoutesResultStepsItemAnswerRoutesCandidatesItemStatus::Retired => {
                        ResolverRoutesResultStepsItemAnswerWaitedRoutesCandidatesItemStatus::Retired
                    }
                    ResolverRoutesResultStepsItemAnswerRoutesCandidatesItemStatus::NoZdrHost => {
                        ResolverRoutesResultStepsItemAnswerWaitedRoutesCandidatesItemStatus::NoZdrHost
                    }
                    ResolverRoutesResultStepsItemAnswerRoutesCandidatesItemStatus::NoTools => {
                        ResolverRoutesResultStepsItemAnswerWaitedRoutesCandidatesItemStatus::NoTools
                    }
                    ResolverRoutesResultStepsItemAnswerRoutesCandidatesItemStatus::Unknown => {
                        ResolverRoutesResultStepsItemAnswerWaitedRoutesCandidatesItemStatus::Unknown
                    }
                },
                zdr_only: each.zdr_only,
                path: each.path,
            })
            .collect(),
        passed: held.passed,
    }
}

fn resolve_line(held: &Candidate) -> Outcome<ResolverRoutesResultStepsItemAnswerResolveCandidate> {
    Ok(ResolverRoutesResultStepsItemAnswerResolveCandidate {
        entry: held.entry.clone(),
        kind: held.kind,
        binding: held.binding.clone(),
        status: convert(&held.status)?,
        zdr_only: held.zdr_only,
        path: held.path.clone(),
    })
}

/// A nested step as a step of its own.
fn step_of<T: Shape>(held: &T) -> Outcome<ResolverRoutesInputStepsItem> {
    ResolverRoutesInputStepsItem::decode(&held.encode()).map_err(|refused| refused.to_string())
}

/// What the script's ports and resolver stand on.
struct Rig {
    list: Arc<ScriptedProviderList>,
    keys: Arc<ScriptedEntryKeys>,
    registry: Arc<ScriptedModelRegistry>,
    zdr: Arc<AtomicBool>,
    host: Arc<Host>,
    hub: Arc<Hub>,
    resolver: ModelResolver,
    refreshed: Arc<Mutex<Vec<Via>>>,
    gate: Arc<Keys>,
    wheel: Arc<TimerWheel>,
}

impl Rig {
    /// Lets the views waiting on a cold catalog go on.
    fn release(&self) {
        for release in std::mem::take(&mut *locked(&self.hub.waiting)) {
            let _ = release.send(());
        }
    }

    fn change(&self, step: &ResolverRoutesInputStepsItem) -> Outcome<()> {
        match step {
            ResolverRoutesInputStepsItem::List(each) => self.list.set(each.entries.clone()),
            ResolverRoutesInputStepsItem::Answer(each) => {
                self.keys.set(&each.id, answer_of(&each.answer)?);
            }
            ResolverRoutesInputStepsItem::Registry(each) => {
                self.registry.set_rows(rows_for(&each.rows)?);
            }
            ResolverRoutesInputStepsItem::Zdr(each) => self.zdr.store(each.on, Ordering::SeqCst),
            _ => return Err("not a change".to_owned()),
        }
        Ok(())
    }

    /// A call on the resolver, or through a candidate an earlier step gave.
    async fn called(
        &self,
        held: &mut [Vec<Resolved>],
        step: &ResolverRoutesInputStepsItem,
    ) -> Outcome<Answer> {
        Ok(match step {
            ResolverRoutesInputStepsItem::Routes(each) => {
                match self.resolver.routes(&each.model, &each.use_).await {
                    Planned::Routes(routes) => {
                        let mut candidates = Vec::new();
                        for resolved in &routes.candidates {
                            candidates.push(candidate_line(&candidate_of(resolved)?)?);
                        }
                        let passed = routes.passed.clone();
                        if let Some(last) = held.last_mut() {
                            *last = routes.candidates;
                        }
                        Answer::Routes(ResolverRoutesResultStepsItemAnswerRoutes {
                            unresolved: None,
                            candidates,
                            passed,
                        })
                    }
                    Planned::Unresolved(unresolved) => {
                        Answer::Routes(ResolverRoutesResultStepsItemAnswerRoutes {
                            unresolved: Some(unresolved),
                            candidates: Vec::new(),
                            passed: Vec::new(),
                        })
                    }
                }
            }
            ResolverRoutesInputStepsItem::Resolve(each) => {
                match self.resolver.resolve(&each.model, &each.use_).await {
                    Resolution::Resolved(resolved) => {
                        let line = resolve_line(&candidate_of(&resolved)?)?;
                        if let Some(last) = held.last_mut() {
                            *last = vec![*resolved];
                        }
                        Answer::Resolve(ResolverRoutesResultStepsItemAnswerResolve {
                            unresolved: None,
                            candidate: Some(line),
                        })
                    }
                    Resolution::Unresolved(unresolved) => {
                        Answer::Resolve(ResolverRoutesResultStepsItemAnswerResolve {
                            unresolved: Some(unresolved),
                            candidate: None,
                        })
                    }
                }
            }
            ResolverRoutesInputStepsItem::Rows(each) => {
                let mut options = Vec::new();
                for option in self.resolver.rows(&each.use_).await {
                    options.push(ResolverRoutesResultStepsItemAnswerRowsOptionsItem {
                        key: option.key,
                        status: convert(&option.view.status)?,
                        via: option.via,
                    });
                }
                Answer::Rows(ResolverRoutesResultStepsItemAnswerRows { options })
            }
            ResolverRoutesInputStepsItem::Candidates(each) => {
                let mut served = Vec::new();
                for option in self.resolver.candidates(&each.use_).await {
                    served.push(ResolverRoutesResultStepsItemAnswerCandidatesServedItem {
                        key: option.key,
                        status: convert(&option.view.status)?,
                        via: option
                            .via
                            .ok_or_else(|| "a served row has a via".to_owned())?,
                    });
                }
                Answer::Candidates(ResolverRoutesResultStepsItemAnswerCandidates { served })
            }
            ResolverRoutesInputStepsItem::Send(each) => {
                let Some(found) = held
                    .get(each.from as usize)
                    .and_then(|from| from.get(each.candidate as usize))
                else {
                    return Ok(Answer::None(ResolverRoutesResultStepsItemAnswerNone {}));
                };
                let mode = each
                    .mode
                    .as_ref()
                    .map(|mode| matches!(mode, ResolverRoutesInputStepsItemSendMode::Stream));
                let fail = each
                    .fail
                    .as_ref()
                    .map(|fail| (fail.status, fail.body.clone()));
                Answer::Sent(send_through(found, &self.host, mode, fail).await?)
            }
            _ => return Err("not a call".to_owned()),
        })
    }

    async fn said(
        &self,
        held: &mut [Vec<Resolved>],
        step: &ResolverRoutesInputStepsItem,
    ) -> Outcome<Answer> {
        match step {
            ResolverRoutesInputStepsItem::List(_)
            | ResolverRoutesInputStepsItem::Answer(_)
            | ResolverRoutesInputStepsItem::Registry(_)
            | ResolverRoutesInputStepsItem::Zdr(_) => {
                self.change(step)?;
                Ok(changed())
            }
            ResolverRoutesInputStepsItem::Refreshed(each) => {
                let told = locked(&self.hub.refreshes).get(&each.entry).cloned();
                if let Some(told) = told {
                    told();
                }
                Ok(changed())
            }
            ResolverRoutesInputStepsItem::Warm(each) => {
                locked(&self.hub.cold).remove(&each.entry);
                self.release();
                Ok(changed())
            }
            ResolverRoutesInputStepsItem::After(each) => {
                let (told, arrived) = oneshot::channel();
                *locked(&self.hub.asked) = Some(told);
                let step =
                    ResolverRoutesInputStepsItem::Routes(ResolverRoutesInputStepsItemRoutes {
                        model: each.model.clone(),
                        use_: each.use_.clone(),
                    });
                let mut call = Box::pin(self.called(held, &step));
                let mut arrived = arrived;
                // The call runs until a cold catalog is asked, or ends.
                let first = std::future::poll_fn(|context| {
                    if let Poll::Ready(done) = call.as_mut().poll(context) {
                        return Poll::Ready(Some(done));
                    }
                    if Pin::new(&mut arrived).poll(context).is_ready() {
                        return Poll::Ready(None);
                    }
                    Poll::Pending
                })
                .await;
                self.wheel.advance(each.ms as i64 - 1);
                let timers = self.wheel.waiting().len() as i64;
                self.wheel.advance(1);
                for entry in &each.warmed {
                    locked(&self.hub.cold).remove(entry);
                }
                if !each.warmed.is_empty() {
                    self.release();
                }
                let answered = match first {
                    Some(done) => done?,
                    None => call.await?,
                };
                *locked(&self.hub.asked) = None;
                // Reads that went on past their bound run once before the step is read.
                let _ = nyaaai_testkit::cases::runtime().spawn(async {}).await;
                let Answer::Routes(routes) = answered else {
                    return Err("A routes call answers routes.".to_owned());
                };
                Ok(Answer::Waited(ResolverRoutesResultStepsItemAnswerWaited {
                    timers,
                    routes: waited_of(routes),
                }))
            }
            ResolverRoutesInputStepsItem::Route(each) => {
                let thrown = self.resolver.route(&each.entry).err().map(|error| error.0);
                Ok(Answer::Routed(ResolverRoutesResultStepsItemAnswerRouted {
                    thrown,
                }))
            }
            ResolverRoutesInputStepsItem::Dispose(_) => {
                self.resolver.dispose();
                Ok(changed())
            }
            ResolverRoutesInputStepsItem::During(each) => {
                let changes = each
                    .changes
                    .iter()
                    .map(step_of)
                    .collect::<Outcome<Vec<_>>>()?;
                let call = step_of(&each.call)?;
                let (reached, mut arrived) = oneshot::channel();
                let (release, held_back) = oneshot::channel();
                *locked(&self.gate.gate) = Some(Gate {
                    reached,
                    release: held_back,
                });
                let mut running = Box::pin(self.called(held, &call));
                // The call runs until it holds at its first key read, or ends.
                let first = std::future::poll_fn(|context| {
                    if let Poll::Ready(done) = running.as_mut().poll(context) {
                        return Poll::Ready(Some(done));
                    }
                    if Pin::new(&mut arrived).poll(context).is_ready() {
                        return Poll::Ready(None);
                    }
                    Poll::Pending
                })
                .await;
                for change in &changes {
                    self.change(change)?;
                }
                *locked(&self.gate.gate) = None;
                let _ = release.send(());
                match first {
                    Some(done) => done,
                    None => running.await,
                }
            }
            other => self.called(held, other).await,
        }
    }
}

pub fn routes(input: ResolverRoutesInput) -> Outcome<ResolverRoutesResult> {
    let wheel = Arc::new(TimerWheel::new(input.start_ms.unwrap_or(0)));
    let scheduler = Arc::new(FakeScheduler::new(Arc::clone(&wheel)));
    let scripted = Arc::new(ScriptedEntryKeys::new());
    for (id, answer) in input.answers.iter() {
        scripted.set(id, answer_of(answer)?);
    }
    let list = Arc::new(ScriptedProviderList::new(input.entries.clone()));
    let registry = Arc::new(ScriptedModelRegistry::empty());
    registry.set_rows(rows_for(&input.rows)?);
    let mut listings: HashMap<String, HashMap<String, Listing>> = HashMap::new();
    for (entry, rows) in input.listings.iter() {
        let mut held = HashMap::new();
        for (key, listing) in rows.iter() {
            held.insert(
                key.to_owned(),
                Listing {
                    status: listed_of(&listing.status)?,
                    when_zdr_off: match &listing.when_zdr_off {
                        None => None,
                        Some(off) => Some(listed_of(off)?),
                    },
                },
            );
        }
        listings.insert(entry.to_owned(), held);
    }
    let hub = Arc::new(Hub {
        cold: Mutex::new(input.cold.clone().unwrap_or_default().into_iter().collect()),
        listings,
        dispose_fails: input.dispose_fails.clone().unwrap_or_default(),
        slow: input
            .slow
            .iter()
            .flat_map(|slow| slow.iter().map(|(entry, ms)| (entry.to_owned(), *ms)))
            .collect(),
        scheduler: Some(Arc::clone(&scheduler)),
        ..Hub::default()
    });
    let gate = Arc::new(Keys {
        inner: Arc::clone(&scripted),
        failing: input.failing.clone(),
        gate: Mutex::new(None),
        hub: Arc::clone(&hub),
        slow: input
            .key_slow
            .iter()
            .flat_map(|slow| slow.iter().map(|(entry, ms)| (entry.to_owned(), *ms)))
            .collect(),
    });
    let zdr = Arc::new(AtomicBool::new(input.zdr_only));
    let refreshed = Arc::new(Mutex::new(Vec::<Via>::new()));
    let host = Arc::new(Host::default());
    let mut kinds = Vec::new();
    for name in ProviderKindName::ALL {
        if input.kinds.contains(&name) {
            kinds.push(stub_of(name, &hub)?);
        }
    }
    let told = Arc::clone(&refreshed);
    let live = Arc::clone(&zdr);
    let deps_host = Arc::clone(&host);
    let resolver = ModelResolver::new(ResolverDeps {
        list: list.clone(),
        keys: gate.clone(),
        registry: registry.clone() as Arc<dyn ModelRegistry>,
        kinds,
        kind_deps: Arc::new(move |_| deps_of(&deps_host, Arc::clone(&scheduler))),
        zdr_only: Arc::new(move || live.load(Ordering::SeqCst)),
        refreshed: Some(Arc::new(move |entry| {
            locked(&told).push(nyaaai_agentic::provider::resolver::via_of(entry));
        })),
    });
    let rig = Rig {
        list: Arc::clone(&list),
        keys: Arc::clone(&scripted),
        registry,
        zdr,
        host,
        hub: Arc::clone(&hub),
        resolver,
        refreshed: Arc::clone(&refreshed),
        gate,
        wheel: Arc::clone(&wheel),
    };
    let mut held: Vec<Vec<Resolved>> = Vec::new();
    let mut steps = Vec::new();
    let mut transports: Vec<String> = Vec::new();
    block_on(async {
        for step in &input.steps {
            held.push(Vec::new());
            let answer = rig.said(&mut held, step).await?;
            // The routes a step made are reported in the list's order.
            let order = rig.list.entries();
            let position = |id: &str| {
                order
                    .iter()
                    .position(|each| each.id == id)
                    .unwrap_or(usize::MAX)
            };
            let mut made = std::mem::take(&mut *locked(&hub.made));
            made.sort_by_key(|(id, _)| position(id));
            transports.extend(made.into_iter().map(|(_, label)| label));
            steps.push(ResolverRoutesResultStepsItem {
                answer,
                built: transports.len() as i64,
                disposed: locked(&hub.stopped).len() as i64,
            });
        }
        Ok::<(), String>(())
    })??;
    rig.resolver.dispose();
    let refreshed = locked(&rig.refreshed).clone();
    Ok(ResolverRoutesResult {
        steps,
        transports,
        stopped: locked(&hub.stopped).clone(),
        refreshed,
        key_reads: scripted.reads() as i64,
    })
}

/// The real OpenRouter kind with a catalog noting each use asked.
struct Asked {
    real: Arc<dyn ProviderKind>,
    asked: Arc<Mutex<Vec<ModelUse>>>,
}

struct Asking {
    rows: Arc<dyn ModelRegistry>,
    asked: Arc<Mutex<Vec<ModelUse>>>,
}

impl EntryCatalog for Asking {
    fn views(
        &self,
        use_: ModelUse,
        _zdr_only: bool,
        _cancel: Option<&CancelToken>,
    ) -> BoxFuture<'_, Result<Vec<ModelView>, Aborted>> {
        {
            let mut asked = locked(&self.asked);
            if !asked.contains(&use_) {
                asked.push(use_);
            }
        }
        Box::pin(async move {
            Ok(self
                .rows
                .rows()
                .iter()
                .map(|row| {
                    let mut view = unknown_view(row);
                    view.status = ModelViewStatus::Ok;
                    view
                })
                .collect())
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn arc_any(self: Arc<Self>) -> Arc<dyn std::any::Any + Send + Sync> {
        self
    }
}

impl ProviderKind for Asked {
    fn kind(&self) -> ProviderKindName {
        self.real.kind()
    }

    fn label(&self) -> &str {
        self.real.label()
    }

    fn field(&self) -> KindField {
        self.real.field()
    }

    fn check<'a>(
        &'a self,
        entry: &'a EntryFields,
        secret: Option<&'a str>,
        deps: &'a KindDeps,
        cancel: &'a CancelToken,
    ) -> BoxFuture<'a, Result<nyaaai_contract::shapes::provider::KeyState, KindError>> {
        self.real.check(entry, secret, deps, cancel)
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
        _entry: &ProviderEntry,
        _transport: Arc<Transport>,
        deps: CatalogDeps,
    ) -> Arc<dyn EntryCatalog> {
        Arc::new(Asking {
            rows: deps.rows,
            asked: Arc::clone(&self.asked),
        })
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
}

pub fn uses(input: ResolverUsesInput) -> Outcome<ResolverUsesResult> {
    type Call = ResolverUsesResultCallsItemCall;
    let asked = Arc::new(Mutex::new(Vec::<ModelUse>::new()));
    let kind: Arc<dyn ProviderKind> = Arc::new(Asked {
        real: real_of(ProviderKindName::Openrouter)?,
        asked: Arc::clone(&asked),
    });
    let host = Arc::new(Host::default());
    let scheduler = Arc::new(FakeScheduler::new(Arc::new(TimerWheel::new(0))));
    let entry = ProviderEntry {
        id: "a".to_owned(),
        kind: ProviderKindName::Openrouter,
        label: "First".to_owned(),
        plan: None,
        url: None,
        extra_usage: None,
    };
    let keys = Arc::new(ScriptedEntryKeys::new());
    keys.set("a", KeyAnswer::Key(KEY.to_owned()));
    let registry = Arc::new(ScriptedModelRegistry::empty());
    registry.set_rows(rows_for(&input.rows)?);
    let resolver = ModelResolver::new(ResolverDeps {
        list: Arc::new(ScriptedProviderList::new(vec![entry])),
        keys,
        registry: registry as Arc<dyn ModelRegistry>,
        kinds: vec![kind],
        kind_deps: Arc::new(move |_| deps_of(&host, Arc::clone(&scheduler))),
        zdr_only: Arc::new(|| true),
        refreshed: None,
    });
    let model = input.rows.first().cloned().unwrap_or_default();
    let mut calls = Vec::new();
    block_on(async {
        for use_ in &input.uses {
            for call in [Call::Routes, Call::Resolve, Call::Rows, Call::Candidates] {
                locked(&asked).clear();
                match call {
                    Call::Routes => {
                        resolver.routes(&model, use_).await;
                    }
                    Call::Resolve => {
                        resolver.resolve(&model, use_).await;
                    }
                    Call::Rows => {
                        resolver.rows(use_).await;
                    }
                    Call::Candidates => {
                        resolver.candidates(use_).await;
                    }
                }
                calls.push(ResolverUsesResultCallsItem {
                    call,
                    use_: use_.clone(),
                    asked: locked(&asked).clone(),
                });
            }
        }
    })?;
    resolver.dispose();
    Ok(ResolverUsesResult { calls })
}
