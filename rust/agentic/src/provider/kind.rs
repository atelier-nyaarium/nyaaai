//! What a provider kind gives a host: catalogs, a browser and a tester.

use std::any::Any;
use std::sync::Arc;

use tokio::runtime::Handle;

use nyaaai_contract::cancel::CancelToken;
use nyaaai_contract::fetch::{Budgets, Fetch, Span};
use nyaaai_contract::future::BoxFuture;
use nyaaai_contract::model_registry::ModelRegistry;
use nyaaai_contract::shapes::model::{BrowseDetail, BrowseModel, ModelRow, ModelUse, ModelView};
use nyaaai_contract::shapes::provider::{
    Billed, Disposition, EntryFields, KeyState, KindLine, KindLineField, Limits, ProviderEntry,
    ProviderKindName, ProviderPlan, ProviderView,
};
use nyaaai_contract::shapes::step::Failure;
use nyaaai_contract::time::Scheduler;

use crate::codec::Codec;
use crate::failure::ProviderError;
use crate::kinds::fetcher::Aborted;
use crate::transport::{KeySource, Transport, TransportError};

/// What a catalog stands on.
#[derive(Clone)]
pub struct CatalogDeps {
    /// Where a refresh or a read runs, from any thread.
    pub runtime: Handle,
    /// The time, and the spans of its reads.
    pub scheduler: Arc<dyn Scheduler>,
    /// Default the presets.
    pub budgets: Option<Budgets>,
    /// Rows; a catalog that caches a join rejoins when this tells a change.
    pub rows: Arc<dyn ModelRegistry>,
    /// Told after each successful refresh.
    pub refreshed: Arc<dyn Fn() + Send + Sync>,
}

/// How a call of a kind ended, short of an answer.
#[derive(Debug, Clone, PartialEq)]
pub enum KindError {
    /// The caller's token was cancelled.
    Cancelled,
    /// A deadline of the kind's own passed.
    TimedOut,
    Provider(ProviderError),
    /// Any other refusal, by its text.
    Message(String),
}

impl From<Aborted> for KindError {
    fn from(_: Aborted) -> Self {
        Self::Cancelled
    }
}

impl From<TransportError> for KindError {
    fn from(error: TransportError) -> Self {
        match error {
            TransportError::Cancelled => Self::Cancelled,
            TransportError::Provider(error) => Self::Provider(error),
        }
    }
}

/// Entry-specific bound model data.
pub trait EntryCatalog: Send + Sync {
    /// ZDR-scoped bound rows.
    fn views(
        &self,
        use_: ModelUse,
        zdr_only: bool,
        cancel: Option<&CancelToken>,
    ) -> BoxFuture<'_, Result<Vec<ModelView>, Aborted>>;

    /// Reads the server again, when the kind can.
    fn rediscover(&self) {}

    /// Stops what the catalog listens to; a route that goes calls it.
    fn dispose(&self) {}

    /// Answers when no refresh, discovery or reread of this catalog is running.
    fn idle(&self) -> BoxFuture<'_, ()> {
        Box::pin(async {})
    }

    /// The catalog as its own type, for a kind's browser.
    fn as_any(&self) -> &dyn Any;

    /// The same, held: a browser keeps its catalog.
    fn arc_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync>;
}

/// An entry's model browser.
pub trait Browser: Send + Sync {
    /// Fresh retries failed reads.
    fn list(&self, fresh: bool) -> BoxFuture<'_, Result<Option<Vec<BrowseModel>>, KindError>>;

    /// Optional detail by the kind's own model id.
    fn detail<'a>(
        &'a self,
        id: &'a str,
        cancel: &'a CancelToken,
    ) -> BoxFuture<'a, Result<Option<BrowseDetail>, KindError>>;
}

/// What a tester reads of a listed model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Listed {
    pub label: String,
    pub reasoning: bool,
}

/// One entry's model facts for the tester.
pub trait Tester: Send + Sync {
    /// `id` is the kind's own model id.
    fn listed<'a>(
        &'a self,
        id: &'a str,
        cancel: &'a CancelToken,
    ) -> BoxFuture<'a, Result<Option<Listed>, KindError>>;

    /// ZDR view per use.
    fn view<'a>(
        &'a self,
        row: &'a ModelRow,
        use_: ModelUse,
        cancel: &'a CancelToken,
    ) -> BoxFuture<'a, Result<ModelView, KindError>>;

    /// Fresh per-test view.
    fn pins(&self) -> Option<Box<dyn Pins>> {
        None
    }
}

/// Test host routing.
pub trait Pins: Send + Sync {
    /// Two-host ZDR alternation; `id` is the kind's own.
    fn hop<'a>(
        &'a self,
        id: &'a str,
        view: &'a ModelView,
        cancel: &'a CancelToken,
    ) -> BoxFuture<'a, Result<Vec<String>, KindError>>;

    /// Pin and record streams.
    fn wrap(&self, codec: Box<dyn Codec>) -> Box<dyn Codec>;

    /// Alternating hosts; empty unpins.
    fn pin(&self, hosts: &[String]);

    /// Detect resent reasoning.
    fn passed_back(&self) -> bool;
}

/// What a kind needs of its host.
#[derive(Clone)]
pub struct KindDeps {
    /// Where the kind's background work runs, from any thread.
    pub runtime: Handle,
    pub fetch: Arc<dyn Fetch>,
    pub scheduler: Arc<dyn Scheduler>,
    pub log: Arc<dyn Fn(&str) + Send + Sync>,
    pub user_agent: String,
    /// Overrides the entry's URL.
    pub base_url: Option<String>,
    /// What every span opens from; default the presets.
    pub budgets: Option<Budgets>,
}

/// An entry's transport and catalog.
#[derive(Clone)]
pub struct Route {
    pub transport: Arc<Transport>,
    pub catalog: Arc<dyn EntryCatalog>,
}

/// The third field of the add form.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KindField {
    Key,
    Url,
}

/// Provider engine.
pub trait ProviderKind: Send + Sync {
    fn kind(&self) -> ProviderKindName;

    fn label(&self) -> &str;

    /// Plan dropdown lines.
    fn plans(&self) -> &[(ProviderPlan, &'static str)] {
        &[]
    }

    /// Add-form third field.
    fn field(&self) -> KindField;

    /// URL kinds that also take a key.
    fn optional_key(&self) -> bool {
        false
    }

    /// Default URL.
    fn url(&self) -> Option<&str> {
        None
    }

    /// The balance page.
    fn balance(&self) -> Option<&str> {
        None
    }

    /// Plans with extra usage.
    fn extra_usage(&self) -> &[ProviderPlan] {
        &[]
    }

    /// Delayed generation stats.
    fn stats(&self) -> bool {
        false
    }

    /// Provider status facts.
    fn facts(
        &self,
        _state: &KeyState,
        _entry: &EntryFields,
        _bill: Option<&Billed>,
    ) -> Option<String> {
        None
    }

    /// Key or URL check, its reads drawn from the span; `Err` is the span's end or a refusal, not a state.
    fn check<'a>(
        &'a self,
        entry: &'a EntryFields,
        secret: Option<&'a str>,
        deps: &'a KindDeps,
        span: &'a Span,
    ) -> BoxFuture<'a, Result<KeyState, KindError>>;

    /// A refusal is the text of why no transport can be made.
    fn transport(
        &self,
        entry: &EntryFields,
        key: Arc<dyn KeySource>,
        deps: &KindDeps,
    ) -> Result<Arc<Transport>, String>;

    fn catalog(
        &self,
        entry: &ProviderEntry,
        transport: Arc<Transport>,
        deps: CatalogDeps,
    ) -> Arc<dyn EntryCatalog>;

    /// Omitted if unavailable.
    fn browse(
        &self,
        _entry: &ProviderEntry,
        _route: &Route,
        _deps: &KindDeps,
    ) -> Option<Box<dyn Browser>> {
        None
    }

    fn tester(
        &self,
        _entry: &ProviderEntry,
        _route: &Route,
        _deps: &KindDeps,
    ) -> Option<Box<dyn Tester>> {
        None
    }

    /// Model request codec.
    fn codec(
        &self,
        view: Option<&ModelView>,
        entry: &EntryFields,
        deps: &KindDeps,
    ) -> Box<dyn Codec>;

    /// Retry, fall-through, or stop.
    fn failure(&self, failure: &Failure, view: Option<&ModelView>) -> Disposition;

    /// Whether the kind reads provider limits.
    fn has_limits(&self) -> bool {
        false
    }

    /// Optional provider limits.
    fn limits<'a>(
        &'a self,
        _transport: &'a Transport,
        _span: &'a Span,
    ) -> BoxFuture<'a, Option<Limits>> {
        Box::pin(async { None })
    }
}

/// Dropdown order.
pub static KINDS: std::sync::LazyLock<Vec<Arc<dyn ProviderKind>>> =
    std::sync::LazyLock::new(|| {
        vec![
            Arc::new(crate::kinds::opencode::kind::OpenCodeKind),
            Arc::new(crate::kinds::openrouter::kind::OpenRouterKind),
            crate::kinds::ollama::OLLAMA.clone() as Arc<dyn ProviderKind>,
            crate::kinds::lmstudio::LMSTUDIO.clone() as Arc<dyn ProviderKind>,
            crate::kinds::llamacpp::LLAMACPP.clone() as Arc<dyn ProviderKind>,
            crate::kinds::compatible::COMPATIBLE.clone() as Arc<dyn ProviderKind>,
        ]
    });

/// One line per plan.
pub fn kind_lines(kinds: &[Arc<dyn ProviderKind>]) -> Vec<KindLine> {
    kinds
        .iter()
        .flat_map(|kind| {
            let line = |plan: Option<ProviderPlan>, label: &str| KindLine {
                kind: kind.kind(),
                plan,
                label: label.to_owned(),
                field: match kind.field() {
                    KindField::Key => KindLineField::Key,
                    KindField::Url => KindLineField::Url,
                },
                url: kind.url().map(str::to_owned),
                optional_key: kind.optional_key().then_some(()),
            };
            let plans = kind.plans();
            if plans.is_empty() {
                vec![line(None, kind.label())]
            } else {
                plans
                    .iter()
                    .map(|(plan, label)| line(Some(*plan), label))
                    .collect()
            }
        })
        .collect()
}

/// An entry's line of the provider list.
pub fn provider_view(
    entry: &ProviderEntry,
    status: &KeyState,
    kinds: &[Arc<dyn ProviderKind>],
    bill: Option<&Billed>,
) -> ProviderView {
    let kind = kinds.iter().find(|held| held.kind() == entry.kind);
    let fields = EntryFields {
        kind: entry.kind,
        label: entry.label.clone(),
        plan: entry.plan,
        url: entry.url.clone(),
        extra_usage: entry.extra_usage,
    };
    ProviderView {
        id: entry.id.clone(),
        kind: entry.kind,
        plan: entry.plan,
        label: entry.label.clone(),
        url: entry.url.clone(),
        status: status.clone(),
        facts: kind.and_then(|held| held.facts(status, &fields, bill)),
        balance: kind.and_then(|held| held.balance()).map(|_| ()),
        extra_usage: match (entry.plan, kind) {
            (Some(plan), Some(held)) if held.extra_usage().contains(&plan) => {
                Some(entry.extra_usage == Some(true))
            }
            _ => None,
        },
    }
}
