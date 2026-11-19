//! The local catalog: one entry's discovered models, their views, browser and tester.

use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};

use nyaaai_contract::cancel::CancelToken;
use nyaaai_contract::future::BoxFuture;
use nyaaai_contract::json::{Object, Value};
use nyaaai_contract::model_registry::ModelRegistry;
use nyaaai_contract::shapes::model::{
    BrowseDetail, BrowseModel, BrowseModelPrice, BrowseModelTools, ModelRow, ModelUse, ModelView,
    ModelViewCache, ModelViewPrice, ModelViewStatus, RowIdentity, RowIdentityLocal,
};
use nyaaai_contract::shapes::step::{ApiFacts, ApiFamily, Effort};
use nyaaai_contract::text::{head_of, trim_text};
use nyaaai_contract::time::{Scheduler, deadline};

use crate::effort::EFFORTS;
use crate::kinds::fetcher::{Aborted, Fetcher, Refreshed, Waiter};
use crate::kinds::readers::{field_of, fields_of};
use crate::models::row_key::{row_key, shown_id};
use crate::provider::kind::{Browser, EntryCatalog, KindError, Listed, Tester};

use super::kind::Terms;

/// Loaded contexts by tag, in the order the server told them.
pub type Loaded = Vec<(String, f64)>;

/// One served model.
#[derive(Debug, Clone, PartialEq)]
pub struct LocalModel {
    pub tag: String,
    pub tools: bool,
    pub thinking: bool,
    pub efforts: Vec<Effort>,
    /// Configured context.
    pub context: Option<f64>,
    /// Current load's context.
    pub loaded: Option<f64>,
    /// Trained length.
    pub trained: Option<f64>,
    /// Facts not read this load.
    pub unprobed: bool,
}

/// One server's discovery.
pub trait LocalServer: Send + Sync {
    /// Fails when unreadable; `known` is the last list.
    fn models<'a>(
        &'a self,
        known: &'a [LocalModel],
        cancel: &'a CancelToken,
    ) -> BoxFuture<'a, Result<Vec<LocalModel>, KindError>>;

    /// Whether the server tells a loaded context after a reply.
    fn reads_loaded(&self) -> bool {
        false
    }

    /// Loaded context by tag, after a reply.
    fn loaded<'a>(
        &'a self,
        _tag: &'a str,
        _cancel: &'a CancelToken,
    ) -> BoxFuture<'a, Result<Loaded, KindError>> {
        Box::pin(async { Ok(Vec::new()) })
    }

    /// A reload can change context.
    fn live(&self) -> bool {
        false
    }

    /// Whether the server loads an unprobed model.
    fn warms(&self) -> bool {
        false
    }

    /// Loads an unprobed model.
    fn warm<'a>(
        &'a self,
        _tag: &'a str,
        _cancel: &'a CancelToken,
    ) -> BoxFuture<'a, Result<(), KindError>> {
        Box::pin(async { Ok(()) })
    }
}

/// Server-reported model.
#[derive(Debug, Clone, PartialEq)]
pub struct Discovered {
    pub key: String,
    /// `shown_id` text.
    pub display: String,
    pub tag: String,
    pub tools: bool,
    pub reasoning: bool,
    pub context: Option<f64>,
}

#[derive(Clone)]
pub struct LocalCatalogDeps {
    /// Discovering entry ID.
    pub entry: String,
    pub server: Arc<dyn LocalServer>,
    pub terms: Terms,
    /// The time, and the deadlines of a read.
    pub scheduler: Arc<dyn Scheduler>,
    pub rows: Arc<dyn ModelRegistry>,
    pub refreshed: Arc<dyn Fn() + Send + Sync>,
}

/// A thinking value a server lists: on or off, or a named level.
#[derive(Debug, Clone, PartialEq)]
pub enum Think {
    Flag(bool),
    Text(String),
}

const RETRY_MS: i64 = 60_000;
const DISCOVER_MS: i64 = 10_000;
const LABEL_UNITS: usize = 200;
const ON: [Effort; 3] = [Effort::Low, Effort::Medium, Effort::High];

/// The items of `{ [key]: [...] }`; none unless it is a list.
pub fn items_of<'a>(raw: &'a Value, key: &str) -> Option<&'a [Value]> {
    field_of(fields_of(raw)?, key)?.as_array()
}

/// `false`: none; `true`: all.
pub fn efforts_of(values: &[Think]) -> Vec<Effort> {
    let has_flag = |flag: bool| values.contains(&Think::Flag(flag));
    EFFORTS
        .iter()
        .copied()
        .filter(|effort| {
            values.contains(&Think::Text(effort.as_str().to_owned()))
                || (*effort == Effort::None && has_flag(false))
                || (has_flag(true) && ON.contains(effort))
        })
        .collect()
}

/// Efforts read from named levels.
pub fn efforts_of_texts(values: &[String]) -> Vec<Effort> {
    let thinks: Vec<Think> = values
        .iter()
        .map(|text| Think::Text(text.clone()))
        .collect();
    efforts_of(&thinks)
}

fn api_of(model: &LocalModel) -> ApiFacts {
    ApiFacts {
        family: ApiFamily::ChatCompletions,
        structured: true,
        choice: Vec::new(),
        strict: false,
        reasoning: model.thinking.then(|| "reasoning".to_owned()),
        toggle: None,
    }
}

/// Trained length caps context.
pub fn context_of(model: &LocalModel, loaded: Option<f64>) -> Option<f64> {
    let known = model.context.or(loaded)?;
    Some(known.min(model.trained.unwrap_or(known)))
}

/// A model's view for a row under the entry's terms.
pub fn local_view(
    row: &ModelRow,
    model: &LocalModel,
    loaded: Option<f64>,
    terms: &Terms,
    zdr_only: bool,
) -> ModelView {
    let plain = |status| ModelView {
        row: row.clone(),
        status,
        retired_on: None,
        efforts: model.efforts.clone(),
        prompt_limit: None,
        output_limit: None,
        price: Some(terms.view_price()),
        cache_write1h: None,
        hosts: Vec::new(),
        data_kept: None,
        api: None,
        cache: None,
    };
    if !model.tools {
        return plain(ModelViewStatus::NoTools);
    }
    if zdr_only && terms.kept {
        return plain(ModelViewStatus::NoZdrHost);
    }
    ModelView {
        prompt_limit: context_of(model, loaded),
        api: Some(api_of(model)),
        cache: Some(ModelViewCache::Implicit),
        data_kept: terms.kept.then_some(()),
        ..plain(ModelViewStatus::Ok)
    }
}

/// The last non-blank part of a path-like tag; record labels fit 200.
pub fn label_of(tag: &str) -> String {
    let mut last: Option<&str> = None;
    let mut from = 0;
    for (at, char) in tag.char_indices().chain([(tag.len(), '/')]) {
        if char != '/' && char != '\\' {
            continue;
        }
        let part = &tag[from..at];
        if !trim_text(part).is_empty() {
            last = Some(part);
        }
        from = at + char.len_utf8();
    }
    head_of(last.unwrap_or(tag), LABEL_UNITS).to_owned()
}

/// Retestable; the server behind a tag can change.
fn browse_of(model: &Discovered, terms: &Terms) -> BrowseModel {
    BrowseModel {
        id: model.key.clone(),
        display: model.display.clone(),
        label: model.tag.clone(),
        curated: false,
        description: String::new(),
        context: model.context,
        inputs: Vec::new(),
        outputs: Vec::new(),
        parameters: [
            model.tools.then(|| "tools".to_owned()),
            model.reasoning.then(|| "reasoning".to_owned()),
        ]
        .into_iter()
        .flatten()
        .collect(),
        price: match terms.view_price() {
            ModelViewPrice::Free => BrowseModelPrice::Free,
            _ => BrowseModelPrice::Unknown,
        },
        tools: if model.tools {
            BrowseModelTools::Ok
        } else {
            BrowseModelTools::None
        },
        retestable: Some(()),
        family: Some(ApiFamily::ChatCompletions),
    }
}

struct Inner {
    /// Loaded context by tag.
    loaded: HashMap<String, f64>,
    /// Last reread by tag.
    reread: HashMap<String, i64>,
    reading: bool,
}

type Models = Arc<Vec<LocalModel>>;

struct Shared {
    entry: String,
    server: Arc<dyn LocalServer>,
    terms: Terms,
    scheduler: Arc<dyn Scheduler>,
    rows: Arc<dyn ModelRegistry>,
    refreshed: Arc<dyn Fn() + Send + Sync>,
    fetcher: Fetcher<Models>,
    inner: Mutex<Inner>,
}

impl Shared {
    fn locked(&self) -> MutexGuard<'_, Inner> {
        self.inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn loaded_of(&self, tag: &str) -> Option<f64> {
        self.locked().loaded.get(tag).copied()
    }

    /// This entry's local rows only.
    fn tag_of<'a>(&self, row: &'a ModelRow) -> Option<&'a str> {
        match &row.identity {
            RowIdentity::Local(local) if local.entry == self.entry => Some(&local.tag),
            _ => None,
        }
    }
}

/// One refresh: the server's list, and the loaded contexts it carries.
fn refresh(shared: &Arc<Shared>, known: Vec<LocalModel>) -> BoxFuture<'static, Refreshed<Models>> {
    let shared = Arc::clone(shared);
    let limit = deadline(&*shared.scheduler, DISCOVER_MS, &CancelToken::new());
    Box::pin(async move {
        let read = shared.server.models(&known, limit.token()).await;
        limit.clear();
        match read {
            Ok(models) => {
                {
                    let mut inner = shared.locked();
                    inner
                        .loaded
                        .retain(|tag, _| models.iter().any(|model| &model.tag == tag));
                    for model in &models {
                        if let Some(loaded) = model.loaded {
                            inner.loaded.insert(model.tag.clone(), loaded);
                        } else if shared.server.live() {
                            inner.loaded.remove(&model.tag);
                        }
                    }
                }
                (shared.refreshed)();
                Refreshed::Good {
                    value: Arc::new(models),
                    at: shared.scheduler.now_ms(),
                }
            }
            Err(_) => Refreshed::Failed {
                at: shared.scheduler.now_ms(),
            },
        }
    })
}

/// The last list, which a refresh hands the server.
fn known_of(shared: &Shared) -> Vec<LocalModel> {
    shared
        .fetcher
        .good()
        .map(|good| good.as_ref().clone())
        .unwrap_or_default()
}

fn fetched(shared: &Arc<Shared>) -> Waiter<Models> {
    let now = shared.scheduler.now_ms();
    let known = known_of(shared);
    shared
        .fetcher
        .fetched(now, i64::MAX, || refresh(shared, known))
}

fn discovering(shared: &Arc<Shared>) -> Waiter<Models> {
    let known = known_of(shared);
    shared.fetcher.refresh_now(|| refresh(shared, known))
}

/// A reply was seen for a tag: reread an unprobed model, or learn its loaded context.
fn settled(shared: &Arc<Shared>, tag: &str) {
    let model = shared
        .fetcher
        .good()
        .and_then(|models| models.iter().find(|each| each.tag == tag).cloned());
    if model.as_ref().is_some_and(|held| held.unprobed) {
        let now = shared.scheduler.now_ms();
        {
            let mut inner = shared.locked();
            if inner.reread.get(tag).is_some_and(|at| now - at < RETRY_MS) {
                return;
            }
            inner.reread.insert(tag.to_owned(), now);
        }
        let _ = discovering(shared);
        return;
    }
    let Some(model) = model else { return };
    if !shared.server.reads_loaded() {
        return;
    }
    {
        let mut inner = shared.locked();
        if model.context.is_some()
            || inner.reading
            || (inner.loaded.contains_key(tag) && !shared.server.live())
        {
            return;
        }
        inner.reading = true;
    }
    let limit = deadline(&*shared.scheduler, DISCOVER_MS, &CancelToken::new());
    let shared = Arc::clone(shared);
    let tag = tag.to_owned();
    tokio::spawn(async move {
        let read = shared.server.loaded(&tag, limit.token()).await;
        if let Ok(contexts) = read {
            let mut changed = false;
            {
                let mut inner = shared.locked();
                for (name, context) in contexts {
                    if inner.loaded.get(&name) == Some(&context) {
                        continue;
                    }
                    inner.loaded.insert(name, context);
                    changed = true;
                }
            }
            if changed {
                (shared.refreshed)();
            }
        }
        limit.clear();
        shared.locked().reading = false;
    });
}

/// One entry's discovered models; keeps the last list on errors.
#[derive(Clone)]
pub struct LocalCatalog {
    shared: Arc<Shared>,
}

impl LocalCatalog {
    pub fn new(deps: LocalCatalogDeps) -> Self {
        Self {
            shared: Arc::new(Shared {
                entry: deps.entry,
                server: deps.server,
                terms: deps.terms,
                scheduler: deps.scheduler,
                rows: deps.rows,
                refreshed: deps.refreshed,
                fetcher: Fetcher::new(),
                inner: Mutex::new(Inner {
                    loaded: HashMap::new(),
                    reread: HashMap::new(),
                    reading: false,
                }),
            }),
        }
    }

    pub fn terms(&self) -> &Terms {
        &self.shared.terms
    }

    /// A reply was seen for this tag.
    pub fn settled(&self, tag: &str) {
        settled(&self.shared, tag);
    }

    /// A listener for a transport's settled replies; it holds the catalog weakly.
    pub fn listener(&self) -> impl Fn(&str) + Send + Sync + 'static {
        let weak = Arc::downgrade(&self.shared);
        move |tag| {
            if let Some(shared) = weak.upgrade() {
                settled(&shared, tag);
            }
        }
    }

    pub fn rediscover(&self) {
        let _ = discovering(&self.shared);
    }

    /// Tested local model views.
    pub async fn views(
        &self,
        zdr_only: bool,
        cancel: Option<&CancelToken>,
    ) -> Result<Vec<ModelView>, Aborted> {
        let waiter = fetched(&self.shared);
        let Some(found) = waiter.raced(cancel).await? else {
            return Ok(Vec::new());
        };
        let rows = self.shared.rows.rows();
        let mut tested: HashMap<&str, &ModelRow> = HashMap::new();
        for row in &rows {
            if let Some(tag) = self.shared.tag_of(row) {
                tested.insert(tag, row);
            }
        }
        Ok(found
            .iter()
            .filter_map(|model| {
                let row = tested.get(model.tag.as_str())?;
                Some(local_view(
                    row,
                    model,
                    self.shared.loaded_of(&model.tag),
                    &self.shared.terms,
                    zdr_only,
                ))
            })
            .collect())
    }

    /// Refreshes server discovery.
    pub async fn discovered(
        &self,
        fresh: bool,
        cancel: Option<&CancelToken>,
    ) -> Result<Option<Vec<Discovered>>, Aborted> {
        let waiter = if fresh {
            discovering(&self.shared)
        } else {
            fetched(&self.shared)
        };
        let found = waiter.raced(cancel).await?;
        Ok(found.map(|models| {
            models
                .iter()
                .map(|model| {
                    let context = context_of(model, self.shared.loaded_of(&model.tag));
                    let identity = RowIdentity::Local(RowIdentityLocal {
                        entry: self.shared.entry.clone(),
                        tag: model.tag.clone(),
                    });
                    Discovered {
                        key: row_key(&identity),
                        display: shown_id(&identity),
                        tag: model.tag.clone(),
                        tools: model.tools,
                        reasoning: model.thinking,
                        context,
                    }
                })
                .collect()
        }))
    }

    /// Tester view; its prompts are synthetic.
    pub async fn view_for(
        &self,
        row: &ModelRow,
        cancel: Option<&CancelToken>,
    ) -> Result<ModelView, KindError> {
        let waiter = fetched(&self.shared);
        let found = waiter.raced(cancel).await?;
        let tag = self.shared.tag_of(row);
        let model = found
            .as_ref()
            .and_then(|models| models.iter().find(|each| Some(each.tag.as_str()) == tag));
        let Some(model) = model else {
            return Err(KindError::Message(format!(
                "{} is not on the server.",
                row.label
            )));
        };
        // Synthetic prompts; no retention gate.
        let terms = Terms {
            kept: false,
            ..self.shared.terms.clone()
        };
        Ok(local_view(
            row,
            model,
            self.shared.loaded_of(&model.tag),
            &terms,
            false,
        ))
    }

    /// Tests read real capabilities.
    pub async fn warmed(&self, tag: &str, cancel: &CancelToken) -> Result<(), KindError> {
        let waiter = fetched(&self.shared);
        let found = waiter.raced(Some(cancel)).await?;
        let unprobed = found
            .and_then(|models| models.iter().find(|each| each.tag == tag).cloned())
            .is_some_and(|model| model.unprobed);
        if !self.shared.server.warms() || !unprobed {
            return Ok(());
        }
        self.shared.server.warm(tag, cancel).await?;
        discovering(&self.shared).raced(Some(cancel)).await?;
        Ok(())
    }
}

impl EntryCatalog for LocalCatalog {
    fn views(
        &self,
        _use: ModelUse,
        zdr_only: bool,
        cancel: Option<&CancelToken>,
    ) -> BoxFuture<'_, Result<Vec<ModelView>, Aborted>> {
        let cancel = cancel.cloned();
        Box::pin(async move { LocalCatalog::views(self, zdr_only, cancel.as_ref()).await })
    }

    fn rediscover(&self) {
        LocalCatalog::rediscover(self);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn arc_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync> {
        self
    }
}

/// The browser of a local catalog.
pub struct LocalBrowser {
    catalog: LocalCatalog,
}

pub fn local_browser(catalog: &LocalCatalog) -> LocalBrowser {
    LocalBrowser {
        catalog: catalog.clone(),
    }
}

impl Browser for LocalBrowser {
    fn list(&self, fresh: bool) -> BoxFuture<'_, Result<Option<Vec<BrowseModel>>, KindError>> {
        Box::pin(async move {
            let found = self.catalog.discovered(fresh, None).await?;
            Ok(found.map(|models| {
                models
                    .iter()
                    .map(|model| browse_of(model, self.catalog.terms()))
                    .collect()
            }))
        })
    }

    fn detail<'a>(
        &'a self,
        _id: &'a str,
        _cancel: &'a CancelToken,
    ) -> BoxFuture<'a, Result<Option<BrowseDetail>, KindError>> {
        Box::pin(async {
            Ok(Some(BrowseDetail {
                hosts: None,
                max_output: None,
                efforts: None,
                choice: None,
                cache: None,
                retention: None,
            }))
        })
    }
}

/// The tester of a local catalog.
pub struct LocalTester {
    catalog: LocalCatalog,
}

pub fn local_tester(catalog: &LocalCatalog) -> LocalTester {
    LocalTester {
        catalog: catalog.clone(),
    }
}

impl Tester for LocalTester {
    fn listed<'a>(
        &'a self,
        id: &'a str,
        cancel: &'a CancelToken,
    ) -> BoxFuture<'a, Result<Option<Listed>, KindError>> {
        Box::pin(async move {
            self.catalog.warmed(id, cancel).await?;
            let found = self.catalog.discovered(false, Some(cancel)).await?;
            Ok(found
                .and_then(|models| models.into_iter().find(|each| each.tag == id))
                .map(|model| Listed {
                    label: label_of(&model.tag),
                    reasoning: model.reasoning,
                }))
        })
    }

    fn view<'a>(
        &'a self,
        row: &'a ModelRow,
        _use: ModelUse,
        cancel: &'a CancelToken,
    ) -> BoxFuture<'a, Result<ModelView, KindError>> {
        Box::pin(async move { self.catalog.view_for(row, Some(cancel)).await })
    }
}

/// An object holding one text under a key, for a request body.
pub fn body_of(key: &str, value: &str) -> Value {
    let mut object = Object::new();
    object.insert(key, Value::from(value));
    Value::Object(object)
}

/// Sets a context by tag, keeping the first place of a tag read twice.
pub fn put_loaded(contexts: &mut Loaded, tag: &str, context: f64) {
    match contexts.iter_mut().find(|(name, _)| name == tag) {
        Some(held) => held.1 = context,
        None => contexts.push((tag.to_owned(), context)),
    }
}
