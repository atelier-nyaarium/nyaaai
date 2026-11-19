//! What a provider kind gives a host: catalogs, a browser and a tester.

use std::any::Any;
use std::sync::Arc;

use nyaaai_contract::cancel::CancelToken;
use nyaaai_contract::fetch::Fetch;
use nyaaai_contract::future::BoxFuture;
use nyaaai_contract::model_registry::ModelRegistry;
use nyaaai_contract::shapes::model::{BrowseDetail, BrowseModel, ModelRow, ModelUse, ModelView};
use nyaaai_contract::shapes::provider::{
    Billed, Disposition, EntryFields, KeyState, Limits, ProviderEntry, ProviderKindName,
    ProviderPlan,
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
    /// The time, and the deadlines of a fetch.
    pub scheduler: Arc<dyn Scheduler>,
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

    /// The catalog as its own type, for a kind's browser.
    fn as_any(&self) -> &dyn Any;
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
    pub fetch: Arc<dyn Fetch>,
    pub scheduler: Arc<dyn Scheduler>,
    pub log: Arc<dyn Fn(&str) + Send + Sync>,
    pub user_agent: String,
    /// Overrides the entry's URL.
    pub base_url: Option<String>,
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

    /// Key or URL check; an error is the caller's cancel, or a refusal that is no state.
    fn check<'a>(
        &'a self,
        entry: &'a EntryFields,
        secret: Option<&'a str>,
        deps: &'a KindDeps,
        cancel: &'a CancelToken,
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
    fn browse(&self, _entry: &ProviderEntry, _route: &Route) -> Option<Box<dyn Browser>> {
        None
    }

    fn tester(&self, _entry: &ProviderEntry, _route: &Route) -> Option<Box<dyn Tester>> {
        None
    }

    /// Model request codec.
    fn codec(&self, view: Option<&ModelView>, entry: &EntryFields) -> Box<dyn Codec>;

    /// Retry, fall-through, or stop.
    fn failure(&self, failure: &Failure, view: Option<&ModelView>) -> Disposition;

    /// Optional provider limits.
    fn limits<'a>(
        &'a self,
        _transport: &'a Transport,
        _cancel: &'a CancelToken,
    ) -> BoxFuture<'a, Option<Limits>> {
        Box::pin(async { None })
    }
}
