//! Catalog cases.

use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use nyaaai_agentic::dialects::openrouter::OPENROUTER_CHAT;
use nyaaai_agentic::kinds::opencode::catalog::{
    OpenCodeCatalog, OpenCodeCatalogDeps, OpenCodeModel, join_open_code, parse_open_code,
};
use nyaaai_agentic::kinds::openrouter::catalog::{
    Endpoint, ModelCatalog, ModelCatalogDeps, ModelEntry, ParamNeed, Pricing, Standing,
    chat_standing, expiry_of, join_catalog, parse_endpoints, parse_models, per_million as price_of,
};
use nyaaai_agentic::models::curated::MODELS;
use nyaaai_agentic::models::registry::row_of;
use nyaaai_agentic::models::row_key::parse_row_key_text;
use nyaaai_agentic::models::uses::CHAT;
use nyaaai_agentic::provider::kind::{CatalogDeps, EntryCatalog};
use nyaaai_agentic::transport::{Auth, Forbidden, KeySource, Transport, TransportDeps};
use nyaaai_contract::cancel::CancelToken;
use nyaaai_contract::future::BoxFuture;
use nyaaai_contract::json::{Object, Value, stringify_json};
use nyaaai_contract::model_registry::ModelRegistry;
use nyaaai_contract::shape::Shape;
use nyaaai_contract::shapes::model::{
    ModelRow, ModelUse, ModelView, RowIdentity, StoredRegistry, TestRecord, TestRecordCache,
    TestRecordChecks, TestRecordChecksReasoning, TestRecordChecksRecovery, TestRecordChecksSchema,
    TestRecordChecksToolPlus, TestRecordVerifiedPrice,
};
use nyaaai_testkit::cases::{Outcome, block_on};
use nyaaai_testkit::keys::FakeKeys;
use nyaaai_testkit::scheduler::{FakeScheduler, TimerWheel};
use nyaaai_testkit::scripted_fetch::{ScriptedChunk, ScriptedEnd, ScriptedExchange, ScriptedFetch};
use nyaaai_testkit::scripted_ports::ScriptedModelRegistry;

use nyaaai_agentic::kinds::join::join_all;

use super::shapes::*;

pub(super) const BASE_URL: &str = "https://catalog.test/v1";

pub(super) const CALLER_REASON: &str = "caller aborted";

/// One generated shape as another with the same wire form.
pub(super) fn convert<From: Shape, To: Shape>(held: &From) -> Outcome<To> {
    To::decode(&held.encode()).map_err(|refused| refused.to_string())
}

/// A passing record for a row made up from its key.
fn passing() -> TestRecord {
    TestRecord {
        label: "Made Up".to_owned(),
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
            prompt: 1.0,
            completion: 2.0,
        },
    }
}

/// A curated row by key, or a hosted row (`provider:id`) made up.
pub(super) fn row_of_key(key: &str) -> Outcome<ModelRow> {
    if let Some(row) = MODELS.iter().find(|row| row.key == key) {
        return Ok(row.clone());
    }
    let identity = parse_row_key_text(key);
    let Some(RowIdentity::Hosted(hosted)) = &identity else {
        return Err(format!("No row {key}."));
    };
    identity
        .as_ref()
        .and_then(|held| row_of(held, &passing(), hosted.provider))
        .ok_or_else(|| format!("No row {key}."))
}

pub(super) fn rows_of_keys(keys: &[String]) -> Outcome<Vec<ModelRow>> {
    keys.iter().map(|key| row_of_key(key)).collect()
}

pub(super) fn view_line(view: &ModelView) -> Outcome<ViewLine> {
    Ok(ViewLine {
        key: view.row.key.clone(),
        status: convert(&view.status)?,
        retired_on: view.retired_on.clone(),
        efforts: view.efforts.clone(),
        prompt_limit: view.prompt_limit,
        output_limit: view.output_limit,
        price: match &view.price {
            None => None,
            Some(price) => Some(convert(price)?),
        },
        cache_write1h: view.cache_write1h,
        hosts: view.hosts.clone(),
        data_kept: view.data_kept,
        api: view.api.clone(),
        cache: match &view.cache {
            None => None,
            Some(cache) => Some(convert(cache)?),
        },
    })
}

pub(super) fn view_lines(views: &[ModelView]) -> Outcome<Vec<ViewLine>> {
    views.iter().map(view_line).collect()
}

struct Keys(Arc<FakeKeys>);

impl KeySource for Keys {
    fn get(&self) -> BoxFuture<'_, Result<Option<String>, Forbidden>> {
        let read = self.0.get();
        Box::pin(async move { Ok(read.await) })
    }
}

/// A transport on a wheel, over the exchanges in order.
pub(super) struct Net {
    pub wheel: Arc<TimerWheel>,
    pub scheduler: Arc<FakeScheduler>,
    pub transport: Arc<Transport>,
    pub played: Arc<ScriptedFetch>,
}

impl Net {
    /// Paths asked, in order.
    pub fn asked(&self) -> Vec<String> {
        self.played
            .requests()
            .iter()
            .map(|sent| sent.url[BASE_URL.len()..].to_owned())
            .collect()
    }
}

pub(super) fn exchange_of(held: &NetExchange) -> ScriptedExchange {
    let body = held
        .text
        .clone()
        .unwrap_or_else(|| stringify_json(held.json.as_ref().unwrap_or(&Value::Null)));
    ScriptedExchange {
        status: held.status as u16,
        headers: vec![("content-type".to_owned(), "application/json".to_owned())],
        chunks: vec![ScriptedChunk {
            bytes: body.into_bytes(),
            delay_ms: 0,
        }],
        head_ms: held.head_ms.unwrap_or(0),
        end: match held.end {
            None | Some(NetExchangeEnd::Closed) => ScriptedEnd::Closed,
            Some(NetExchangeEnd::Dropped) => ScriptedEnd::Dropped,
            Some(NetExchangeEnd::Refused) => ScriptedEnd::Refused,
        },
        error: None,
    }
}

pub(super) fn network_of(
    start_ms: i64,
    exchanges: &[NetExchange],
    stored: Option<&str>,
) -> Outcome<Net> {
    let wheel = Arc::new(TimerWheel::new(start_ms));
    let scheduler = Arc::new(FakeScheduler::new(Arc::clone(&wheel)));
    let played = Arc::new(ScriptedFetch::new(
        exchanges.iter().map(exchange_of).collect(),
        Arc::clone(&scheduler),
    ));
    let keys = Arc::new(
        FakeKeys::new(vec![Some(stored.unwrap_or("fake-key-one").to_owned())])
            .ok_or_else(|| "fakeKeys needs a value".to_owned())?,
    );
    let transport = Arc::new(Transport::new(TransportDeps {
        fetch: played.clone(),
        scheduler: scheduler.clone(),
        key: Arc::new(Keys(keys)),
        log: Box::new(|_| {}),
        base_url: BASE_URL.to_owned(),
        label: "Catalog".to_owned(),
        user_agent: "cases/1.0".to_owned(),
        errors: Arc::new(OPENROUTER_CHAT),
        auth: Auth::Bearer,
        budgets: None,
    }));
    Ok(Net {
        wheel,
        scheduler,
        transport,
        played,
    })
}

/// A token of the caller's, cancelled at its time on the wheel.
pub(super) fn caller_token(
    wheel: &TimerWheel,
    tag: &str,
    after: Option<i64>,
    before: Option<bool>,
) -> CancelToken {
    let caller = CancelToken::new();
    if before == Some(true) {
        caller.cancel();
    } else if let Some(ms) = after {
        let held = caller.clone();
        wheel.after_run(ms, tag, Box::new(move || held.cancel()));
    }
    caller
}

fn plain(value: &Option<Value>) -> Value {
    value.clone().unwrap_or(Value::Null)
}

fn text_value(held: &Option<String>) -> Value {
    held.as_deref().map_or(Value::Null, Value::from)
}

fn texts_value(held: &Option<Vec<String>>) -> Value {
    held.as_ref().map_or(Value::Null, |items| {
        Value::Array(
            items
                .iter()
                .map(|item| Value::from(item.as_str()))
                .collect(),
        )
    })
}

fn number_value(held: Option<f64>) -> Value {
    held.map_or(Value::Null, Value::Number)
}

fn object_value(entries: Vec<(&str, Value)>) -> Value {
    let mut object = Object::new();
    for (key, value) in entries {
        object.insert(key, value);
    }
    Value::Object(object)
}

fn pricing_value(pricing: &Option<Pricing>) -> Value {
    let Some(held) = pricing else {
        return Value::Null;
    };
    object_value(vec![
        ("prompt", plain(&held.prompt)),
        ("completion", plain(&held.completion)),
        ("input_cache_read", plain(&held.input_cache_read)),
        ("input_cache_write", plain(&held.input_cache_write)),
        ("input_cache_write_1h", plain(&held.input_cache_write_1h)),
        (
            "overrides",
            held.overrides.as_ref().map_or(Value::Null, |items| {
                Value::Array(
                    items
                        .iter()
                        .map(|item| {
                            object_value(vec![
                                ("min_prompt_tokens", Value::Number(item.min_prompt_tokens)),
                                ("prompt", plain(&item.prompt)),
                                ("completion", plain(&item.completion)),
                                ("input_cache_read", plain(&item.input_cache_read)),
                                ("input_cache_write", plain(&item.input_cache_write)),
                                ("input_cache_write_1h", plain(&item.input_cache_write_1h)),
                            ])
                        })
                        .collect(),
                )
            }),
        ),
    ])
}

fn model_value(entry: &ModelEntry) -> Value {
    object_value(vec![
        ("id", Value::from(entry.id.as_str())),
        ("name", text_value(&entry.name)),
        ("created", number_value(entry.created)),
        ("context_length", number_value(entry.context_length)),
        ("expiration_date", text_value(&entry.expiration_date)),
        ("pricing", pricing_value(&entry.pricing)),
        (
            "reasoning",
            entry.reasoning.as_ref().map_or(Value::Null, |held| {
                object_value(vec![(
                    "supported_efforts",
                    texts_value(&held.supported_efforts),
                )])
            }),
        ),
        (
            "supported_parameters",
            texts_value(&entry.supported_parameters),
        ),
    ])
}

fn endpoint_value(endpoint: &Endpoint) -> Value {
    object_value(vec![
        ("model_id", Value::from(endpoint.model_id.as_str())),
        ("tag", text_value(&endpoint.tag)),
        ("provider_name", text_value(&endpoint.provider_name)),
        ("context_length", number_value(endpoint.context_length)),
        (
            "max_prompt_tokens",
            number_value(endpoint.max_prompt_tokens),
        ),
        (
            "supported_parameters",
            texts_value(&endpoint.supported_parameters),
        ),
        ("pricing", pricing_value(&endpoint.pricing)),
    ])
}

fn open_code_value(model: &OpenCodeModel) -> Value {
    let flag = |held: Option<bool>| held.map_or(Value::Null, Value::Bool);
    object_value(vec![
        ("id", Value::from(model.id.as_str())),
        ("name", text_value(&model.name)),
        ("description", text_value(&model.description)),
        ("input", texts_value(&model.input)),
        ("output", texts_value(&model.output)),
        ("tools", flag(model.tools)),
        ("reasoning", flag(model.reasoning)),
        ("efforts", texts_value(&model.efforts)),
        ("toggle", flag(model.toggle)),
        ("structured_output", flag(model.structured_output)),
        (
            "cache",
            model.cache.as_ref().map_or(Value::Null, |held| {
                object_value(vec![
                    ("implicit", flag(held.implicit)),
                    ("explicit", Value::Bool(held.explicit)),
                ])
            }),
        ),
        ("context", number_value(model.context)),
        ("input_limit", number_value(model.input_limit)),
        ("output_limit", number_value(model.output_limit)),
        (
            "price",
            model.price.map_or(Value::Null, |(input, output)| {
                object_value(vec![
                    ("input", Value::Number(input)),
                    ("output", Value::Number(output)),
                ])
            }),
        ),
        (
            "api",
            Value::Array(
                model
                    .api
                    .iter()
                    .map(|api| {
                        object_value(vec![
                            ("family", Value::from(api.family.as_str())),
                            ("reasoning_field", text_value(&api.reasoning_field)),
                            ("choice", texts_value(&api.choice)),
                            ("strict", flag(api.strict)),
                        ])
                    })
                    .collect(),
            ),
        ),
    ])
}

fn decoded<T: Shape>(value: &Value) -> Outcome<T> {
    T::decode(value).map_err(|refused| refused.to_string())
}

pub fn openrouter_models(
    input: CatalogOpenrouterModelsInput,
) -> Outcome<CatalogOpenrouterModelsResult> {
    let models = match parse_models(&input.body) {
        None => None,
        Some(parsed) => Some(
            parsed
                .iter()
                .map(|(_, entry)| decoded(&model_value(entry)))
                .collect::<Outcome<Vec<_>>>()?,
        ),
    };
    Ok(CatalogOpenrouterModelsResult { models })
}

pub fn openrouter_endpoints(
    input: CatalogOpenrouterEndpointsInput,
) -> Outcome<CatalogOpenrouterEndpointsResult> {
    let endpoints = match parse_endpoints(&input.body) {
        None => None,
        Some(parsed) => Some(
            parsed
                .iter()
                .map(|(model, hosts)| {
                    Ok(CatalogOpenrouterEndpointsResultEndpointsItem {
                        model: model.to_owned(),
                        hosts: hosts
                            .iter()
                            .map(|host| decoded(&endpoint_value(host)))
                            .collect::<Outcome<Vec<_>>>()?,
                    })
                })
                .collect::<Outcome<Vec<_>>>()?,
        ),
    };
    Ok(CatalogOpenrouterEndpointsResult { endpoints })
}

pub fn opencode_models(input: CatalogOpencodeModelsInput) -> Outcome<CatalogOpencodeModelsResult> {
    let models = match parse_open_code(&input.body) {
        None => None,
        Some(parsed) => Some(
            parsed
                .iter()
                .map(|(_, model)| decoded(&open_code_value(model)))
                .collect::<Outcome<Vec<_>>>()?,
        ),
    };
    Ok(CatalogOpencodeModelsResult { models })
}

pub fn openrouter_join(input: CatalogOpenrouterJoinInput) -> Outcome<CatalogOpenrouterJoinResult> {
    let rows = rows_of_keys(&input.rows)?;
    let passes = input
        .passes
        .iter()
        .map(|pass| {
            view_lines(&join_catalog(
                &rows,
                &input.models,
                &input.zdr,
                input.now_ms,
                &pass.use_,
                pass.zdr_only,
            ))
        })
        .collect::<Outcome<Vec<_>>>()?;
    Ok(CatalogOpenrouterJoinResult { passes })
}

pub fn opencode_join(input: CatalogOpencodeJoinInput) -> Outcome<CatalogOpencodeJoinResult> {
    let rows = rows_of_keys(&input.rows)?;
    let models = parse_open_code(&input.models);
    let passes = input
        .passes
        .iter()
        .map(|pass| {
            view_lines(&join_open_code(
                &rows,
                models.as_ref(),
                input.plan,
                &pass.use_,
                pass.zdr_only,
                input.now_ms,
            ))
        })
        .collect::<Outcome<Vec<_>>>()?;
    Ok(CatalogOpencodeJoinResult { passes })
}

pub fn per_million(input: CatalogPerMillionInput) -> Outcome<CatalogPerMillionResult> {
    Ok(CatalogPerMillionResult {
        prices: input
            .values
            .iter()
            .map(|value| price_of(Some(value)))
            .collect(),
    })
}

pub fn expiry(input: CatalogExpiryInput) -> Outcome<CatalogExpiryResult> {
    Ok(CatalogExpiryResult {
        ms: input.texts.iter().map(|text| expiry_of(text)).collect(),
    })
}

pub fn standing(input: CatalogStandingInput) -> Outcome<CatalogStandingResult> {
    let rows = input
        .rows
        .iter()
        .map(|row| {
            let hosts: Vec<Endpoint> = row
                .hosts
                .iter()
                .enumerate()
                .map(|(at, host)| Endpoint {
                    model_id: "m".to_owned(),
                    tag: Some(format!("h{at}")),
                    provider_name: None,
                    context_length: None,
                    max_prompt_tokens: None,
                    supported_parameters: host.params.clone(),
                    pricing: None,
                })
                .collect();
            let needs: Vec<ParamNeed> = row
                .needs
                .iter()
                .map(|need| match need {
                    CatalogStandingInputRowsItemNeedsItem::Text(text) => {
                        ParamNeed::One(text.clone())
                    }
                    CatalogStandingInputRowsItemNeedsItem::List(list) => {
                        ParamNeed::Any(list.clone())
                    }
                })
                .collect();
            let (held, usable) =
                chat_standing(row.expiration.as_deref(), &hosts, &needs, row.now_ms);
            CatalogStandingResultRowsItem {
                standing: match held {
                    Standing::Ok => CatalogStandingResultRowsItemStanding::Ok,
                    Standing::Retired => CatalogStandingResultRowsItemStanding::Retired,
                    Standing::NoZdrHost => CatalogStandingResultRowsItemStanding::NoZdrHost,
                },
                usable: usable.into_iter().map(|at| at as i64).collect(),
            }
        })
        .collect();
    Ok(CatalogStandingResult { rows })
}

/// A call on a catalog, in the spelling both kinds' scripts convert to.
#[derive(Clone)]
enum Call {
    Views {
        use_: Option<nyaaai_contract::shapes::model::ModelUse>,
        zdr_only: Option<bool>,
    },
    View {
        key: String,
        use_: Option<nyaaai_contract::shapes::model::ModelUse>,
        zdr_only: Option<bool>,
    },
    ViewFor {
        row: String,
        use_: Option<nyaaai_contract::shapes::model::ModelUse>,
    },
    Entry {
        id: String,
    },
    Listing {
        retry: Option<bool>,
    },
    Models {
        fresh: Option<bool>,
    },
}

struct Placed {
    call: Call,
    after: Option<i64>,
    before: Option<bool>,
}

fn placed_openrouter(call: &CatalogOpenrouterRunInputStepsItemCallCall) -> Placed {
    use CatalogOpenrouterRunInputStepsItemCallCall as C;
    match call {
        C::Views(held) => Placed {
            call: Call::Views {
                use_: held.use_.clone(),
                zdr_only: held.zdr_only,
            },
            after: held.abort_after_ms,
            before: held.aborted_before,
        },
        C::View(held) => Placed {
            call: Call::View {
                key: held.key.clone(),
                use_: held.use_.clone(),
                zdr_only: held.zdr_only,
            },
            after: held.abort_after_ms,
            before: held.aborted_before,
        },
        C::ViewFor(held) => Placed {
            call: Call::ViewFor {
                row: held.row.clone(),
                use_: held.use_.clone(),
            },
            after: held.abort_after_ms,
            before: held.aborted_before,
        },
        C::Entry(held) => Placed {
            call: Call::Entry {
                id: held.id.clone(),
            },
            after: held.abort_after_ms,
            before: held.aborted_before,
        },
        C::Listing(held) => Placed {
            call: Call::Listing { retry: held.retry },
            after: held.abort_after_ms,
            before: held.aborted_before,
        },
    }
}

fn placed_opencode(call: &CatalogOpencodeRunInputStepsItemCallCall) -> Placed {
    use CatalogOpencodeRunInputStepsItemCallCall as C;
    match call {
        C::Views(held) => Placed {
            call: Call::Views {
                use_: held.use_.clone(),
                zdr_only: held.zdr_only,
            },
            after: held.abort_after_ms,
            before: held.aborted_before,
        },
        C::ViewFor(held) => Placed {
            call: Call::ViewFor {
                row: held.row.clone(),
                use_: held.use_.clone(),
            },
            after: held.abort_after_ms,
            before: held.aborted_before,
        },
        C::Models(held) => Placed {
            call: Call::Models { fresh: held.fresh },
            after: held.abort_after_ms,
            before: held.aborted_before,
        },
    }
}

/// A step of a script.
enum Step {
    Calls(Vec<Placed>),
    Advance(u64),
    Rows(Vec<String>),
}

fn steps_openrouter(steps: &[CatalogOpenrouterRunInputStepsItem]) -> Vec<Step> {
    steps
        .iter()
        .map(|step| match step {
            CatalogOpenrouterRunInputStepsItem::Call(held) => {
                Step::Calls(vec![placed_openrouter(&held.call)])
            }
            CatalogOpenrouterRunInputStepsItem::Together(held) => Step::Calls(
                held.calls
                    .iter()
                    .filter_map(|each| {
                        CatalogOpenrouterRunInputStepsItemCallCall::decode(&each.encode()).ok()
                    })
                    .map(|each| placed_openrouter(&each))
                    .collect(),
            ),
            CatalogOpenrouterRunInputStepsItem::Advance(held) => Step::Advance(held.ms),
            CatalogOpenrouterRunInputStepsItem::Rows(held) => Step::Rows(held.rows.clone()),
        })
        .collect()
}

fn steps_opencode(steps: &[CatalogOpencodeRunInputStepsItem]) -> Vec<Step> {
    steps
        .iter()
        .map(|step| match step {
            CatalogOpencodeRunInputStepsItem::Call(held) => {
                Step::Calls(vec![placed_opencode(&held.call)])
            }
            CatalogOpencodeRunInputStepsItem::Together(held) => Step::Calls(
                held.calls
                    .iter()
                    .filter_map(|each| {
                        CatalogOpencodeRunInputStepsItemCallCall::decode(&each.encode()).ok()
                    })
                    .map(|each| placed_opencode(&each))
                    .collect(),
            ),
            CatalogOpencodeRunInputStepsItem::Advance(held) => Step::Advance(held.ms),
            CatalogOpencodeRunInputStepsItem::Rows(held) => Step::Rows(held.rows.clone()),
        })
        .collect()
}

/// What one call answered, as a value of the answer's own shape.
fn answer_value(kind: &str, fields: Vec<(&str, Value)>) -> Value {
    let mut entries = vec![("kind", Value::from(kind))];
    entries.extend(fields);
    object_value(entries)
}

fn views_value(views: &[ModelView]) -> Outcome<Value> {
    Ok(Value::Array(
        view_lines(views)?.iter().map(Shape::encode).collect(),
    ))
}

fn thrown(message: &str) -> Value {
    answer_value("thrown", vec![("message", Value::from(message))])
}

type Answered<'a> = Pin<Box<dyn Future<Output = Outcome<Value>> + 'a>>;

/// What a script run gives: time, answers and row reads per step.
type Played = Vec<(i64, Vec<Value>, i64)>;

/// Plays a script on a catalog; `answer` runs one call.
async fn play<'a>(
    net: &Net,
    registry: &ScriptedModelRegistry,
    steps: Vec<Step>,
    idle: impl Fn() -> BoxFuture<'a, ()>,
    answer: impl Fn(Call, CancelToken) -> Answered<'a>,
) -> Outcome<Played> {
    let mut tags = 0;
    let mut outcomes = Vec::new();
    for step in steps {
        let reads = registry.reads();
        let mut answers: Vec<Value> = Vec::new();
        match step {
            Step::Calls(calls) => {
                let mut futures: Vec<Answered<'a>> = Vec::new();
                for placed in calls {
                    tags += 1;
                    let token = caller_token(
                        &net.wheel,
                        &format!("caller-{tags}"),
                        placed.after,
                        placed.before,
                    );
                    futures.push(answer(placed.call, token));
                }
                for held in join_all(futures).await {
                    answers.push(held?);
                }
            }
            Step::Advance(ms) => {
                net.wheel.advance(ms as i64);
            }
            Step::Rows(keys) => registry.set_rows(rows_of_keys(&keys)?),
        }
        idle().await;
        outcomes.push((
            nyaaai_contract::time::Clock::now_ms(&*net.wheel),
            answers,
            (registry.reads() - reads) as i64,
        ));
    }
    for tag in 1..=tags {
        net.wheel.cancel(&format!("caller-{tag}"));
    }
    Ok(outcomes)
}

fn registry_of(keys: &[String]) -> Outcome<Arc<ScriptedModelRegistry>> {
    Ok(Arc::new(ScriptedModelRegistry::new(
        rows_of_keys(keys)?,
        StoredRegistry {
            tested: Default::default(),
            added: Default::default(),
            hidden: Vec::new(),
        },
    )))
}

fn use_of(held: Option<ModelUse>) -> ModelUse {
    held.unwrap_or_else(|| CHAT.clone())
}

/// The script's `refreshed`: counts, may read its catalog and throw.
pub(super) struct Told {
    count: Arc<AtomicI64>,
    saw: Arc<Mutex<Vec<BoxFuture<'static, String>>>>,
    read: Arc<OnceLock<ReadCatalog>>,
    /// `Some(throws)` when it reads the catalog.
    reads: Option<bool>,
}

type ReadCatalog = Box<dyn Fn() -> Option<BoxFuture<'static, String>> + Send + Sync>;

impl Told {
    pub(super) fn new(reads: Option<bool>) -> Self {
        Self {
            count: Arc::new(AtomicI64::new(0)),
            saw: Arc::new(Mutex::new(Vec::new())),
            read: Arc::new(OnceLock::new()),
            reads,
        }
    }

    pub(super) fn callback(&self) -> Arc<dyn Fn() + Send + Sync> {
        let count = Arc::clone(&self.count);
        let saw = Arc::clone(&self.saw);
        let read = Arc::clone(&self.read);
        let reads = self.reads;
        Arc::new(move || {
            count.fetch_add(1, Ordering::SeqCst);
            let Some(throws) = reads else {
                return;
            };
            if let Some(started) = read.get().and_then(|held| held()) {
                saw.lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .push(started);
            }
            if throws {
                std::panic::resume_unwind(Box::new("told badly"));
            }
        })
    }

    /// How the callback starts its read of the catalog.
    pub(super) fn reading(
        &self,
        read: impl Fn() -> Option<BoxFuture<'static, String>> + Send + Sync + 'static,
    ) {
        let _ = self.read.set(Box::new(read));
    }

    pub(super) fn count(&self) -> i64 {
        self.count.load(Ordering::SeqCst)
    }

    /// What each callback read, in order.
    pub(super) async fn seen(&self) -> Vec<String> {
        let reads: Vec<BoxFuture<'static, String>> = std::mem::take(
            &mut *self
                .saw
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()),
        );
        join_all(reads).await
    }
}

pub fn openrouter_run(input: CatalogOpenrouterRunInput) -> Outcome<CatalogOpenrouterRunResult> {
    let net = network_of(input.start_ms, &input.exchanges, None)?;
    let registry = registry_of(&input.rows)?;
    let told = Told::new(
        input
            .on_refreshed
            .map(|held| held.as_str() == "readsAndThrows"),
    );
    let catalog = Arc::new(ModelCatalog::new(ModelCatalogDeps {
        transport: Arc::clone(&net.transport),
        deps: CatalogDeps {
            runtime: nyaaai_testkit::cases::runtime(),
            scheduler: net.scheduler.clone(),
            budgets: None,
            rows: registry.clone() as Arc<dyn ModelRegistry>,
            refreshed: told.callback(),
        },
        ttl_ms: input.ttl_ms,
    }));
    told.reading({
        let weak = Arc::downgrade(&catalog);
        move || {
            let catalog = weak.upgrade()?;
            let read = catalog.listing(None, false);
            Some(Box::pin(async move {
                match read.await {
                    Ok(Some(held)) => held.raw().len().to_string(),
                    _ => "none".to_owned(),
                }
            }) as BoxFuture<'static, String>)
        }
    });
    let steps = steps_openrouter(&input.steps);
    let catalog_ref = &catalog;
    let outcomes =
        block_on(play(
            &net,
            &registry,
            steps,
            || Box::pin(catalog_ref.idle()),
            move |call, token| {
                Box::pin(async move {
                    Ok(match &call {
                        Call::Views { use_, zdr_only } => {
                            match catalog_ref
                                .views(use_of(use_.clone()), zdr_only.unwrap_or(true), Some(&token))
                                .await
                            {
                                Ok(views) => {
                                    answer_value("views", vec![("views", views_value(&views)?)])
                                }
                                Err(_) => thrown(CALLER_REASON),
                            }
                        }
                        Call::View {
                            key,
                            use_,
                            zdr_only,
                        } => match catalog_ref
                            .view(
                                key,
                                use_of(use_.clone()),
                                zdr_only.unwrap_or(true),
                                Some(&token),
                            )
                            .await
                        {
                            Ok(view) => answer_value(
                                "view",
                                vec![(
                                    "view",
                                    match view {
                                        None => Value::Null,
                                        Some(held) => view_line(&held)?.encode(),
                                    },
                                )],
                            ),
                            Err(_) => thrown(CALLER_REASON),
                        },
                        Call::ViewFor { row, use_ } => {
                            let row = row_of_key(row)?;
                            match catalog_ref
                                .view_for(&row, use_of(use_.clone()), Some(&token))
                                .await
                            {
                                Ok(view) => {
                                    answer_value("view", vec![("view", view_line(&view)?.encode())])
                                }
                                Err(_) => thrown(CALLER_REASON),
                            }
                        }
                        Call::Entry { id } => match catalog_ref.entry(id, Some(&token)).await {
                            Ok(entry) => answer_value(
                                "entry",
                                vec![(
                                    "entry",
                                    match entry {
                                        None => Value::Null,
                                        Some(held) => object_value(vec![
                                            ("id", Value::from(held.id.as_str())),
                                            ("name", text_value(&held.name)),
                                            (
                                                "efforts",
                                                texts_value(&held.reasoning.and_then(
                                                    |reasoning| reasoning.supported_efforts,
                                                )),
                                            ),
                                        ]),
                                    },
                                )],
                            ),
                            Err(_) => thrown(CALLER_REASON),
                        },
                        Call::Listing { retry } => {
                            match catalog_ref
                                .listing(Some(&token), retry.unwrap_or(false))
                                .await
                            {
                                Ok(listing) => answer_value(
                                    "listing",
                                    vec![(
                                        "listing",
                                        match listing {
                                            None => Value::Null,
                                            Some(held) => object_value(vec![
                                                ("raw", Value::from(held.raw().len())),
                                                (
                                                    "endpoints",
                                                    Value::Array(
                                                        held.endpoints()
                                                            .iter()
                                                            .map(|(key, _)| Value::from(key))
                                                            .collect(),
                                                    ),
                                                ),
                                            ]),
                                        },
                                    )],
                                ),
                                Err(_) => thrown(CALLER_REASON),
                            }
                        }
                        Call::Models { .. } => thrown("a catalog has no models call"),
                    })
                })
            },
        ))??;
    catalog.dispose();
    let count = told.count();
    let saw = block_on(told.seen())?;
    Ok(CatalogOpenrouterRunResult {
        steps: outcomes
            .into_iter()
            .map(|(now_ms, answers, row_reads)| {
                Ok(CatalogOpenrouterRunResultStepsItem {
                    now_ms,
                    answers: answers.iter().map(decoded).collect::<Outcome<Vec<_>>>()?,
                    row_reads,
                })
            })
            .collect::<Outcome<Vec<_>>>()?,
        requests: net.asked(),
        read: net
            .played
            .read()
            .into_iter()
            .map(|read| read as i64)
            .collect(),
        waiting: net.wheel.waiting(),
        refreshed: count,
        saw,
    })
}

pub fn opencode_run(input: CatalogOpencodeRunInput) -> Outcome<CatalogOpencodeRunResult> {
    let net = network_of(input.start_ms, &input.exchanges, None)?;
    let registry = registry_of(&input.rows)?;
    let told = Told::new(
        input
            .on_refreshed
            .map(|held| held.as_str() == "readsAndThrows"),
    );
    let catalog = Arc::new(OpenCodeCatalog::new(OpenCodeCatalogDeps {
        plan: input.plan,
        transport: Arc::clone(&net.transport),
        deps: CatalogDeps {
            runtime: nyaaai_testkit::cases::runtime(),
            scheduler: net.scheduler.clone(),
            budgets: None,
            rows: registry.clone() as Arc<dyn ModelRegistry>,
            refreshed: told.callback(),
        },
        ttl_ms: input.ttl_ms,
    }));
    told.reading({
        let weak = Arc::downgrade(&catalog);
        move || {
            let catalog = weak.upgrade()?;
            let read = catalog.models(false, None);
            Some(Box::pin(async move {
                match read.await {
                    Ok(Some(held)) => held
                        .iter()
                        .map(|(key, _)| key.to_owned())
                        .collect::<Vec<_>>()
                        .join(" "),
                    _ => "none".to_owned(),
                }
            }) as BoxFuture<'static, String>)
        }
    });
    let steps = steps_opencode(&input.steps);
    let catalog_ref = &catalog;
    let outcomes = block_on(play(
        &net,
        &registry,
        steps,
        || Box::pin(catalog_ref.idle()),
        move |call, token| {
            Box::pin(async move {
                Ok(match &call {
                    Call::Views { use_, zdr_only } => {
                        match catalog_ref
                            .views(use_of(use_.clone()), zdr_only.unwrap_or(true), Some(&token))
                            .await
                        {
                            Ok(views) => {
                                answer_value("views", vec![("views", views_value(&views)?)])
                            }
                            Err(_) => thrown(CALLER_REASON),
                        }
                    }
                    Call::ViewFor { row, use_ } => {
                        let row = row_of_key(row)?;
                        match catalog_ref
                            .view_for(&row, use_of(use_.clone()), Some(&token))
                            .await
                        {
                            Ok(view) => {
                                answer_value("view", vec![("view", view_line(&view)?.encode())])
                            }
                            Err(_) => thrown(CALLER_REASON),
                        }
                    }
                    Call::Models { fresh } => {
                        match catalog_ref
                            .models(fresh.unwrap_or(false), Some(&token))
                            .await
                        {
                            Ok(models) => answer_value(
                                "models",
                                vec![(
                                    "models",
                                    match models {
                                        None => Value::Null,
                                        Some(held) => Value::Array(
                                            held.iter().map(|(key, _)| Value::from(key)).collect(),
                                        ),
                                    },
                                )],
                            ),
                            Err(_) => thrown(CALLER_REASON),
                        }
                    }
                    _ => thrown("an OpenCode catalog has no such call"),
                })
            })
        },
    ))??;
    let count = told.count();
    let saw = block_on(told.seen())?;
    Ok(CatalogOpencodeRunResult {
        steps: outcomes
            .into_iter()
            .map(|(now_ms, answers, row_reads)| {
                Ok(CatalogOpencodeRunResultStepsItem {
                    now_ms,
                    answers: answers.iter().map(decoded).collect::<Outcome<Vec<_>>>()?,
                    row_reads,
                })
            })
            .collect::<Outcome<Vec<_>>>()?,
        requests: net.asked(),
        read: net
            .played
            .read()
            .into_iter()
            .map(|read| read as i64)
            .collect(),
        waiting: net.wheel.waiting(),
        refreshed: count,
        saw,
    })
}
