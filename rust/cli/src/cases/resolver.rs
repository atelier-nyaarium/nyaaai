//! Resolver cases.
//!
//! The resolver runs over stub kinds: a stub kind builds a real `Transport` over a fetch that
//! answers with what it saw, a codec that carries it back, and a catalog that lists what the
//! script says. The real kinds are cased by `kind.*`.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use nyaaai_agentic::codec::{Codec, Mode, StreamDecoder};
use nyaaai_agentic::dialects::openrouter::OPENROUTER_CHAT;
use nyaaai_agentic::failure::ProviderError;
use nyaaai_agentic::kinds::fetcher::Aborted;
use nyaaai_agentic::models::view::unknown_view;
use nyaaai_agentic::provider::kind::{
    Browser, CatalogDeps, EntryCatalog, KindDeps, KindError, KindField, ProviderKind, Route, Tester,
};
use nyaaai_agentic::provider::resolver::{
    Catalog, ModelResolver, Planned, Resolution, Resolved, ResolverDeps, ServedByDeps,
    key_source_of, reach_of, served_by as plan_served,
};
use nyaaai_agentic::step::reply_text;
use nyaaai_agentic::transport::{Auth, KeySource, Transport, TransportDeps};
use nyaaai_contract::cancel::{CancelToken, Unsubscribe};
use nyaaai_contract::entry_keys::{EntryKeys, KeyAnswer};
use nyaaai_contract::fetch::{Chunks, Fetch, FetchError, FetchRequest, FetchResponse};
use nyaaai_contract::future::BoxFuture;
use nyaaai_contract::json::{Object, Value, parse_json};
use nyaaai_contract::model_registry::ModelRegistry;
use nyaaai_contract::shape::Shape;
use nyaaai_contract::shapes::model::{ModelRow, ModelUse, ModelView, ModelViewStatus};
use nyaaai_contract::shapes::provider::{EntryFields, ProviderEntry, ProviderKindName};
use nyaaai_contract::shapes::step::{CanonicalRequest, Failure, HeaderList, StepResult};
use nyaaai_testkit::cases::{Outcome, block_on};
use nyaaai_testkit::scheduler::{FakeScheduler, TimerWheel};
use nyaaai_testkit::scripted_ports::{
    ScriptedEntryKeys, ScriptedModelRegistry, ScriptedProviderList,
};

use super::catalog::{convert, row_of_key};
use super::local::local_row_of;
use super::shapes::*;

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

/// The name a transport is told by.
fn label_of(label: &str, plan: Option<&str>, url: Option<&str>) -> String {
    format!(
        "{label}{}{}",
        plan.map_or(String::new(), |plan| format!("/{plan}")),
        url.map_or(String::new(), |url| format!(" {url}")),
    )
}

fn entry_label(entry: &ProviderEntry) -> String {
    label_of(
        &entry.label,
        entry.plan.map(|plan| plan.as_str()),
        entry.url.as_deref(),
    )
}

fn request(model: &str) -> Outcome<CanonicalRequest> {
    let text = format!(
        "{{\"model\":{},\"system\":[],\"history\":[],\"cache\":{{\"kind\":\"none\"}},\"session\":\"session-1\",\"zdrOnly\":true}}",
        nyaaai_contract::json::quote_json(model)
    );
    let value = parse_json(&text).ok_or_else(|| "the request does not parse".to_owned())?;
    CanonicalRequest::decode(&value).map_err(|refused| refused.to_string())
}

////////////////////////////////
//  The stubs

/// What the stub transport saw, carried back through the codec.
struct Echo {
    label: String,
}

struct Body(Option<Vec<u8>>);

impl Chunks for Body {
    fn next(&mut self) -> BoxFuture<'_, Result<Option<Vec<u8>>, FetchError>> {
        let held = self.0.take();
        Box::pin(async move { Ok(held) })
    }
}

impl Fetch for Echo {
    fn fetch(
        &self,
        request: FetchRequest,
        _cancel: CancelToken,
    ) -> BoxFuture<'_, Result<FetchResponse, FetchError>> {
        let key = request
            .headers
            .iter()
            .find(|(name, _)| name == "Authorization")
            .and_then(|(_, value)| value.strip_prefix("Bearer "))
            .map(str::to_owned);
        let mut seen = Object::new();
        seen.insert("transport", Value::from(self.label.as_str()));
        seen.insert(
            "key",
            key.map_or(Value::Null, |key| Value::from(key.as_str())),
        );
        let text = nyaaai_contract::json::stringify_json(&Value::Object(seen));
        Box::pin(async move {
            Ok(FetchResponse {
                status: 200,
                headers: vec![("content-type".to_owned(), "application/json".to_owned())],
                chunks: Box::new(Body(Some(text.into_bytes()))),
            })
        })
    }
}

/// The real kind's codec, carrying what the transport saw back as the reply.
struct Carrying {
    inner: Box<dyn Codec>,
    sent: Mutex<(bool, String)>,
}

impl Codec for Carrying {
    fn encode(
        &self,
        request: &CanonicalRequest,
        mode: Mode,
    ) -> Result<nyaaai_contract::shapes::step::Encoded, String> {
        let encoded = self.inner.encode(request, mode)?;
        *self.sent.lock().unwrap_or_else(|held| held.into_inner()) =
            (request.zdr_only, encoded.path.clone());
        Ok(encoded)
    }

    fn stream(&self) -> Box<dyn StreamDecoder + '_> {
        self.inner.stream()
    }

    fn complete(&self, body: &Value) -> Result<StepResult, ProviderError> {
        let (zdr_only, path) = self
            .sent
            .lock()
            .unwrap_or_else(|held| held.into_inner())
            .clone();
        let mut seen = Object::new();
        for name in ["transport", "key"] {
            seen.insert(
                name,
                body.as_object()
                    .and_then(|fields| fields.get(name))
                    .cloned()
                    .unwrap_or(Value::Null),
            );
        }
        seen.insert("zdrOnly", Value::Bool(zdr_only));
        seen.insert("path", Value::from(path.as_str()));
        let text = nyaaai_contract::json::stringify_json(&Value::Object(seen));
        let mut part = Object::new();
        part.insert("kind", Value::from("text"));
        part.insert("text", Value::from(text.as_str()));
        let mut result = Object::new();
        result.insert("parts", Value::Array(vec![Value::Object(part)]));
        result.insert("finish", Value::from("stop"));
        result.insert("serving", Value::Object(Object::new()));
        StepResult::decode(&Value::Object(result)).map_err(|refused| {
            nyaaai_agentic::failure::failed(
                nyaaai_contract::shapes::step::FailureCategory::Stream,
                &refused.to_string(),
                nyaaai_agentic::failure::FailureExtra::default(),
            )
        })
    }

    fn failure(&self, status: i64, body: &Value, headers: &HeaderList) -> Failure {
        self.inner.failure(status, body, headers)
    }
}

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
    transports: Mutex<Vec<String>>,
    stopped: Mutex<Vec<String>>,
    refreshes: Mutex<HashMap<String, Arc<dyn Fn() + Send + Sync>>>,
    listings: HashMap<String, HashMap<String, Listing>>,
}

fn locked<T>(held: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    held.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
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
                match status {
                    Listed::Unlisted => {}
                    Listed::Status(status) => {
                        let mut view = unknown_view(&row);
                        view.status = status;
                        views.push(view);
                    }
                }
            }
            Ok(views)
        })
    }

    fn dispose(&self) {
        locked(&self.hub.stopped).push(entry_label(&self.entry));
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn arc_any(self: Arc<Self>) -> Arc<dyn std::any::Any + Send + Sync> {
        self
    }
}

/// A real kind's codec and field, with a stub transport and catalog.
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
        let label = label_of(
            &entry.label,
            entry.plan.map(|plan| plan.as_str()),
            entry.url.as_deref(),
        );
        locked(&self.hub.transports).push(label.clone());
        Ok(Arc::new(Transport::new(TransportDeps {
            fetch: Arc::new(Echo {
                label: label.clone(),
            }),
            scheduler: Arc::clone(&deps.scheduler),
            key,
            log: Box::new(|_| {}),
            base_url: "https://stub.test".to_owned(),
            label,
            user_agent: "cases/1.0".to_owned(),
            errors: Arc::new(OPENROUTER_CHAT),
            auth: Auth::Optional,
            idle_ms: None,
            max_bytes: None,
        })))
    }

    fn catalog(
        &self,
        entry: &ProviderEntry,
        _transport: Arc<Transport>,
        deps: CatalogDeps,
    ) -> Arc<dyn EntryCatalog> {
        locked(&self.hub.refreshes).insert(entry.id.clone(), Arc::clone(&deps.refreshed));
        Arc::new(StubCatalog {
            name: self.real.kind(),
            entry: entry.clone(),
            rows: deps.rows,
            hub: Arc::clone(&self.hub),
        })
    }

    fn browse(&self, _entry: &ProviderEntry, _route: &Route) -> Option<Box<dyn Browser>> {
        None
    }

    fn tester(&self, _entry: &ProviderEntry, _route: &Route) -> Option<Box<dyn Tester>> {
        None
    }

    fn codec(&self, view: Option<&ModelView>, entry: &EntryFields) -> Box<dyn Codec> {
        Box::new(Carrying {
            inner: self.real.codec(view, entry),
            sent: Mutex::new((false, String::new())),
        })
    }

    fn failure(
        &self,
        failure: &Failure,
        view: Option<&ModelView>,
    ) -> nyaaai_contract::shapes::provider::Disposition {
        self.real.failure(failure, view)
    }
}

fn stub_of(name: ProviderKindName, hub: &Arc<Hub>) -> Outcome<Arc<dyn ProviderKind>> {
    let real = nyaaai_agentic::provider::kind::KINDS
        .iter()
        .find(|held| held.kind() == name)
        .ok_or_else(|| format!("No {} kind.", name.as_str()))?;
    Ok(Arc::new(Stub {
        real: Arc::clone(real),
        hub: Arc::clone(hub),
    }))
}

fn deps_of(scheduler: Arc<FakeScheduler>) -> KindDeps {
    KindDeps {
        fetch: Arc::new(Echo {
            label: String::new(),
        }),
        scheduler,
        log: Arc::new(|_| {}),
        user_agent: "cases/1.0".to_owned(),
        base_url: None,
    }
}

fn scheduler_of() -> Arc<FakeScheduler> {
    Arc::new(FakeScheduler::new(Arc::new(TimerWheel::new(0))))
}

/// Keys that fail to read answer none, without reading the scripted ones.
struct Keys {
    inner: Arc<ScriptedEntryKeys>,
    failing: Vec<String>,
}

impl EntryKeys for Keys {
    fn get(&self, id: &str) -> BoxFuture<'_, KeyAnswer> {
        if self.failing.iter().any(|held| held == id) {
            return Box::pin(async { KeyAnswer::None });
        }
        self.inner.get(id)
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

fn sent_of(result: &StepResult) -> Outcome<Object> {
    match parse_json(&reply_text(result)) {
        Some(Value::Object(fields)) => Ok(fields),
        _ => Err("the stub transport did not answer".to_owned()),
    }
}

pub fn served_by(input: ResolverServedByInput) -> Outcome<ResolverServedByResult> {
    let hub = Arc::new(Hub::default());
    let kind = stub_of(input.kind, &hub)?;
    let scheduler = scheduler_of();
    let zdr = Arc::new(AtomicBool::new(input.zdr_only));
    let fields = EntryFields {
        kind: input.entry.kind,
        label: input.entry.label.clone(),
        plan: input.entry.plan,
        url: input.entry.url.clone(),
        extra_usage: input.entry.extra_usage,
    };
    let transport = kind.transport(&fields, Arc::new(NoKey), &deps_of(scheduler))?;
    let live = Arc::clone(&zdr);
    let status = match &input.view {
        None => None,
        Some(held) => Some(convert::<ResolverServedByInputView, ModelViewStatus>(held)?),
    };
    let deps = ServedByDeps {
        entry: input.entry.clone(),
        kind,
        transport,
        catalog: Arc::new(Standing { status }),
        zdr_only: Arc::new(move || live.load(Ordering::SeqCst)),
    };
    let routes = block_on(plan_served(&deps, &input.key, input.use_))?;
    let first = routes
        .candidates
        .first()
        .ok_or_else(|| "A plan serves its entry.".to_owned())?;
    let zdr_on_resolve = first.zdr_only();
    zdr.store(input.flipped_to, Ordering::SeqCst);
    let sent = sent_of(
        &block_on(first.transport.complete(
            &*first.codec,
            &request(&input.key)?,
            &CancelToken::new(),
        ))?
        .map_err(|error| error.to_string())?,
    )?;
    let path = first.codec.encode(&request("m")?, Mode::Complete)?.path;
    Ok(ResolverServedByResult {
        key: routes.key.clone(),
        entry: first.entry.id.clone(),
        binding: first.binding.id.clone(),
        view: first.view.clone(),
        path,
        passed: routes.passed.clone(),
        candidates: routes.candidates.len() as i64,
        zdr_on_resolve,
        sent: ResolverServedByResultSent {
            zdr_only: sent.get("zdrOnly") == Some(&Value::Bool(true)),
            path: sent
                .get("path")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned(),
        },
    })
}

struct NoKey;

impl KeySource for NoKey {
    fn get(&self) -> BoxFuture<'_, Option<String>> {
        Box::pin(async { None })
    }
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

pub fn routes(input: ResolverRoutesInput) -> Outcome<ResolverRoutesResult> {
    let scheduler = scheduler_of();
    let scripted = Arc::new(ScriptedEntryKeys::new());
    for (id, answer) in input.answers.iter() {
        scripted.set(id, answer_of(answer)?);
    }
    let keys = Arc::new(Keys {
        inner: Arc::clone(&scripted),
        failing: input.failing.clone(),
    });
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
        listings,
        ..Hub::default()
    });
    let zdr = Arc::new(AtomicBool::new(input.zdr_only));
    let refreshed = Arc::new(Mutex::new(Vec::<String>::new()));
    let mut kinds = Vec::new();
    for name in ProviderKindName::ALL {
        if input.kinds.contains(&name) {
            kinds.push(stub_of(name, &hub)?);
        }
    }
    let told = Arc::clone(&refreshed);
    let live = Arc::clone(&zdr);
    let resolver = ModelResolver::new(ResolverDeps {
        list: list.clone(),
        keys,
        registry: registry.clone(),
        kinds,
        kind_deps: Arc::new({
            let scheduler = Arc::clone(&scheduler);
            move |_| deps_of(Arc::clone(&scheduler))
        }),
        zdr_only: Arc::new(move || live.load(Ordering::SeqCst)),
        refreshed: Some(Arc::new(move |entry| locked(&told).push(entry.id.clone()))),
    });
    let mut held: Vec<Vec<Resolved>> = Vec::new();
    let mut steps = Vec::new();
    for step in &input.steps {
        held.push(Vec::new());
        let answer = match step {
            ResolverRoutesInputStepsItem::Routes(each) => {
                match block_on(resolver.routes(&each.model, each.use_))?.map_err(|e| e.0)? {
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
                match block_on(resolver.resolve(&each.model, each.use_))?.map_err(|e| e.0)? {
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
                let options = block_on(resolver.rows(each.use_))?.map_err(|e| e.0)?;
                let mut lines = Vec::new();
                for option in options {
                    lines.push(ResolverRoutesResultStepsItemAnswerRowsOptionsItem {
                        key: option.key,
                        status: convert(&option.view.status)?,
                        via: option.via,
                    });
                }
                Answer::Rows(ResolverRoutesResultStepsItemAnswerRows { options: lines })
            }
            ResolverRoutesInputStepsItem::Candidates(each) => {
                let options = block_on(resolver.candidates(each.use_))?.map_err(|e| e.0)?;
                let mut served = Vec::new();
                for option in options {
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
            ResolverRoutesInputStepsItem::List(each) => {
                list.set(each.entries.clone());
                changed()
            }
            ResolverRoutesInputStepsItem::Answer(each) => {
                scripted.set(&each.id, answer_of(&each.answer)?);
                changed()
            }
            ResolverRoutesInputStepsItem::Registry(each) => {
                registry.set_rows(rows_for(&each.rows)?);
                changed()
            }
            ResolverRoutesInputStepsItem::Zdr(each) => {
                zdr.store(each.on, Ordering::SeqCst);
                changed()
            }
            ResolverRoutesInputStepsItem::Refreshed(each) => {
                let told = locked(&hub.refreshes).get(&each.entry).cloned();
                if let Some(told) = told {
                    told();
                }
                changed()
            }
            ResolverRoutesInputStepsItem::Send(each) => {
                match held
                    .get(each.from as usize)
                    .and_then(|from| from.get(each.candidate as usize))
                {
                    None => Answer::None(ResolverRoutesResultStepsItemAnswerNone {}),
                    Some(found) => {
                        let result = block_on(found.transport.complete(
                            &*found.codec,
                            &request(&found.key)?,
                            &CancelToken::new(),
                        ))?
                        .map_err(|error| error.to_string())?;
                        let sent = sent_of(&result)?;
                        Answer::Sent(ResolverRoutesResultStepsItemAnswerSent {
                            transport: sent
                                .get("transport")
                                .and_then(Value::as_str)
                                .unwrap_or_default()
                                .to_owned(),
                            zdr_only: sent.get("zdrOnly") == Some(&Value::Bool(true)),
                            key: sent.get("key").and_then(Value::as_str).map(str::to_owned),
                            path: sent
                                .get("path")
                                .and_then(Value::as_str)
                                .unwrap_or_default()
                                .to_owned(),
                        })
                    }
                }
            }
        };
        steps.push(ResolverRoutesResultStepsItem {
            answer,
            built: locked(&hub.transports).len() as i64,
            disposed: locked(&hub.stopped).len() as i64,
        });
    }
    resolver.dispose();
    Ok(ResolverRoutesResult {
        steps,
        transports: locked(&hub.transports).clone(),
        stopped: locked(&hub.stopped).clone(),
        refreshed: locked(&refreshed).clone(),
        key_reads: scripted.reads() as i64,
        list_reads: list.reads() as i64,
    })
}

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
