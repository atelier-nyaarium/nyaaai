//! The OpenRouter catalog: decoders, the join to rows and the cached fetch.

use std::collections::HashMap;
use std::future::Future;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use nyaaai_contract::cancel::{CancelToken, Unsubscribe};
use nyaaai_contract::json::{Object, Value};
use nyaaai_contract::number::{number_of_text, round_half_up};
use nyaaai_contract::shapes::model::{
    ModelRow, ModelUse, ModelView, ModelViewPrice, ModelViewStatus, Price, PriceTier,
};
use nyaaai_contract::text::text_length;
use nyaaai_contract::time::{deadline, parse_iso_time};

use crate::effort::EFFORTS;
use crate::kinds::fetcher::{Aborted, FETCH_MS, Fetcher, Refreshed, TTL_MS};
use crate::kinds::readers::{
    data_list, field_of, fields_of, number_field, object_field, text_field, texts_field,
};
use crate::models::view::unknown_view;
use crate::provider::kind::CatalogDeps;
use crate::transport::{KeyAuth, Transport};

/// Any listed parameter suffices.
#[derive(Debug, Clone, PartialEq)]
pub enum ParamNeed {
    One(String),
    Any(Vec<String>),
}

/// A price override above a prompt size.
#[derive(Debug, Clone, PartialEq)]
pub struct PriceOverride {
    pub min_prompt_tokens: f64,
    pub prompt: Option<Value>,
    pub completion: Option<Value>,
    pub input_cache_read: Option<Value>,
    pub input_cache_write: Option<Value>,
    pub input_cache_write_1h: Option<Value>,
}

/// Prices as listed; a price is a number or its text.
#[derive(Debug, Clone, PartialEq)]
pub struct Pricing {
    pub prompt: Option<Value>,
    pub completion: Option<Value>,
    pub input_cache_read: Option<Value>,
    pub input_cache_write: Option<Value>,
    pub input_cache_write_1h: Option<Value>,
    pub overrides: Option<Vec<PriceOverride>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReasoningEntry {
    pub supported_efforts: Option<Vec<String>>,
}

/// A missing, null or mistyped field is none.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelEntry {
    pub id: String,
    pub name: Option<String>,
    pub created: Option<f64>,
    pub context_length: Option<f64>,
    pub expiration_date: Option<String>,
    pub pricing: Option<Pricing>,
    pub reasoning: Option<ReasoningEntry>,
    pub supported_parameters: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Endpoint {
    pub model_id: String,
    pub tag: Option<String>,
    pub provider_name: Option<String>,
    pub context_length: Option<f64>,
    pub max_prompt_tokens: Option<f64>,
    pub supported_parameters: Option<Vec<String>>,
    pub pricing: Option<Pricing>,
}

/// Values by text key in the order the keys first came; a later one replaces in place.
#[derive(Debug, Clone, PartialEq)]
pub struct Ordered<T> {
    items: Vec<(String, T)>,
    index: HashMap<String, usize>,
}

impl<T> Default for Ordered<T> {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            index: HashMap::new(),
        }
    }
}

impl<T> Ordered<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, key: &str, value: T) {
        match self.index.get(key) {
            Some(at) => self.items[*at].1 = value,
            None => {
                self.index.insert(key.to_owned(), self.items.len());
                self.items.push((key.to_owned(), value));
            }
        }
    }

    pub fn get(&self, key: &str) -> Option<&T> {
        self.index.get(key).map(|at| &self.items[*at].1)
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut T> {
        let at = *self.index.get(key)?;
        Some(&mut self.items[at].1)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &T)> {
        self.items.iter().map(|(key, value)| (key.as_str(), value))
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

#[derive(Debug)]
pub struct Fetched {
    pub entries: Ordered<ModelEntry>,
    pub endpoints: Ordered<Vec<Endpoint>>,
    /// Raw `/models` items for browsing.
    pub raw: Vec<Value>,
}

/// One fetch's `/models` items and zero-retention endpoints.
#[derive(Debug, Clone)]
pub struct CatalogListing {
    pub fetched: Arc<Fetched>,
}

impl CatalogListing {
    pub fn raw(&self) -> &[Value] {
        &self.fetched.raw
    }

    pub fn endpoints(&self) -> &Ordered<Vec<Endpoint>> {
        &self.fetched.endpoints
    }
}

pub struct ModelCatalogDeps {
    pub transport: Arc<Transport>,
    pub deps: CatalogDeps,
    pub ttl_ms: Option<i64>,
}

/// What `chat_standing` says.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Standing {
    Ok,
    Retired,
    NoZdrHost,
}

const STRUCTURED_PARAMS: [&str; 2] = ["response_format", "structured_outputs"];
/// Characters of a date alone.
const DATE_CHARS: usize = 10;

fn text_of(fields: &Object, key: &str) -> Option<String> {
    text_field(fields, key).map(str::to_owned)
}

fn loose(fields: &Object, key: &str) -> Option<Value> {
    field_of(fields, key).cloned()
}

fn override_of(value: &Value) -> Option<PriceOverride> {
    let fields = fields_of(value)?;
    Some(PriceOverride {
        min_prompt_tokens: number_field(fields, "min_prompt_tokens")?,
        prompt: loose(fields, "prompt"),
        completion: loose(fields, "completion"),
        input_cache_read: loose(fields, "input_cache_read"),
        input_cache_write: loose(fields, "input_cache_write"),
        input_cache_write_1h: loose(fields, "input_cache_write_1h"),
    })
}

/// One bad override drops the list.
fn overrides_of(value: Option<&Value>) -> Option<Vec<PriceOverride>> {
    value?.as_array()?.iter().map(override_of).collect()
}

fn pricing_of(fields: &Object, key: &str) -> Option<Pricing> {
    let held = object_field(fields, key)?;
    Some(Pricing {
        prompt: loose(held, "prompt"),
        completion: loose(held, "completion"),
        input_cache_read: loose(held, "input_cache_read"),
        input_cache_write: loose(held, "input_cache_write"),
        input_cache_write_1h: loose(held, "input_cache_write_1h"),
        overrides: overrides_of(field_of(held, "overrides")),
    })
}

/// USD per million tokens, rounded to six decimals.
pub fn per_million(value: Option<&Value>) -> Option<f64> {
    let price = match value? {
        Value::String(text) => number_of_text(text)?,
        Value::Number(number) => *number,
        _ => return None,
    };
    (price.is_finite() && price >= 0.0).then(|| round_half_up(price * 1e12) / 1e6 + 0.0)
}

/// A removal date: a day, or an ISO time; none when neither.
pub fn expiry_of(text: &str) -> Option<i64> {
    if text_length(text) == DATE_CHARS {
        parse_iso_time(&format!("{text}T00:00:00Z"))
    } else {
        parse_iso_time(text)
    }
}

pub fn range_of(values: &[Option<f64>]) -> Option<(f64, f64)> {
    let known: Vec<f64> = values.iter().flatten().copied().collect();
    if known.is_empty() {
        return None;
    }
    let low = known.iter().copied().fold(f64::INFINITY, f64::min);
    let high = known.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    Some((low, high))
}

/// A tier; a price that is not listed is left out.
fn tier(
    min_prompt_tokens: f64,
    prompt: f64,
    completion: f64,
    cache: (Option<f64>, Option<f64>, Option<f64>),
) -> PriceTier {
    PriceTier {
        min_prompt_tokens,
        prompt,
        completion,
        cache_read: cache.0,
        cache_write: cache.1,
        cache_write1h: cache.2,
    }
}

fn tiers_of(pricing: Option<&Pricing>) -> Vec<PriceTier> {
    let mut tiers = Vec::new();
    let price = |value: Option<&Option<Value>>| per_million(value.and_then(Option::as_ref));
    let (Some(prompt), Some(completion)) = (
        price(pricing.map(|held| &held.prompt)),
        price(pricing.map(|held| &held.completion)),
    ) else {
        return tiers;
    };
    tiers.push(tier(
        0.0,
        prompt,
        completion,
        (
            price(pricing.map(|held| &held.input_cache_read)),
            price(pricing.map(|held| &held.input_cache_write)),
            price(pricing.map(|held| &held.input_cache_write_1h)),
        ),
    ));
    let mut overrides: Vec<&PriceOverride> = pricing
        .and_then(|held| held.overrides.as_ref())
        .map(|held| held.iter().collect())
        .unwrap_or_default();
    overrides.sort_by(|left, right| {
        left.min_prompt_tokens
            .partial_cmp(&right.min_prompt_tokens)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    for held in overrides {
        let own = |value: &Option<Value>| per_million(value.as_ref());
        tiers.push(tier(
            held.min_prompt_tokens,
            own(&held.prompt).unwrap_or(prompt),
            own(&held.completion).unwrap_or(completion),
            (
                own(&held.input_cache_read),
                own(&held.input_cache_write)
                    .or_else(|| price(pricing.map(|base| &base.input_cache_write))),
                own(&held.input_cache_write_1h)
                    .or_else(|| price(pricing.map(|base| &base.input_cache_write_1h))),
            ),
        ));
    }
    tiers
}

pub fn sent_params(row: &ModelRow, use_: ModelUse) -> Vec<ParamNeed> {
    let mut params = vec![ParamNeed::One("tools".to_owned())];
    if row.reasoning {
        params.push(ParamNeed::One("reasoning".to_owned()));
    }
    if use_ != ModelUse::Chat {
        params.extend(
            STRUCTURED_PARAMS
                .iter()
                .map(|name| ParamNeed::One((*name).to_owned())),
        );
    }
    params
}

/// An item with a text id; none drops the item.
pub fn parse_model(item: &Value) -> Option<ModelEntry> {
    let fields = fields_of(item)?;
    let id = text_field(fields, "id")?.to_owned();
    let reasoning = object_field(fields, "reasoning");
    Some(ModelEntry {
        id,
        name: text_of(fields, "name"),
        created: number_field(fields, "created"),
        context_length: number_field(fields, "context_length"),
        expiration_date: text_of(fields, "expiration_date"),
        pricing: pricing_of(fields, "pricing"),
        reasoning: reasoning.map(|held| ReasoningEntry {
            supported_efforts: texts_field(held, "supported_efforts"),
        }),
        supported_parameters: texts_field(fields, "supported_parameters"),
    })
}

/// An item with a text model id.
pub fn parse_endpoint(item: &Value) -> Option<Endpoint> {
    let fields = fields_of(item)?;
    let model_id = text_field(fields, "model_id")?.to_owned();
    Some(Endpoint {
        model_id,
        tag: text_of(fields, "tag"),
        provider_name: text_of(fields, "provider_name"),
        context_length: number_field(fields, "context_length"),
        max_prompt_tokens: number_field(fields, "max_prompt_tokens"),
        supported_parameters: texts_field(fields, "supported_parameters"),
        pricing: pricing_of(fields, "pricing"),
    })
}

/// None when the body holds no list; a later entry of an id replaces the earlier in its place.
pub fn parse_models(raw: &Value) -> Option<Ordered<ModelEntry>> {
    let mut models = Ordered::new();
    for item in data_list(raw)? {
        if let Some(entry) = parse_model(item) {
            models.set(&entry.id.clone(), entry);
        }
    }
    Some(models)
}

/// None when the body holds no list.
pub fn parse_endpoints(raw: &Value) -> Option<Ordered<Vec<Endpoint>>> {
    let mut endpoints: Ordered<Vec<Endpoint>> = Ordered::new();
    for item in data_list(raw)? {
        let Some(endpoint) = parse_endpoint(item) else {
            continue;
        };
        match endpoints.get_mut(&endpoint.model_id) {
            Some(known) => known.push(endpoint),
            None => {
                let key = endpoint.model_id.clone();
                endpoints.set(&key, vec![endpoint]);
            }
        }
    }
    Some(endpoints)
}

fn usable_at(endpoints: &[Endpoint], params: &[ParamNeed]) -> Vec<usize> {
    let listed = |endpoint: &Endpoint, name: &str| {
        endpoint
            .supported_parameters
            .as_ref()
            .is_some_and(|held| held.iter().any(|each| each == name))
    };
    endpoints
        .iter()
        .enumerate()
        .filter(|(_, endpoint)| {
            params.iter().all(|param| match param {
                ParamNeed::One(name) => listed(endpoint, name),
                ParamNeed::Any(names) => names.iter().any(|name| listed(endpoint, name)),
            })
        })
        .map(|(at, _)| at)
        .collect()
}

pub fn usable_hosts<'a>(endpoints: &'a [Endpoint], params: &[ParamNeed]) -> Vec<&'a Endpoint> {
    usable_at(endpoints, params)
        .into_iter()
        .map(|at| &endpoints[at])
        .collect()
}

/// Checks model eligibility; the places in `endpoints` that count come with it.
pub fn chat_standing(
    expiration: Option<&str>,
    endpoints: &[Endpoint],
    params: &[ParamNeed],
    now: i64,
) -> (Standing, Vec<usize>) {
    if expiration
        .and_then(expiry_of)
        .is_some_and(|expires| expires <= now)
    {
        return (Standing::Retired, Vec::new());
    }
    let hosts = usable_at(endpoints, params);
    let standing = if hosts.is_empty() {
        Standing::NoZdrHost
    } else {
        Standing::Ok
    };
    (standing, hosts)
}

/// The model's own listing stands for its hosts that keep data.
fn listed_host(entry: &ModelEntry) -> Endpoint {
    Endpoint {
        model_id: entry.id.clone(),
        tag: None,
        provider_name: None,
        context_length: entry.context_length,
        max_prompt_tokens: None,
        supported_parameters: entry.supported_parameters.clone(),
        pricing: entry.pricing.clone(),
    }
}

fn is_bound(row: &ModelRow) -> bool {
    row.bindings.get("openrouter").is_some()
}

/// Catalog-backed model views.
pub fn join_catalog(
    rows: &[ModelRow],
    models: &Value,
    zdr: &Value,
    now: i64,
    use_: ModelUse,
    zdr_only: bool,
) -> Vec<ModelView> {
    let (Some(entries), Some(endpoints)) = (parse_models(models), parse_endpoints(zdr)) else {
        return rows
            .iter()
            .filter(|row| is_bound(row))
            .map(unknown_view)
            .collect();
    };
    join_parsed(rows, &entries, &endpoints, now, use_, zdr_only)
}

fn join_parsed(
    rows: &[ModelRow],
    entries: &Ordered<ModelEntry>,
    endpoints: &Ordered<Vec<Endpoint>>,
    now: i64,
    use_: ModelUse,
    zdr_only: bool,
) -> Vec<ModelView> {
    rows.iter()
        .filter(|row| is_bound(row))
        .map(|row| join_row(row, entries, endpoints, now, use_, zdr_only))
        .collect()
}

fn join_row(
    row: &ModelRow,
    entries: &Ordered<ModelEntry>,
    endpoints: &Ordered<Vec<Endpoint>>,
    now: i64,
    use_: ModelUse,
    zdr_only: bool,
) -> ModelView {
    let Some(binding) = row.bindings.get("openrouter") else {
        return unknown_view(row);
    };
    let Some(entry) = entries.get(&binding.id) else {
        let mut view = unknown_view(row);
        view.status = ModelViewStatus::Retired;
        return view;
    };
    let retired_on = entry.expiration_date.clone();
    let offered: &[String] = entry
        .reasoning
        .as_ref()
        .and_then(|held| held.supported_efforts.as_deref())
        .unwrap_or(&[]);
    let efforts = if row.reasoning {
        EFFORTS
            .iter()
            .copied()
            .filter(|effort| offered.iter().any(|held| held == effort.as_str()))
            .collect()
    } else {
        Vec::new()
    };
    let mut view = unknown_view(row);
    view.retired_on = retired_on.clone();
    view.efforts = efforts;
    let zdr: &[Endpoint] = endpoints.get(&binding.id).map(Vec::as_slice).unwrap_or(&[]);
    let mut reached: Vec<Endpoint> = zdr.to_vec();
    if !zdr_only {
        reached.push(listed_host(entry));
    }
    let (standing, hosts) = chat_standing(
        retired_on.as_deref(),
        &reached,
        &sent_params(row, use_),
        now,
    );
    if standing != Standing::Ok {
        view.status = match standing {
            Standing::Retired => ModelViewStatus::Retired,
            _ => ModelViewStatus::NoZdrHost,
        };
        return view;
    }
    let held: Vec<&Endpoint> = hosts.iter().map(|at| &reached[*at]).collect();
    let limits: Vec<f64> = held
        .iter()
        .filter_map(|host| {
            host.max_prompt_tokens
                .or(host.context_length)
                .or(entry.context_length)
        })
        .collect();
    let prompt = range_of(
        &held
            .iter()
            .map(|host| {
                per_million(
                    host.pricing
                        .as_ref()
                        .and_then(|price| price.prompt.as_ref()),
                )
            })
            .collect::<Vec<_>>(),
    );
    let completion = range_of(
        &held
            .iter()
            .map(|host| {
                per_million(
                    host.pricing
                        .as_ref()
                        .and_then(|price| price.completion.as_ref()),
                )
            })
            .collect::<Vec<_>>(),
    );
    view.status = ModelViewStatus::Ok;
    view.prompt_limit = limits.iter().copied().reduce(f64::min);
    if let (Some(prompt), Some(completion)) = (prompt, completion) {
        view.price = Some(ModelViewPrice::Price(Price {
            prompt,
            completion,
            tiers: tiers_of(entry.pricing.as_ref()),
        }));
    }
    let long_write = per_million(
        entry
            .pricing
            .as_ref()
            .and_then(|price| price.input_cache_write_1h.as_ref()),
    );
    if long_write.is_some() {
        view.cache_write1h = Some(());
    }
    view.hosts = held
        .iter()
        .map(|host| {
            host.tag
                .clone()
                .or_else(|| host.provider_name.clone())
                .unwrap_or_default()
        })
        .filter(|tag| !tag.is_empty())
        .collect();
    if !hosts.iter().any(|at| *at < zdr.len()) {
        view.data_kept = Some(());
    }
    view
}

/// A join taken at a version of the rows.
struct Joined {
    /// Row changes told before this join.
    version: u64,
    data: Option<Arc<Fetched>>,
    zdr_only: bool,
    views: Vec<ModelView>,
    until: f64,
}

struct Shared {
    deps: ModelCatalogDeps,
    fetcher: Fetcher<Arc<Fetched>>,
    joined: Mutex<HashMap<ModelUse, Joined>>,
    /// Counts the registry's changes; a join taken before one is stale.
    version: Arc<AtomicU64>,
}

/// Shared 12-hour cache; failed refreshes keep the last good data.
pub struct ModelCatalog {
    shared: Arc<Shared>,
    unsubscribe: Mutex<Option<Unsubscribe>>,
}

fn same_data(left: &Option<Arc<Fetched>>, right: &Option<Arc<Fetched>>) -> bool {
    match (left, right) {
        (None, None) => true,
        (Some(left), Some(right)) => Arc::ptr_eq(left, right),
        _ => false,
    }
}

impl ModelCatalog {
    pub fn new(deps: ModelCatalogDeps) -> Self {
        let version = Arc::new(AtomicU64::new(0));
        let counted = Arc::clone(&version);
        let unsubscribe = deps.deps.rows.on_change(Box::new(move || {
            counted.fetch_add(1, Ordering::SeqCst);
        }));
        Self {
            shared: Arc::new(Shared {
                deps,
                fetcher: Fetcher::new(),
                joined: Mutex::new(HashMap::new()),
                version,
            }),
            unsubscribe: Mutex::new(Some(unsubscribe)),
        }
    }

    /// Stops listening to the registry.
    pub fn dispose(&self) {
        let held = self
            .unsubscribe
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take();
        if let Some(unsubscribe) = held {
            unsubscribe();
        }
    }

    /// Rows bound under OpenRouter; off `zdr_only`, hosts that keep data count.
    pub async fn views(
        &self,
        use_: ModelUse,
        zdr_only: bool,
        cancel: Option<&CancelToken>,
    ) -> Result<Vec<ModelView>, Aborted> {
        let data = self.fetched().raced(cancel).await?;
        Ok(self.join(data, use_, zdr_only))
    }

    pub async fn view(
        &self,
        key: &str,
        use_: ModelUse,
        zdr_only: bool,
        cancel: Option<&CancelToken>,
    ) -> Result<Option<ModelView>, Aborted> {
        let views = self.views(use_, zdr_only, cancel).await?;
        Ok(views.into_iter().find(|view| view.row.key == key))
    }

    /// Joins a non-registry row on zero-retention hosts, as the tester checks it.
    pub async fn view_for(
        &self,
        row: &ModelRow,
        use_: ModelUse,
        cancel: Option<&CancelToken>,
    ) -> Result<ModelView, Aborted> {
        let data = self.fetched().raced(cancel).await?;
        let Some(data) = data else {
            return Ok(unknown_view(row));
        };
        let now = self.shared.deps.deps.scheduler.now_ms();
        let views = join_parsed(
            std::slice::from_ref(row),
            &data.entries,
            &data.endpoints,
            now,
            use_,
            true,
        );
        Ok(views
            .into_iter()
            .next()
            .unwrap_or_else(|| unknown_view(row)))
    }

    pub async fn entry(
        &self,
        id: &str,
        cancel: Option<&CancelToken>,
    ) -> Result<Option<ModelEntry>, Aborted> {
        let data = self.fetched().raced(cancel).await?;
        Ok(data.and_then(|held| held.entries.get(id).cloned()))
    }

    /// None if fetching has never succeeded.
    pub async fn listing(
        &self,
        cancel: Option<&CancelToken>,
        retry: bool,
    ) -> Result<Option<CatalogListing>, Aborted> {
        if retry {
            self.shared.fetcher.retry();
        }
        let data = self.fetched().raced(cancel).await?;
        Ok(data.map(|fetched| CatalogListing { fetched }))
    }

    fn fetched(&self) -> crate::kinds::fetcher::Waiter<Arc<Fetched>> {
        let shared = Arc::clone(&self.shared);
        let deps = &self.shared.deps;
        let now = deps.deps.scheduler.now_ms();
        self.shared
            .fetcher
            .fetched(now, deps.ttl_ms.unwrap_or(TTL_MS), move || {
                let limit = deadline(&*shared.deps.deps.scheduler, FETCH_MS, &CancelToken::new());
                Box::pin(async move { refresh(&shared, limit).await })
            })
    }

    /// Reuses the views until the rows, the data, the flag or a removal date changes.
    fn join(&self, data: Option<Arc<Fetched>>, use_: ModelUse, zdr_only: bool) -> Vec<ModelView> {
        let shared = &self.shared;
        let now = shared.deps.deps.scheduler.now_ms();
        let held: MutexGuard<'_, HashMap<ModelUse, Joined>> = shared
            .joined
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut held = held;
        let version = shared.version.load(Ordering::SeqCst);
        if let Some(known) = held.get(&use_)
            && known.version == version
            && same_data(&known.data, &data)
            && known.zdr_only == zdr_only
            && (now as f64) < known.until
        {
            return known.views.clone();
        }
        let rows = shared.deps.deps.rows.rows();
        let views = match &data {
            None => rows
                .iter()
                .filter(|row| is_bound(row))
                .map(unknown_view)
                .collect(),
            Some(fetched) => join_parsed(
                &rows,
                &fetched.entries,
                &fetched.endpoints,
                now,
                use_,
                zdr_only,
            ),
        };
        let until = views
            .iter()
            .filter_map(|view| view.retired_on.as_deref().or(Some("")).and_then(expiry_of))
            .filter(|at| *at > now)
            .map(|at| at as f64)
            .fold(f64::INFINITY, f64::min);
        held.insert(
            use_,
            Joined {
                version,
                data,
                zdr_only,
                views: views.clone(),
                until,
            },
        );
        views
    }
}

async fn refresh(
    shared: &Shared,
    limit: nyaaai_contract::time::Deadline,
) -> Refreshed<Arc<Fetched>> {
    let scheduler = &shared.deps.deps.scheduler;
    let transport = &shared.deps.transport;
    let (models, zdr) = {
        let first = transport.get_json("/models", limit.token(), &KeyAuth::None);
        let second = transport.get_json("/endpoints/zdr", limit.token(), &KeyAuth::None);
        both(first, second).await
    };
    let outcome = match (models, zdr) {
        (Ok(models), Ok(zdr)) => {
            let entries = parse_models(&models);
            let endpoints = parse_endpoints(&zdr);
            let listed = data_list(&models).map(<[Value]>::to_vec);
            match (entries, endpoints, listed) {
                (Some(entries), Some(endpoints), Some(raw)) => Some(Fetched {
                    entries,
                    endpoints,
                    raw,
                }),
                _ => None,
            }
        }
        _ => None,
    };
    limit.clear();
    match outcome {
        Some(fetched) => {
            let at = scheduler.now_ms();
            (shared.deps.deps.refreshed)();
            Refreshed::Good {
                value: Arc::new(fetched),
                at,
            }
        }
        None => Refreshed::Failed {
            at: scheduler.now_ms(),
        },
    }
}

/// Runs two reads at once, the first started first.
pub(crate) async fn both<A, B>(
    first: impl Future<Output = A>,
    second: impl Future<Output = B>,
) -> (A, B) {
    let mut first = std::pin::pin!(first);
    let mut second = std::pin::pin!(second);
    let (mut a, mut b) = (None, None);
    std::future::poll_fn(|context| {
        use std::task::Poll;
        if a.is_none()
            && let Poll::Ready(held) = first.as_mut().poll(context)
        {
            a = Some(held);
        }
        if b.is_none()
            && let Poll::Ready(held) = second.as_mut().poll(context)
        {
            b = Some(held);
        }
        if a.is_some() && b.is_some() {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    })
    .await;
    match (a, b) {
        (Some(a), Some(b)) => (a, b),
        _ => unreachable!("both reads finished"),
    }
}
