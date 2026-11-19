//! Local kind cases.

use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex};

use nyaaai_agentic::codec::Mode;
use nyaaai_agentic::kinds::compatible::server as compatible_server;
use nyaaai_agentic::kinds::compatible::{
    COMPATIBLE, listed as compatible_listed, model_of as compatible_model, remote, terms_of,
};
use nyaaai_agentic::kinds::llamacpp::{
    LLAMACPP, Props, listed as llamacpp_listed, model_of as llamacpp_model_of, props_of_body,
    server as llamacpp_server,
};
use nyaaai_agentic::kinds::lmstudio::{
    LMSTUDIO, chat_models, loaded_contexts, model_of as lmstudio_model, server as lmstudio_server,
};
use nyaaai_agentic::kinds::local::catalog::{
    Discovered, Loaded, LocalCatalog, LocalCatalogDeps, LocalModel, LocalServer, Think, context_of,
    efforts_of, label_of, local_view,
};
use nyaaai_agentic::kinds::local::kind::{LocalKind, Terms, TermsPrice};
use nyaaai_agentic::kinds::ollama::{
    OLLAMA, local_tags, model_of as ollama_model, num_ctx_of, running_of, server as ollama_server,
};
use nyaaai_agentic::models::curated::MODELS;
use nyaaai_agentic::models::registry::row_of;
use nyaaai_agentic::models::row_key::parse_row_key_text;
use nyaaai_agentic::models::view::{codec_facts_of, unknown_view};
use nyaaai_agentic::provider::kind::{CatalogDeps, KindDeps, KindError, ProviderKind, Route};
use nyaaai_agentic::step::reply_text;
use nyaaai_agentic::transport::{KeySource, Transport, TransportError};
use nyaaai_contract::cancel::CancelToken;
use nyaaai_contract::fetch::Method;
use nyaaai_contract::future::BoxFuture;
use nyaaai_contract::json::{Value, parse_json};
use nyaaai_contract::model_registry::ModelRegistry;
use nyaaai_contract::shape::Shape;
use nyaaai_contract::shapes::model::{
    ModelRow, ModelUse, ModelView, ModelViewPrice, ModelViewStatus, RowIdentity, TestRecord,
    TestRecordCache, TestRecordChecks, TestRecordChecksReasoning, TestRecordChecksRecovery,
    TestRecordChecksSchema, TestRecordChecksToolPlus, TestRecordVerifiedPrice,
};
use nyaaai_contract::shapes::provider::{EntryFields, KeyState, ProviderEntry, ProviderKindName};
use nyaaai_contract::shapes::step::CanonicalRequest;
use nyaaai_contract::time::Clock;
use nyaaai_testkit::cases::{Outcome, block_on};
use nyaaai_testkit::keys::FakeKeys;
use nyaaai_testkit::scheduler::{FakeScheduler, TimerWheel};
use nyaaai_testkit::scripted_fetch::ScriptedFetch;
use nyaaai_testkit::scripted_ports::ScriptedModelRegistry;

use super::catalog::{CALLER_REASON, Told, caller_token, exchange_of, view_line, view_lines};
use super::shapes::*;
use nyaaai_agentic::kinds::join::join_all;

const USER_AGENT: &str = "cases/1.0";

////////////////////////////////
//  Shared pieces

#[derive(Clone, Copy)]
enum Which {
    Ollama,
    Lmstudio,
    Llamacpp,
    Compatible,
}

macro_rules! which_of {
    ($($shape:ident),* $(,)?) => {$(
        impl From<&$shape> for Which {
            fn from(held: &$shape) -> Self {
                match held {
                    $shape::Ollama => Self::Ollama,
                    $shape::Lmstudio => Self::Lmstudio,
                    $shape::Llamacpp => Self::Llamacpp,
                    $shape::Compatible => Self::Compatible,
                }
            }
        }
    )*};
}

which_of!(
    LocalReachInputKind,
    LocalServerInputKind,
    LocalFlowInputKind,
    LocalKindFactsInputKind,
    LocalKindCodecInputKind,
    LocalKindDispositionInputKind,
);

impl Which {
    fn name(self) -> ProviderKindName {
        match self {
            Self::Ollama => ProviderKindName::Ollama,
            Self::Lmstudio => ProviderKindName::Lmstudio,
            Self::Llamacpp => ProviderKindName::Llamacpp,
            Self::Compatible => ProviderKindName::Compatible,
        }
    }

    fn kind(self) -> &'static LocalKind {
        match self {
            Self::Ollama => &OLLAMA,
            Self::Lmstudio => &LMSTUDIO,
            Self::Llamacpp => &LLAMACPP,
            Self::Compatible => &COMPATIBLE,
        }
    }

    fn server(self, transport: Arc<Transport>) -> Arc<dyn LocalServer> {
        match self {
            Self::Ollama => ollama_server(transport),
            Self::Lmstudio => lmstudio_server(transport),
            Self::Llamacpp => llamacpp_server(transport),
            Self::Compatible => compatible_server(transport),
        }
    }
}

pub(super) fn message_of(error: &KindError) -> String {
    match error {
        KindError::Cancelled => CALLER_REASON.to_owned(),
        KindError::TimedOut => "The operation timed out.".to_owned(),
        KindError::Provider(held) => held.failure.message.clone(),
        KindError::Message(text) => text.clone(),
    }
}

fn transport_message(error: &TransportError) -> String {
    match error {
        TransportError::Cancelled => CALLER_REASON.to_owned(),
        TransportError::Provider(held) => held.failure.message.clone(),
    }
}

fn category_of(error: &KindError) -> Option<String> {
    match error {
        KindError::Provider(held) => Some(held.failure.category.as_str().to_owned()),
        _ => None,
    }
}

fn served_of(model: &LocalModel) -> ServedModel {
    ServedModel {
        tag: model.tag.clone(),
        tools: model.tools,
        thinking: model.thinking,
        efforts: model.efforts.clone(),
        context: model.context,
        loaded: model.loaded,
        trained: model.trained,
        probed: model.unprobed.then_some(()),
    }
}

fn local_of(model: &ServedModel) -> LocalModel {
    LocalModel {
        tag: model.tag.clone(),
        tools: model.tools,
        thinking: model.thinking,
        efforts: model.efforts.clone(),
        context: model.context,
        loaded: model.loaded,
        trained: model.trained,
        unprobed: model.probed.is_some(),
    }
}

fn terms_from(held: &ServedTerms) -> Terms {
    Terms {
        kept: held.kept,
        price: match held.price {
            ServedTermsPrice::Free => TermsPrice::Free,
            ServedTermsPrice::Unknown => TermsPrice::Unknown,
        },
    }
}

fn price_term(held: TermsPrice) -> LocalCompatibleTermsResultTermsItemPrice {
    match held {
        TermsPrice::Free => LocalCompatibleTermsResultTermsItemPrice::Free,
        TermsPrice::Unknown => LocalCompatibleTermsResultTermsItemPrice::Unknown,
    }
}

fn contexts_of(contexts: &Loaded) -> Vec<(String, f64)> {
    contexts.clone()
}

/// A passing record every row of these cases holds.
fn passing(label: &str) -> TestRecord {
    TestRecord {
        label: label.to_owned(),
        date: "2026-01-01".to_owned(),
        cost: 0.0,
        estimated: None,
        checks: TestRecordChecks {
            tool_plus: TestRecordChecksToolPlus::Pass,
            recovery: TestRecordChecksRecovery::Pass,
            reasoning: TestRecordChecksReasoning::Pass,
            schema: TestRecordChecksSchema::Pass,
        },
        reasoning: true,
        cache: TestRecordCache::None,
        verified_price: TestRecordVerifiedPrice {
            prompt: 0.0,
            completion: 0.0,
        },
    }
}

/// A local row by its key, `local:<entry>/<tag>`.
pub(super) fn local_row_of(key: &str) -> Outcome<ModelRow> {
    let identity = parse_row_key_text(key);
    let Some(RowIdentity::Local(local)) = &identity else {
        return Err(format!("No local row {key}."));
    };
    identity
        .as_ref()
        .and_then(|held| row_of(held, &passing(&local.tag), ProviderKindName::Ollama))
        .ok_or_else(|| format!("No local row {key}."))
}

fn local_rows_of(keys: &[String]) -> Outcome<Vec<ModelRow>> {
    keys.iter().map(|key| local_row_of(key)).collect()
}

fn entry_of(which: Which, label: &str, url: &Option<String>) -> EntryFields {
    EntryFields {
        kind: which.name(),
        label: label.to_owned(),
        plan: None,
        url: url.clone(),
        extra_usage: None,
    }
}

struct Keys(Arc<FakeKeys>);

impl KeySource for Keys {
    fn get(&self) -> BoxFuture<'_, Option<String>> {
        self.0.get()
    }
}

/// A kind's deps over a scripted network on a wheel.
pub(super) struct Net {
    pub wheel: Arc<TimerWheel>,
    pub played: Arc<ScriptedFetch>,
    pub scheduler: Arc<FakeScheduler>,
    pub deps: KindDeps,
}

pub(super) fn net_of(start_ms: i64, exchanges: &[NetExchange]) -> Net {
    let wheel = Arc::new(TimerWheel::new(start_ms));
    let played = Arc::new(ScriptedFetch::new(
        exchanges.iter().map(exchange_of).collect(),
        Arc::clone(&wheel),
    ));
    let scheduler = Arc::new(FakeScheduler::new(Arc::clone(&wheel)));
    let deps = KindDeps {
        fetch: played.clone(),
        scheduler: scheduler.clone(),
        log: Arc::new(|_| {}),
        user_agent: USER_AGENT.to_owned(),
        base_url: None,
    };
    Net {
        wheel,
        played,
        scheduler,
        deps,
    }
}

pub(super) fn keys_of(stored: &Option<String>) -> Outcome<Arc<dyn KeySource>> {
    let keys =
        FakeKeys::new(vec![stored.clone()]).ok_or_else(|| "fakeKeys needs a value".to_owned())?;
    Ok(Arc::new(Keys(Arc::new(keys))))
}

fn transport_of(
    which: Which,
    entry: &EntryFields,
    deps: &KindDeps,
    stored: &Option<String>,
) -> Outcome<Arc<Transport>> {
    which.kind().transport(entry, keys_of(stored)?, deps)
}

pub(super) fn requests_of(played: &ScriptedFetch) -> Vec<(String, String, Option<String>, Value)> {
    played
        .requests()
        .iter()
        .map(|sent| {
            (
                match sent.method {
                    Method::Get => "GET",
                    Method::Post => "POST",
                }
                .to_owned(),
                sent.url.clone(),
                sent.headers
                    .iter()
                    .find(|(name, _)| name == "Authorization")
                    .map(|(_, value)| value.clone()),
                sent.body
                    .as_deref()
                    .and_then(parse_json)
                    .unwrap_or(Value::Null),
            )
        })
        .collect()
}

macro_rules! request_lines {
    ($played:expr, $item:ident) => {
        requests_of(&$played)
            .into_iter()
            .map(|(method, url, authorization, body)| $item {
                method,
                url,
                authorization,
                body,
            })
            .collect()
    };
}

////////////////////////////////
//  Decoders

pub fn efforts(input: LocalEffortsInput) -> Outcome<LocalEffortsResult> {
    let values: Vec<Think> = input
        .values
        .iter()
        .map(|value| match value {
            LocalEffortsInputValuesItem::Flag(flag) => Think::Flag(*flag),
            LocalEffortsInputValuesItem::Text(text) => Think::Text(text.clone()),
        })
        .collect();
    Ok(LocalEffortsResult {
        efforts: efforts_of(&values),
    })
}

pub fn label(input: LocalLabelInput) -> Outcome<LocalLabelResult> {
    Ok(LocalLabelResult {
        labels: input.tags.iter().map(|tag| label_of(tag)).collect(),
    })
}

pub fn view(input: LocalViewInput) -> Outcome<LocalViewResult> {
    let row = local_row_of(&input.row)?;
    let model = local_of(&input.model);
    let terms = terms_from(&input.terms);
    Ok(LocalViewResult {
        view: view_line(&local_view(
            &row,
            &model,
            input.loaded,
            &terms,
            input.zdr_only,
        ))?,
        context: context_of(&model, input.loaded),
    })
}

pub fn ollama_tags(input: LocalOllamaTagsInput) -> Outcome<LocalOllamaTagsResult> {
    Ok(LocalOllamaTagsResult {
        tags: local_tags(&input.body),
    })
}

pub fn ollama_show(input: LocalOllamaShowInput) -> Outcome<LocalOllamaShowResult> {
    Ok(LocalOllamaShowResult {
        model: ollama_model(&input.tag, &input.body).map(|model| served_of(&model)),
    })
}

pub fn ollama_num_ctx(input: LocalOllamaNumCtxInput) -> Outcome<LocalOllamaNumCtxResult> {
    Ok(LocalOllamaNumCtxResult {
        contexts: input.texts.iter().map(|text| num_ctx_of(text)).collect(),
    })
}

pub fn ollama_running(input: LocalOllamaRunningInput) -> Outcome<LocalOllamaRunningResult> {
    Ok(LocalOllamaRunningResult {
        contexts: contexts_of(&running_of(&input.body))
            .into_iter()
            .map(|(tag, context)| LocalOllamaRunningResultContextsItem { tag, context })
            .collect(),
    })
}

pub fn lmstudio_models(input: LocalLmstudioModelsInput) -> Outcome<LocalLmstudioModelsResult> {
    Ok(LocalLmstudioModelsResult {
        models: chat_models(&input.body).map(|models| {
            models
                .iter()
                .map(|model| served_of(&lmstudio_model(model)))
                .collect()
        }),
    })
}

pub fn lmstudio_loaded(input: LocalLmstudioLoadedInput) -> Outcome<LocalLmstudioLoadedResult> {
    Ok(LocalLmstudioLoadedResult {
        contexts: loaded_contexts(&chat_models(&input.body).unwrap_or_default())
            .into_iter()
            .map(|(tag, context)| LocalLmstudioLoadedResultContextsItem { tag, context })
            .collect(),
    })
}

pub fn llamacpp_models(input: LocalLlamacppModelsInput) -> Outcome<LocalLlamacppModelsResult> {
    Ok(LocalLlamacppModelsResult {
        models: llamacpp_listed(&input.body).map(|models| {
            models
                .into_iter()
                .map(|model| LocalLlamacppModelsResultModelsItem {
                    id: model.id,
                    trained: model.trained,
                })
                .collect()
        }),
    })
}

pub fn llamacpp_props(input: LocalLlamacppPropsInput) -> Outcome<LocalLlamacppPropsResult> {
    Ok(LocalLlamacppPropsResult {
        props: props_of_body(&input.body).map(|props| LocalLlamacppPropsResultProps {
            context: props.context,
            efforts: props.efforts,
        }),
    })
}

pub fn llamacpp_model(input: LocalLlamacppModelInput) -> Outcome<LocalLlamacppModelResult> {
    let props = input.props.as_ref().map(|props| Props {
        context: props.context,
        efforts: props.efforts.clone(),
    });
    let known = input.known.as_ref().map(local_of);
    Ok(LocalLlamacppModelResult {
        model: served_of(&llamacpp_model_of(
            &input.id,
            input.trained,
            props.as_ref(),
            known.as_ref(),
        )),
    })
}

pub fn compatible_models(
    input: LocalCompatibleModelsInput,
) -> Outcome<LocalCompatibleModelsResult> {
    Ok(LocalCompatibleModelsResult {
        models: compatible_listed(&input.body).map(|models| {
            models
                .iter()
                .map(|model| served_of(&compatible_model(model)))
                .collect()
        }),
    })
}

pub fn compatible_terms(input: LocalCompatibleTermsInput) -> Outcome<LocalCompatibleTermsResult> {
    Ok(LocalCompatibleTermsResult {
        terms: input
            .urls
            .iter()
            .map(|url| {
                let entry = EntryFields {
                    kind: ProviderKindName::Compatible,
                    label: "Host".to_owned(),
                    plan: None,
                    url: url.clone(),
                    extra_usage: None,
                };
                let terms = terms_of(&entry);
                LocalCompatibleTermsResultTermsItem {
                    remote: remote(&entry),
                    kept: terms.kept,
                    price: price_term(terms.price),
                }
            })
            .collect(),
    })
}

////////////////////////////////
//  Reach and the server

/// The Settings line; none when the kind cannot read the entry's URL.
fn facts_of(kind: &dyn ProviderKind, state: &KeyState, entry: &EntryFields) -> Option<String> {
    kind.facts(state, entry, None)
}

pub fn reach(input: LocalReachInput) -> Outcome<LocalReachResult> {
    let which = Which::from(&input.kind);
    let net = net_of(input.start_ms, &input.exchanges);
    let named = which.kind();
    let entry = entry_of(which, &input.entry.label, &input.entry.url);
    let token = caller_token(
        &net.wheel,
        "caller",
        input.abort_after_ms,
        input.aborted_before,
    );
    let checked = block_on(named.check(&entry, input.secret.as_deref(), &net.deps, &token))?;
    net.wheel.cancel("caller");
    let (state, thrown) = match checked {
        Ok(state) => (Some(state), None),
        Err(error) => (None, Some(message_of(&error))),
    };
    Ok(LocalReachResult {
        facts: state
            .as_ref()
            .and_then(|state| facts_of(named, state, &entry)),
        state,
        thrown,
        requests: request_lines!(net.played, LocalReachResultRequestsItem),
        waiting: net.wheel.waiting(),
    })
}

pub fn server(input: LocalServerInput) -> Outcome<LocalServerResult> {
    let which = Which::from(&input.kind);
    let net = net_of(input.start_ms, &input.exchanges);
    let entry = entry_of(which, &input.entry.label, &input.entry.url);
    let transport = transport_of(which, &entry, &net.deps, &input.stored)?;
    let server = which.server(transport);
    let mut steps = Vec::new();
    let mut tags = 0;
    block_on(async {
        for step in &input.steps {
            tags += 1;
            let tag = format!("caller-{tags}");
            let (after, before) = match step {
                LocalServerInputStepsItem::Models(held) => {
                    (held.abort_after_ms, held.aborted_before)
                }
                LocalServerInputStepsItem::Loaded(held) => {
                    (held.abort_after_ms, held.aborted_before)
                }
                LocalServerInputStepsItem::Warm(held) => (held.abort_after_ms, held.aborted_before),
            };
            let token = caller_token(&net.wheel, &tag, after, before);
            let mut held = LocalServerResultStepsItem {
                models: None,
                contexts: None,
                done: false,
                thrown: None,
            };
            match step {
                LocalServerInputStepsItem::Models(each) => {
                    let known: Vec<LocalModel> = each.known.iter().map(local_of).collect();
                    match server.models(&known, &token).await {
                        Ok(models) => held.models = Some(models.iter().map(served_of).collect()),
                        Err(error) => held.thrown = Some(message_of(&error)),
                    }
                }
                LocalServerInputStepsItem::Loaded(each) => {
                    if !server.reads_loaded() {
                        held.thrown = Some("This server reads no loaded contexts.".to_owned());
                    } else {
                        match server.loaded(&each.tag, &token).await {
                            Ok(contexts) => {
                                held.contexts = Some(
                                    contexts_of(&contexts)
                                        .into_iter()
                                        .map(|(tag, context)| {
                                            LocalServerResultStepsItemContextsItem { tag, context }
                                        })
                                        .collect(),
                                );
                            }
                            Err(error) => held.thrown = Some(message_of(&error)),
                        }
                    }
                }
                LocalServerInputStepsItem::Warm(each) => {
                    if !server.warms() {
                        held.thrown = Some("This server loads no model.".to_owned());
                    } else {
                        match server.warm(&each.tag, &token).await {
                            Ok(()) => held.done = true,
                            Err(error) => held.thrown = Some(message_of(&error)),
                        }
                    }
                }
            }
            steps.push(held);
        }
    })?;
    for tag in 1..=tags {
        net.wheel.cancel(&format!("caller-{tag}"));
    }
    Ok(LocalServerResult {
        steps,
        requests: request_lines!(net.played, LocalServerResultRequestsItem),
        read: net
            .played
            .read()
            .into_iter()
            .map(|read| read as i64)
            .collect(),
        waiting: net.wheel.waiting(),
    })
}

////////////////////////////////
//  The catalog over a scripted server

struct ScriptedReply {
    models: Vec<LocalModel>,
    contexts: Loaded,
    delay_ms: Option<i64>,
    fails: bool,
}

struct ScriptedCall {
    method: LocalCatalogRunResultStepsItemServedItemMethod,
    tag: Option<String>,
    known: Vec<String>,
    start_ms: i64,
    end_ms: i64,
    aborted: bool,
}

#[derive(Clone, Copy)]
enum Ask {
    Models,
    Loaded,
    Warm,
}

/// A server that answers from a script, spending the wheel's time.
struct ScriptedServer {
    wheel: Arc<TimerWheel>,
    live: bool,
    loaded: bool,
    warm: bool,
    models: Vec<ScriptedReply>,
    loads: Vec<ScriptedReply>,
    warms: Vec<ScriptedReply>,
    taken: Mutex<[usize; 3]>,
    calls: Mutex<Vec<ScriptedCall>>,
}

impl ScriptedServer {
    async fn answer(
        &self,
        method: Ask,
        tag: Option<&str>,
        known: Vec<String>,
        cancel: &CancelToken,
    ) -> Result<(Vec<LocalModel>, Loaded), KindError> {
        let at = method as usize;
        let reply = {
            let mut taken = self.taken.lock().unwrap_or_else(|held| held.into_inner());
            let reply = match method {
                Ask::Models => self.models.get(taken[at]),
                Ask::Loaded => self.loads.get(taken[at]),
                Ask::Warm => self.warms.get(taken[at]),
            };
            taken[at] += 1;
            reply
        };
        let index = {
            let mut calls = self.calls.lock().unwrap_or_else(|held| held.into_inner());
            calls.push(ScriptedCall {
                method: match method {
                    Ask::Models => LocalCatalogRunResultStepsItemServedItemMethod::Models,
                    Ask::Loaded => LocalCatalogRunResultStepsItemServedItemMethod::Loaded,
                    Ask::Warm => LocalCatalogRunResultStepsItemServedItemMethod::Warm,
                },
                tag: tag.map(str::to_owned),
                known,
                start_ms: self.wheel.now_ms(),
                end_ms: 0,
                aborted: false,
            });
            calls.len() - 1
        };
        let outcome = match reply {
            None => Err(KindError::Message(
                "the script holds no further reply".to_owned(),
            )),
            Some(reply) => {
                self.wheel.advance(reply.delay_ms.unwrap_or(0));
                if cancel.is_cancelled() {
                    Err(KindError::Cancelled)
                } else if reply.fails {
                    Err(KindError::Message("the server failed".to_owned()))
                } else {
                    Ok((reply.models.clone(), reply.contexts.clone()))
                }
            }
        };
        let mut calls = self.calls.lock().unwrap_or_else(|held| held.into_inner());
        calls[index].end_ms = self.wheel.now_ms();
        calls[index].aborted = cancel.is_cancelled();
        outcome
    }
}

impl LocalServer for ScriptedServer {
    fn models<'a>(
        &'a self,
        known: &'a [LocalModel],
        cancel: &'a CancelToken,
    ) -> BoxFuture<'a, Result<Vec<LocalModel>, KindError>> {
        Box::pin(async move {
            let known = known.iter().map(|model| model.tag.clone()).collect();
            Ok(self.answer(Ask::Models, None, known, cancel).await?.0)
        })
    }

    fn live(&self) -> bool {
        self.live
    }

    fn reads_loaded(&self) -> bool {
        self.loaded
    }

    fn loaded<'a>(
        &'a self,
        tag: &'a str,
        cancel: &'a CancelToken,
    ) -> BoxFuture<'a, Result<Loaded, KindError>> {
        Box::pin(async move {
            Ok(self
                .answer(Ask::Loaded, Some(tag), Vec::new(), cancel)
                .await?
                .1)
        })
    }

    fn warms(&self) -> bool {
        self.warm
    }

    fn warm<'a>(
        &'a self,
        tag: &'a str,
        cancel: &'a CancelToken,
    ) -> BoxFuture<'a, Result<(), KindError>> {
        Box::pin(async move {
            self.answer(Ask::Warm, Some(tag), Vec::new(), cancel)
                .await?;
            Ok(())
        })
    }
}

/// One call of the catalog, whichever list of the script held it.
enum Call {
    Views { zdr_only: Option<bool> },
    Discovered { fresh: Option<bool> },
    ViewFor { row: String },
    Warmed { tag: String },
}

type Abort = (Option<i64>, Option<bool>);

fn call_of(held: &LocalCatalogRunInputStepsItemCallCall) -> (Call, Abort) {
    type Held = LocalCatalogRunInputStepsItemCallCall;
    match held {
        Held::Views(each) => (
            Call::Views {
                zdr_only: each.zdr_only,
            },
            (each.abort_after_ms, each.aborted_before),
        ),
        Held::Discovered(each) => (
            Call::Discovered { fresh: each.fresh },
            (each.abort_after_ms, each.aborted_before),
        ),
        Held::ViewFor(each) => (
            Call::ViewFor {
                row: each.row.clone(),
            },
            (each.abort_after_ms, each.aborted_before),
        ),
        Held::Warmed(each) => (
            Call::Warmed {
                tag: each.tag.clone(),
            },
            (each.abort_after_ms, each.aborted_before),
        ),
    }
}

fn together_of(held: &LocalCatalogRunInputStepsItemTogetherCallsItem) -> (Call, Abort) {
    type Held = LocalCatalogRunInputStepsItemTogetherCallsItem;
    match held {
        Held::Views(each) => (
            Call::Views {
                zdr_only: each.zdr_only,
            },
            (each.abort_after_ms, each.aborted_before),
        ),
        Held::Discovered(each) => (
            Call::Discovered { fresh: each.fresh },
            (each.abort_after_ms, each.aborted_before),
        ),
        Held::ViewFor(each) => (
            Call::ViewFor {
                row: each.row.clone(),
            },
            (each.abort_after_ms, each.aborted_before),
        ),
        Held::Warmed(each) => (
            Call::Warmed {
                tag: each.tag.clone(),
            },
            (each.abort_after_ms, each.aborted_before),
        ),
    }
}

type Answer = LocalCatalogRunResultStepsItemAnswersItem;

fn discovered_line(
    model: &Discovered,
) -> LocalCatalogRunResultStepsItemAnswersItemDiscoveredModelsItem {
    LocalCatalogRunResultStepsItemAnswersItemDiscoveredModelsItem {
        key: model.key.clone(),
        display: model.display.clone(),
        tag: model.tag.clone(),
        tools: model.tools,
        reasoning: model.reasoning,
        context: model.context,
    }
}

fn thrown(message: String) -> Answer {
    Answer::Thrown(LocalCatalogRunResultStepsItemAnswersItemThrown { message })
}

async fn answer_of(catalog: &LocalCatalog, call: &Call, token: &CancelToken) -> Answer {
    match call {
        Call::Views { zdr_only, .. } => {
            match catalog.views(zdr_only.unwrap_or(true), Some(token)).await {
                Ok(views) => match view_lines(&views) {
                    Ok(views) => {
                        Answer::Views(LocalCatalogRunResultStepsItemAnswersItemViews { views })
                    }
                    Err(message) => thrown(message),
                },
                Err(_) => thrown(CALLER_REASON.to_owned()),
            }
        }
        Call::Discovered { fresh } => {
            match catalog
                .discovered(fresh.unwrap_or(false), Some(token))
                .await
            {
                Ok(models) => {
                    Answer::Discovered(LocalCatalogRunResultStepsItemAnswersItemDiscovered {
                        models: models.map(|models| models.iter().map(discovered_line).collect()),
                    })
                }
                Err(_) => thrown(CALLER_REASON.to_owned()),
            }
        }
        Call::ViewFor { row, .. } => {
            let row = match local_row_of(row) {
                Ok(row) => row,
                Err(message) => return thrown(message),
            };
            match catalog.view_for(&row, Some(token)).await {
                Ok(view) => match view_line(&view) {
                    Ok(view) => {
                        Answer::View(LocalCatalogRunResultStepsItemAnswersItemView { view })
                    }
                    Err(message) => thrown(message),
                },
                Err(error) => thrown(message_of(&error)),
            }
        }
        Call::Warmed { tag } => match catalog.warmed(tag, token).await {
            Ok(()) => Answer::Warmed(LocalCatalogRunResultStepsItemAnswersItemWarmed {}),
            Err(error) => thrown(message_of(&error)),
        },
    }
}

fn scripted_replies<T>(
    held: &[T],
    parts: impl Fn(&T) -> (Vec<LocalModel>, Loaded, Option<i64>, bool),
) -> Vec<ScriptedReply> {
    held.iter()
        .map(|each| {
            let (models, contexts, delay_ms, fails) = parts(each);
            ScriptedReply {
                models,
                contexts,
                delay_ms,
                fails,
            }
        })
        .collect()
}

pub fn catalog_run(input: LocalCatalogRunInput) -> Outcome<LocalCatalogRunResult> {
    let wheel = Arc::new(TimerWheel::new(input.start_ms));
    let scheduler = Arc::new(FakeScheduler::new(Arc::clone(&wheel)));
    let registry = Arc::new(ScriptedModelRegistry::empty());
    registry.set_rows(local_rows_of(&input.rows)?);
    let script = &input.server;
    let scripted = Arc::new(ScriptedServer {
        wheel: Arc::clone(&wheel),
        live: script.live,
        loaded: script.loaded,
        warm: script.warm,
        models: scripted_replies(&script.models, |each| {
            (
                each.models.iter().map(local_of).collect(),
                Vec::new(),
                each.delay_ms,
                each.fails.is_some(),
            )
        }),
        loads: scripted_replies(&script.loads, |each| {
            (
                Vec::new(),
                each.contexts
                    .iter()
                    .map(|held| (held.tag.clone(), held.context))
                    .collect(),
                each.delay_ms,
                each.fails.is_some(),
            )
        }),
        warms: scripted_replies(&script.warms, |each| {
            (Vec::new(), Vec::new(), each.delay_ms, each.fails.is_some())
        }),
        taken: Mutex::new([0; 3]),
        calls: Mutex::new(Vec::new()),
    });
    let told = Told::new(
        input
            .on_refreshed
            .map(|held| held == LocalCatalogRunInputOnRefreshed::ReadsAndThrows),
    );
    let catalog = LocalCatalog::new(LocalCatalogDeps {
        entry: input.entry.clone(),
        server: scripted.clone(),
        terms: terms_from(&input.terms),
        scheduler: scheduler.clone(),
        rows: registry.clone() as Arc<dyn ModelRegistry>,
        refreshed: told.callback(),
    });
    told.reading({
        let catalog = catalog.clone();
        move || {
            let read = catalog.discovered(false, None);
            Some(Box::pin(async move {
                match read.await {
                    Ok(Some(held)) => held
                        .iter()
                        .map(|model| model.tag.as_str())
                        .collect::<Vec<_>>()
                        .join(" "),
                    _ => "none".to_owned(),
                }
            }) as BoxFuture<'static, String>)
        }
    });
    let mut tags = 0;
    let mut steps = Vec::new();
    block_on(async {
        for step in &input.steps {
            let served = scripted
                .calls
                .lock()
                .unwrap_or_else(|held| held.into_inner())
                .len();
            let before = told.count();
            let mut answers = Vec::new();
            match step {
                LocalCatalogRunInputStepsItem::Call(held) => {
                    tags += 1;
                    let (call, (after, aborted)) = call_of(&held.call);
                    let token = caller_token(&wheel, &format!("caller-{tags}"), after, aborted);
                    answers.push(answer_of(&catalog, &call, &token).await);
                }
                LocalCatalogRunInputStepsItem::Together(held) => {
                    let mut calls = Vec::new();
                    for each in &held.calls {
                        tags += 1;
                        let (call, (after, aborted)) = together_of(each);
                        let token = caller_token(&wheel, &format!("caller-{tags}"), after, aborted);
                        calls.push((call, token));
                    }
                    let catalog = &catalog;
                    answers = join_all(
                        calls
                            .iter()
                            .map(|(call, token)| answer_of(catalog, call, token)),
                    )
                    .await;
                }
                LocalCatalogRunInputStepsItem::Settled(held) => {
                    for _ in 0..held.times.unwrap_or(1) {
                        catalog.settled(&held.tag);
                    }
                }
                LocalCatalogRunInputStepsItem::Rediscover(_) => catalog.rediscover(),
                LocalCatalogRunInputStepsItem::Advance(held) => {
                    wheel.advance(held.ms as i64);
                }
                LocalCatalogRunInputStepsItem::Rows(held) => {
                    if let Ok(rows) = local_rows_of(&held.rows) {
                        registry.set_rows(rows);
                    }
                }
            }
            catalog.idle().await;
            let calls = scripted
                .calls
                .lock()
                .unwrap_or_else(|held| held.into_inner());
            steps.push(LocalCatalogRunResultStepsItem {
                now_ms: wheel.now_ms(),
                answers,
                served: calls[served..]
                    .iter()
                    .map(|call| LocalCatalogRunResultStepsItemServedItem {
                        method: call.method,
                        tag: call.tag.clone(),
                        known: call.known.clone(),
                        start_ms: call.start_ms,
                        end_ms: call.end_ms,
                        aborted: call.aborted,
                    })
                    .collect(),
                refreshed: told.count() - before,
            });
        }
    })?;
    for tag in 1..=tags {
        wheel.cancel(&format!("caller-{tag}"));
    }
    let saw = block_on(told.seen())?;
    Ok(LocalCatalogRunResult {
        steps,
        waiting: wheel.waiting(),
        saw,
    })
}

////////////////////////////////
//  A kind's flow

fn chat_request(model: &str) -> Outcome<CanonicalRequest> {
    let text = format!(
        "{{\"model\":{},\"system\":[\"Rules.\"],\"history\":[{{\"role\":\"user\",\"text\":\"Hi\"}}],\"cache\":{{\"kind\":\"implicit\"}},\"session\":\"root-1\",\"zdrOnly\":true}}",
        nyaaai_contract::json::quote_json(model)
    );
    let value = parse_json(&text).ok_or_else(|| "the chat request does not parse".to_owned())?;
    CanonicalRequest::decode(&value).map_err(|refused| refused.to_string())
}

type Flow = LocalFlowResultStepsItemAnswer;

fn flow_thrown(message: String, category: Option<String>) -> Flow {
    Flow::Thrown(LocalFlowResultStepsItemAnswerThrown { message, category })
}

pub fn flow(input: LocalFlowInput) -> Outcome<LocalFlowResult> {
    let which = Which::from(&input.kind);
    let net = net_of(input.start_ms, &input.exchanges);
    let named = which.kind();
    let fields = entry_of(which, &input.entry.label, &input.entry.url);
    let entry = ProviderEntry {
        id: input.entry.id.clone(),
        kind: which.name(),
        label: input.entry.label.clone(),
        plan: None,
        url: input.entry.url.clone(),
        extra_usage: None,
    };
    let registry = Arc::new(ScriptedModelRegistry::empty());
    registry.set_rows(local_rows_of(&input.rows)?);
    let refreshed = Arc::new(AtomicI64::new(0));
    let told = Arc::clone(&refreshed);
    let transport = transport_of(which, &fields, &net.deps, &input.stored)?;
    let catalog = named.catalog(
        &entry,
        Arc::clone(&transport),
        CatalogDeps {
            scheduler: net.scheduler.clone(),
            rows: registry.clone() as Arc<dyn ModelRegistry>,
            refreshed: Arc::new(move || {
                told.fetch_add(1, Ordering::SeqCst);
            }),
        },
    );
    let route = Route {
        transport: Arc::clone(&transport),
        catalog: Arc::clone(&catalog),
    };
    let (Some(browser), Some(tester)) = (
        named.browse(&entry, &route, &net.deps),
        named.tester(&entry, &route, &net.deps),
    ) else {
        return Err("A local kind has a browser and a tester.".to_owned());
    };
    let mut tags = 0;
    let mut steps = Vec::new();
    block_on(async {
        for step in &input.steps {
            tags += 1;
            let tag = format!("caller-{tags}");
            let (after, before) = match step {
                LocalFlowInputStepsItem::Views(held) => (held.abort_after_ms, held.aborted_before),
                LocalFlowInputStepsItem::Chat(held) => (held.abort_after_ms, held.aborted_before),
                LocalFlowInputStepsItem::Detail(held) => (held.abort_after_ms, held.aborted_before),
                LocalFlowInputStepsItem::Listed(held) => (held.abort_after_ms, held.aborted_before),
                LocalFlowInputStepsItem::View(held) => (held.abort_after_ms, held.aborted_before),
                _ => (None, None),
            };
            let token = caller_token(&net.wheel, &tag, after, before);
            let answer: Flow = match step {
                LocalFlowInputStepsItem::Views(held) => {
                    match catalog
                        .views(
                            held.use_.unwrap_or(ModelUse::Chat),
                            held.zdr_only.unwrap_or(true),
                            Some(&token),
                        )
                        .await
                    {
                        Ok(views) => match view_lines(&views) {
                            Ok(views) => Flow::Views(LocalFlowResultStepsItemAnswerViews { views }),
                            Err(message) => flow_thrown(message, None),
                        },
                        Err(_) => flow_thrown(CALLER_REASON.to_owned(), None),
                    }
                }
                LocalFlowInputStepsItem::Chat(held) => {
                    let codec = named.codec(None, &fields, &net.deps);
                    match chat_request(&held.model) {
                        Err(message) => flow_thrown(message, None),
                        Ok(request) => match transport.complete(&*codec, &request, &token).await {
                            Ok(result) => Flow::Chat(LocalFlowResultStepsItemAnswerChat {
                                text: reply_text(&result),
                            }),
                            Err(error) => flow_thrown(
                                transport_message(&error),
                                match &error {
                                    TransportError::Provider(held) => {
                                        Some(held.failure.category.as_str().to_owned())
                                    }
                                    TransportError::Cancelled => None,
                                },
                            ),
                        },
                    }
                }
                LocalFlowInputStepsItem::List(held) => {
                    match browser.list(held.fresh.unwrap_or(false)).await {
                        Ok(models) => Flow::List(LocalFlowResultStepsItemAnswerList { models }),
                        Err(error) => flow_thrown(message_of(&error), category_of(&error)),
                    }
                }
                LocalFlowInputStepsItem::Detail(held) => {
                    match browser.detail(&held.id, &token).await {
                        Ok(detail) => Flow::Detail(LocalFlowResultStepsItemAnswerDetail { detail }),
                        Err(error) => flow_thrown(message_of(&error), category_of(&error)),
                    }
                }
                LocalFlowInputStepsItem::Listed(held) => {
                    match tester.listed(&held.id, &token).await {
                        Ok(listed) => Flow::Listed(LocalFlowResultStepsItemAnswerListed {
                            listed: listed.map(|held| LocalFlowResultStepsItemAnswerListedListed {
                                label: held.label,
                                reasoning: held.reasoning,
                            }),
                        }),
                        Err(error) => flow_thrown(message_of(&error), category_of(&error)),
                    }
                }
                LocalFlowInputStepsItem::View(held) => {
                    let row = local_row_of(&held.row)?;
                    match tester
                        .view(&row, held.use_.unwrap_or(ModelUse::Chat), &token)
                        .await
                    {
                        Ok(view) => match view_line(&view) {
                            Ok(view) => Flow::View(LocalFlowResultStepsItemAnswerView { view }),
                            Err(message) => flow_thrown(message, None),
                        },
                        Err(error) => flow_thrown(message_of(&error), category_of(&error)),
                    }
                }
                LocalFlowInputStepsItem::Advance(held) => {
                    net.wheel.advance(held.ms as i64);
                    Flow::None(LocalFlowResultStepsItemAnswerNone {})
                }
                LocalFlowInputStepsItem::Rows(held) => {
                    registry.set_rows(local_rows_of(&held.rows)?);
                    Flow::None(LocalFlowResultStepsItemAnswerNone {})
                }
            };
            catalog.idle().await;
            steps.push(LocalFlowResultStepsItem {
                now_ms: net.wheel.now_ms(),
                answer,
            });
        }
        Ok::<(), String>(())
    })??;
    for tag in 1..=tags {
        net.wheel.cancel(&format!("caller-{tag}"));
    }
    Ok(LocalFlowResult {
        steps,
        requests: request_lines!(net.played, LocalFlowResultRequestsItem),
        read: net
            .played
            .read()
            .into_iter()
            .map(|read| read as i64)
            .collect(),
        waiting: net.wheel.waiting(),
        refreshed: refreshed.load(Ordering::SeqCst),
    })
}

////////////////////////////////
//  What a kind says

pub fn kind_facts(input: LocalKindFactsInput) -> Outcome<LocalKindFactsResult> {
    let which = Which::from(&input.kind);
    let entry = entry_of(which, "Lab", &input.url);
    Ok(LocalKindFactsResult {
        facts: input
            .states
            .iter()
            .map(|state| which.kind().facts(state, &entry, None))
            .collect(),
    })
}

/// A view of a made-up row that reads as the codec's facts say.
fn view_of_facts(held: &Option<LocalKindCodecInputView>) -> Outcome<Option<ModelView>> {
    let Some(held) = held else {
        return Ok(None);
    };
    let row = MODELS
        .first()
        .ok_or_else(|| "The curated list is empty.".to_owned())?;
    let mut view = unknown_view(row);
    view.status = ModelViewStatus::Ok;
    view.efforts = held.efforts.clone();
    view.api = held.api.clone();
    view.output_limit = held.output_limit;
    view.price = held.price.as_ref().map(|price| match price {
        LocalKindCodecInputViewPrice::Free => ModelViewPrice::Free,
        LocalKindCodecInputViewPrice::Unknown => ModelViewPrice::Unknown,
    });
    Ok(Some(view))
}

pub fn kind_codec(input: LocalKindCodecInput) -> Outcome<LocalKindCodecResult> {
    let which = Which::from(&input.kind);
    let view = view_of_facts(&input.view)?;
    let fields = EntryFields {
        kind: which.name(),
        label: input.label.clone(),
        plan: None,
        url: None,
        extra_usage: None,
    };
    let codec = which
        .kind()
        .codec(view.as_ref(), &fields, &net_of(0, &[]).deps);
    let facts = codec_facts_of(view.as_ref());
    let mode = match input.mode {
        LocalKindCodecInputMode::Stream => Mode::Stream,
        LocalKindCodecInputMode::Complete => Mode::Complete,
    };
    let (encoded, refusal) = match codec.encode(&input.request, mode) {
        Ok(encoded) => (Some(encoded), None),
        Err(message) => (None, Some(message)),
    };
    Ok(LocalKindCodecResult {
        encoded,
        refusal,
        facts: facts.map(|facts| LocalKindCodecResultFacts {
            efforts: facts.efforts,
            free: facts.free == Some(true),
            output_limit: facts.output_limit.map(|limit| limit as f64),
        }),
        failures: input
            .failures
            .iter()
            .map(|each| codec.failure(each.status, &each.body, &Vec::new()))
            .collect(),
    })
}

pub fn kind_disposition(input: LocalKindDispositionInput) -> Outcome<LocalKindDispositionResult> {
    let which = Which::from(&input.kind);
    let row = MODELS
        .first()
        .ok_or_else(|| "The curated list is empty.".to_owned())?;
    Ok(LocalKindDispositionResult {
        dispositions: input
            .rows
            .iter()
            .map(|each| {
                let view = each.status.as_ref().map(|status| {
                    let mut view = unknown_view(row);
                    view.status = match status {
                        LocalKindDispositionInputRowsItemStatus::Ok => ModelViewStatus::Ok,
                        LocalKindDispositionInputRowsItemStatus::Retired => {
                            ModelViewStatus::Retired
                        }
                        LocalKindDispositionInputRowsItemStatus::NoZdrHost => {
                            ModelViewStatus::NoZdrHost
                        }
                        LocalKindDispositionInputRowsItemStatus::NoTools => {
                            ModelViewStatus::NoTools
                        }
                        LocalKindDispositionInputRowsItemStatus::Unknown => {
                            ModelViewStatus::Unknown
                        }
                    };
                    view
                });
                which.kind().failure(&each.failure, view.as_ref())
            })
            .collect(),
    })
}
