//! The OpenCode plan catalog: decoder, join, browser and tester.

use std::future::Future;
use std::sync::Arc;

use nyaaai_contract::cancel::CancelToken;
use nyaaai_contract::fetch::{BudgetName, Budgets, span};
use nyaaai_contract::future::BoxFuture;
use nyaaai_contract::json::Value;
use nyaaai_contract::model_registry::ModelRegistry;
use nyaaai_contract::shapes::model::{
    BrowseDetail, BrowseDetailCache, BrowseDetailRetention, BrowseModel, BrowseModelPrice,
    BrowseModelPriceObject, BrowseModelTools, ModelRow, ModelRowCache, ModelUse, ModelView,
    ModelViewCache, ModelViewPrice, ModelViewStatus, Price, PriceTier, RowIdentity,
    RowIdentityHosted,
};
use nyaaai_contract::shapes::provider::{ProviderKindName, ProviderPlan};
use nyaaai_contract::shapes::step::{ApiFacts, ApiFamily};
use nyaaai_contract::time::Scheduler;

use super::retention::{is_zdr, retention_of};
use crate::effort::EFFORTS;
use crate::kinds::fetcher::{Aborted, Fetcher, Refreshed, TTL_MS, Waiter};
use crate::kinds::openrouter::catalog::Ordered;
use crate::kinds::readers::{
    boolean_field, data_list, field_of, fields_of, number_field, object_field, text_field,
    texts_field,
};
use crate::models::curated::MODELS;
use crate::models::row_key::{row_key, shown_id};
use crate::models::view::unknown_view;
use crate::provider::kind::{Browser, EntryCatalog, KindError, Listed, Tester};
use crate::transport::{KeyAuth, Transport};

/// One API a model is served on. A missing, null or mistyped field is none.
#[derive(Debug, Clone, PartialEq)]
pub struct OpenCodeApi {
    pub family: ApiFamily,
    /// The thought text field.
    pub reasoning_field: Option<String>,
    /// Tool choice modes.
    pub choice: Option<Vec<String>>,
    pub strict: Option<bool>,
}

/// `explicit` is whether the listing has one that is not null.
#[derive(Debug, Clone, PartialEq)]
pub struct OpenCodeCache {
    pub implicit: Option<bool>,
    pub explicit: bool,
}

/// One model of a plan's list; a bad field is none.
#[derive(Debug, Clone, PartialEq)]
pub struct OpenCodeModel {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub input: Option<Vec<String>>,
    pub output: Option<Vec<String>>,
    pub tools: Option<bool>,
    pub reasoning: Option<bool>,
    pub efforts: Option<Vec<String>>,
    pub toggle: Option<bool>,
    pub structured_output: Option<bool>,
    pub cache: Option<OpenCodeCache>,
    pub context: Option<f64>,
    pub input_limit: Option<f64>,
    pub output_limit: Option<f64>,
    /// Base text prices, USD per million tokens.
    pub price: Option<(f64, f64)>,
    /// The listed families, in the order of the four.
    pub api: Vec<OpenCodeApi>,
}

const FAMILIES: [ApiFamily; 4] = [
    ApiFamily::ChatCompletions,
    ApiFamily::Responses,
    ApiFamily::Messages,
    ApiFamily::GenerateContent,
];

/// Unbound model suffixes.
const UNBOUND: [&str; 2] = ["-free", "-latest"];

type Fields = nyaaai_contract::json::Object;

fn into<'a>(fields: Option<&'a Fields>, key: &str) -> Option<&'a Fields> {
    fields.and_then(|held| object_field(held, key))
}

fn api_of(value: Option<&Value>) -> Vec<OpenCodeApi> {
    let Some(record) = value.and_then(fields_of) else {
        return Vec::new();
    };
    FAMILIES
        .iter()
        .filter_map(|family| {
            let held = field_of(record, family.as_str())?;
            let held = fields_of(held);
            let tools = into(into(held, "capabilities"), "tools");
            let compatibility = into(held, "compatibility");
            Some(OpenCodeApi {
                family: *family,
                reasoning_field: compatibility
                    .and_then(|fields| text_field(fields, "reasoning_field"))
                    .map(str::to_owned),
                choice: tools.and_then(|fields| texts_field(fields, "choice")),
                strict: tools.and_then(|fields| boolean_field(fields, "strict")),
            })
        })
        .collect()
}

/// Both prices are numbers, or the cost is unknown.
fn price_of(cost: Option<&Fields>) -> Option<(f64, f64)> {
    let text = into(cost, "text");
    let input = into(text, "input").and_then(|held| number_field(held, "price"))?;
    let output = into(text, "output").and_then(|held| number_field(held, "price"))?;
    Some((input, output))
}

fn cache_field(capabilities: Option<&Fields>) -> Option<OpenCodeCache> {
    let cache = into(capabilities, "cache")?;
    let explicit = field_of(cache, "explicit");
    Some(OpenCodeCache {
        implicit: boolean_field(cache, "implicit"),
        explicit: explicit
            .is_some_and(|held| matches!(held, Value::Bool(true)) || fields_of(held).is_some()),
    })
}

/// An item with a text id; none drops the item.
pub fn parse_open_code_model(item: &Value) -> Option<OpenCodeModel> {
    let fields = fields_of(item)?;
    let id = text_field(fields, "id")?.to_owned();
    let modalities = object_field(fields, "modalities");
    let capabilities = object_field(fields, "capabilities");
    let tools = into(capabilities, "tools");
    let reasoning = into(capabilities, "reasoning");
    let limit = object_field(fields, "limit");
    let owned = |held: Option<&str>| held.map(str::to_owned);
    Some(OpenCodeModel {
        id,
        name: owned(text_field(fields, "name")),
        description: owned(text_field(fields, "description")),
        input: modalities.and_then(|held| texts_field(held, "input")),
        output: modalities.and_then(|held| texts_field(held, "output")),
        tools: tools.and_then(|held| boolean_field(held, "supported")),
        reasoning: reasoning.and_then(|held| boolean_field(held, "supported")),
        efforts: reasoning.and_then(|held| texts_field(held, "effort")),
        toggle: reasoning.and_then(|held| boolean_field(held, "toggle")),
        structured_output: capabilities.and_then(|held| boolean_field(held, "structured_output")),
        cache: cache_field(capabilities),
        context: limit.and_then(|held| number_field(held, "context")),
        input_limit: limit.and_then(|held| number_field(held, "input")),
        output_limit: limit.and_then(|held| number_field(held, "output")),
        price: price_of(object_field(fields, "cost")),
        api: api_of(field_of(fields, "api")),
    })
}

/// None without a list; a repeated id replaces in place.
pub fn parse_open_code(raw: &Value) -> Option<Ordered<OpenCodeModel>> {
    let mut models = Ordered::new();
    for item in data_list(raw)? {
        if let Some(model) = parse_open_code_model(item) {
            models.set(&model.id.clone(), model);
        }
    }
    Some(models)
}

fn family_of(model: &OpenCodeModel) -> Option<ApiFamily> {
    FAMILIES
        .iter()
        .copied()
        .find(|family| model.api.iter().any(|api| api.family == *family))
}

/// API-specific caching defaults.
fn cache_of(model: &OpenCodeModel, family: ApiFamily) -> ModelRowCache {
    match &model.cache {
        None => {
            if family == ApiFamily::Messages {
                ModelRowCache::Explicit
            } else {
                ModelRowCache::Implicit
            }
        }
        Some(cache) => {
            if cache.implicit != Some(true) && cache.explicit {
                ModelRowCache::Explicit
            } else {
                ModelRowCache::Implicit
            }
        }
    }
}

fn facts_of(model: &OpenCodeModel, family: ApiFamily) -> ApiFacts {
    let api = model.api.iter().find(|each| each.family == family);
    ApiFacts {
        family,
        structured: model.structured_output == Some(true),
        // Missing choice means `auto`.
        choice: api
            .and_then(|held| held.choice.clone())
            .unwrap_or_else(|| vec!["auto".to_owned()]),
        strict: api.is_some_and(|held| held.strict == Some(true)),
        reasoning: api.and_then(|held| held.reasoning_field.clone()),
        toggle: (model.toggle == Some(true)).then_some(true),
    }
}

/// Base prices only.
fn view_of(
    row: &ModelRow,
    model: &OpenCodeModel,
    plan: ProviderPlan,
    zdr_only: bool,
    now: i64,
) -> Option<ModelView> {
    let family = family_of(model)?;
    if model.tools != Some(true) {
        return None;
    }
    let offered: &[String] = model.efforts.as_deref().unwrap_or(&[]);
    let efforts = if row.reasoning {
        EFFORTS
            .iter()
            .copied()
            .filter(|effort| offered.iter().any(|held| held == effort.as_str()))
            .collect()
    } else {
        Vec::new()
    };
    let zdr = is_zdr(plan, &model.id, now);
    let mut view = unknown_view(row);
    view.efforts = efforts;
    if zdr_only && !zdr {
        view.status = ModelViewStatus::NoZdrHost;
        return Some(view);
    }
    view.status = ModelViewStatus::Ok;
    view.prompt_limit = model.input_limit.or(model.context);
    view.output_limit = model.output_limit;
    if let Some((input, output)) = model.price {
        view.price = Some(ModelViewPrice::Price(Price {
            prompt: (input, input),
            completion: (output, output),
            tiers: vec![PriceTier {
                min_prompt_tokens: 0.0,
                prompt: input,
                completion: output,
                cache_read: None,
                cache_write: None,
                cache_write1h: None,
            }],
        }));
    }
    if !zdr {
        view.data_kept = Some(());
    }
    view.api = Some(facts_of(model, family));
    view.cache = Some(match cache_of(model, family) {
        ModelRowCache::Explicit => ModelViewCache::Explicit,
        ModelRowCache::Implicit => ModelViewCache::Implicit,
    });
    Some(view)
}

fn is_bound(row: &ModelRow) -> bool {
    row.bindings.get("opencode").is_some()
}

/// Retention survives catalog failure.
fn unlisted(row: &ModelRow, plan: ProviderPlan, zdr_only: bool, now: i64) -> ModelView {
    let id = row.bindings.get("opencode").map_or("", |held| &held.id);
    let mut view = unknown_view(row);
    if is_zdr(plan, id, now) {
        return view;
    }
    if zdr_only {
        view.status = ModelViewStatus::NoZdrHost;
    } else {
        view.data_kept = Some(());
    }
    view
}

fn media_of(types: Option<&[String]>) -> Vec<String> {
    let mut seen: Vec<String> = Vec::new();
    for kind in types.unwrap_or(&[]) {
        let held = kind.split('/').next().unwrap_or(kind).to_owned();
        if !seen.contains(&held) {
            seen.push(held);
        }
    }
    seen
}

fn capabilities_of(model: &OpenCodeModel) -> Vec<String> {
    let mut held = Vec::new();
    if model.tools == Some(true) {
        held.push("tools".to_owned());
    }
    if model.reasoning == Some(true) {
        held.push("reasoning".to_owned());
    }
    if model.structured_output == Some(true) {
        held.push("structured_output".to_owned());
    }
    held
}

fn browse_of(model: &OpenCodeModel, plan: ProviderPlan, now: i64) -> BrowseModel {
    let family = family_of(model);
    let zdr = is_zdr(plan, &model.id, now);
    let identity = RowIdentity::Hosted(RowIdentityHosted {
        provider: ProviderKindName::Opencode,
        id: model.id.clone(),
    });
    BrowseModel {
        id: row_key(&identity),
        display: shown_id(&identity),
        label: model.name.clone().unwrap_or_else(|| model.id.clone()),
        curated: MODELS.iter().any(|row| {
            row.bindings
                .get("opencode")
                .is_some_and(|binding| binding.id == model.id)
        }),
        description: model.description.clone().unwrap_or_default(),
        context: model.context.or(model.input_limit),
        inputs: media_of(model.input.as_deref()),
        outputs: media_of(model.output.as_deref()),
        parameters: capabilities_of(model),
        price: match model.price {
            None => BrowseModelPrice::Unknown,
            Some((input, output)) => BrowseModelPrice::Object(BrowseModelPriceObject {
                prompt: (input, input),
                completion: (output, output),
            }),
        },
        tools: if !zdr {
            BrowseModelTools::NoZdrHost
        } else if family.is_some() && model.tools == Some(true) {
            BrowseModelTools::Ok
        } else {
            BrowseModelTools::None
        },
        retestable: None,
        family,
    }
}

fn detail_of(model: &OpenCodeModel, plan: ProviderPlan, now: i64) -> BrowseDetail {
    let family = family_of(model);
    let until = retention_of(plan, &model.id).and_then(|held| held.until);
    BrowseDetail {
        hosts: None,
        max_output: model.output_limit,
        efforts: Some(model.efforts.clone().unwrap_or_default()),
        choice: family.map(|held| facts_of(model, held).choice),
        cache: family.map(|held| match cache_of(model, held) {
            ModelRowCache::Explicit => BrowseDetailCache::Explicit,
            ModelRowCache::Implicit => BrowseDetailCache::Implicit,
        }),
        retention: Some(BrowseDetailRetention {
            zdr: is_zdr(plan, &model.id, now),
            until: until.map(str::to_owned),
        }),
    }
}

/// No per-model fetches.
pub fn join_open_code(
    rows: &[ModelRow],
    models: Option<&Ordered<OpenCodeModel>>,
    plan: ProviderPlan,
    _use: &ModelUse,
    zdr_only: bool,
    now: i64,
) -> Vec<ModelView> {
    let bound = rows.iter().filter(|row| is_bound(row));
    let Some(models) = models else {
        return bound
            .map(|row| unlisted(row, plan, zdr_only, now))
            .collect();
    };
    bound
        .filter_map(|row| {
            let id = row.bindings.get("opencode").map_or("", |held| &held.id);
            view_of(row, models.get(id)?, plan, zdr_only, now)
        })
        .collect()
}

struct Shared {
    plan: ProviderPlan,
    transport: Arc<Transport>,
    scheduler: Arc<dyn Scheduler>,
    /// A refresh's span opens from `catalog`.
    budgets: Budgets,
    rows: Arc<dyn ModelRegistry>,
    refreshed: Arc<dyn Fn() + Send + Sync>,
    ttl_ms: Option<i64>,
    fetcher: Fetcher<Arc<Ordered<OpenCodeModel>>>,
}

pub struct OpenCodeCatalogDeps {
    pub plan: ProviderPlan,
    pub transport: Arc<Transport>,
    pub deps: crate::provider::kind::CatalogDeps,
    pub ttl_ms: Option<i64>,
}

/// One entry's plan catalog; keeps the last list on errors.
pub struct OpenCodeCatalog {
    shared: Arc<Shared>,
}

impl OpenCodeCatalog {
    pub fn new(deps: OpenCodeCatalogDeps) -> Self {
        Self {
            shared: Arc::new(Shared {
                plan: deps.plan,
                transport: deps.transport,
                scheduler: deps.deps.scheduler,
                budgets: deps.deps.budgets.unwrap_or_default(),
                rows: deps.deps.rows,
                refreshed: deps.deps.refreshed,
                ttl_ms: deps.ttl_ms,
                fetcher: Fetcher::new(deps.deps.runtime.clone()),
            }),
        }
    }

    /// Answers when no refresh is running, having waited for the one that was.
    pub fn idle(&self) -> impl Future<Output = ()> + Send + use<> {
        self.shared.fetcher.idle()
    }

    /// Fresh retries failures. A refresh it needs starts at the call.
    pub fn models(
        &self,
        fresh: bool,
        cancel: Option<&CancelToken>,
    ) -> impl Future<Output = Result<Option<Arc<Ordered<OpenCodeModel>>>, Aborted>> + Send + use<>
    {
        if fresh {
            self.shared.fetcher.retry();
        }
        let waiter = fetched(&self.shared);
        let cancel = cancel.cloned();
        async move { waiter.raced(cancel.as_ref()).await }
    }

    /// Tester ZDR view.
    pub fn view_for(
        &self,
        row: &ModelRow,
        _use: ModelUse,
        cancel: Option<&CancelToken>,
    ) -> impl Future<Output = Result<ModelView, Aborted>> + Send + use<> {
        let waiter = fetched(&self.shared);
        let shared = Arc::clone(&self.shared);
        let cancel = cancel.cloned();
        let row = row.clone();
        async move {
            let models = waiter.raced(cancel.as_ref()).await?;
            let id = row.bindings.get("opencode").map(|held| held.id.as_str());
            let (Some(models), Some(id)) = (models, id) else {
                return Ok(unknown_view(&row));
            };
            let Some(model) = models.get(id) else {
                let mut view = unknown_view(&row);
                view.status = ModelViewStatus::Retired;
                return Ok(view);
            };
            let now = shared.scheduler.now_ms();
            Ok(
                view_of(&row, model, shared.plan, true, now).unwrap_or_else(|| {
                    let mut view = unknown_view(&row);
                    view.status = ModelViewStatus::NoZdrHost;
                    view
                }),
            )
        }
    }
}

fn fetched(shared: &Arc<Shared>) -> Waiter<Arc<Ordered<OpenCodeModel>>> {
    let held = Arc::clone(shared);
    let now = shared.scheduler.now_ms();
    let ttl = shared.ttl_ms.unwrap_or(TTL_MS);
    shared.fetcher.fetched(now, ttl, move || {
        let limit = span(&held.scheduler, held.budgets.get(BudgetName::Catalog), None);
        Box::pin(async move {
            let read = held
                .transport
                .get_json("/v1/models", &limit, &KeyAuth::None)
                .await;
            limit.clear();
            match read.ok().as_ref().and_then(parse_open_code) {
                Some(models) => {
                    let at = held.scheduler.now_ms();
                    let told = Arc::clone(&held);
                    Refreshed::Good {
                        value: Arc::new(models),
                        at,
                        told: Some(Box::new(move || (told.refreshed)())),
                    }
                }
                None => Refreshed::Failed {
                    at: held.scheduler.now_ms(),
                },
            }
        })
    })
}

impl EntryCatalog for OpenCodeCatalog {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn arc_any(self: Arc<Self>) -> Arc<dyn std::any::Any + Send + Sync> {
        self
    }

    fn views(
        &self,
        use_: ModelUse,
        zdr_only: bool,
        cancel: Option<&CancelToken>,
    ) -> BoxFuture<'_, Result<Vec<ModelView>, Aborted>> {
        let waiter = fetched(&self.shared);
        let cancel = cancel.cloned();
        Box::pin(async move {
            let models = waiter.raced(cancel.as_ref()).await?;
            let rows = self.shared.rows.rows();
            Ok(join_open_code(
                &rows,
                models.as_deref(),
                self.shared.plan,
                &use_,
                zdr_only,
                self.shared.scheduler.now_ms(),
            ))
        })
    }

    fn idle(&self) -> BoxFuture<'_, ()> {
        Box::pin(OpenCodeCatalog::idle(self))
    }
}

/// The browser of a plan's list.
pub struct OpenCodeBrowser {
    catalog: Arc<OpenCodeCatalog>,
    plan: ProviderPlan,
    clock: Arc<dyn Scheduler>,
}

pub fn opencode_browser(
    catalog: Arc<OpenCodeCatalog>,
    plan: ProviderPlan,
    clock: Arc<dyn Scheduler>,
) -> OpenCodeBrowser {
    OpenCodeBrowser {
        catalog,
        plan,
        clock,
    }
}

impl Browser for OpenCodeBrowser {
    fn list(&self, fresh: bool) -> BoxFuture<'_, Result<Option<Vec<BrowseModel>>, KindError>> {
        let models = self.catalog.models(fresh, None);
        Box::pin(async move {
            let Some(models) = models.await? else {
                return Ok(None);
            };
            let now = self.clock.now_ms();
            Ok(Some(
                models
                    .iter()
                    .filter(|(_, model)| !UNBOUND.iter().any(|suffix| model.id.ends_with(suffix)))
                    .map(|(_, model)| browse_of(model, self.plan, now))
                    .collect(),
            ))
        })
    }

    fn detail<'a>(
        &'a self,
        id: &'a str,
        _cancel: &'a CancelToken,
    ) -> BoxFuture<'a, Result<Option<BrowseDetail>, KindError>> {
        let models = self.catalog.models(false, None);
        Box::pin(async move {
            let models = models.await?;
            Ok(models
                .and_then(|held| held.get(id).cloned())
                .map(|model| detail_of(&model, self.plan, self.clock.now_ms())))
        })
    }
}

/// The tester of a plan's list.
pub struct OpenCodeTester {
    catalog: Arc<OpenCodeCatalog>,
}

pub fn opencode_tester(catalog: Arc<OpenCodeCatalog>) -> OpenCodeTester {
    OpenCodeTester { catalog }
}

impl Tester for OpenCodeTester {
    fn listed<'a>(
        &'a self,
        id: &'a str,
        cancel: &'a CancelToken,
    ) -> BoxFuture<'a, Result<Option<Listed>, KindError>> {
        let models = self.catalog.models(false, Some(cancel));
        Box::pin(async move {
            let models = models.await?;
            Ok(models
                .and_then(|held| held.get(id).cloned())
                .map(|model| Listed {
                    reasoning: !model.efforts.clone().unwrap_or_default().is_empty(),
                    label: model.name.clone().unwrap_or_else(|| model.id.clone()),
                }))
        })
    }

    fn view<'a>(
        &'a self,
        row: &'a ModelRow,
        use_: ModelUse,
        cancel: &'a CancelToken,
    ) -> BoxFuture<'a, Result<ModelView, KindError>> {
        let view = self.catalog.view_for(row, use_, Some(cancel));
        Box::pin(async move { Ok(view.await?) })
    }
}
