//! What a provider kind gives a host: catalogs, a browser and a tester.

use std::sync::Arc;

use nyaaai_contract::cancel::CancelToken;
use nyaaai_contract::future::BoxFuture;
use nyaaai_contract::model_registry::ModelRegistry;
use nyaaai_contract::shapes::model::{BrowseDetail, BrowseModel, ModelRow, ModelUse, ModelView};
use nyaaai_contract::time::Scheduler;

use crate::codec::Codec;
use crate::failure::ProviderError;
use crate::kinds::fetcher::Aborted;
use crate::transport::TransportError;

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
