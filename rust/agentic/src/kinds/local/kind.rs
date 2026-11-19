//! A provider kind for a server named by URL.

use std::sync::Arc;

use nyaaai_contract::cancel::CancelToken;
use nyaaai_contract::future::BoxFuture;
use nyaaai_contract::shapes::model::{ModelView, ModelViewPrice};
use nyaaai_contract::shapes::provider::{
    Billed, Disposition, DispositionFallThrough, EntryFields, KeyState, KeyStateOk,
    KeyStateRejected, KeyStateUnreachable, NoticeKind, ProviderEntry, ProviderKindName,
};
use nyaaai_contract::shapes::step::{Failure, FailureCategory};

use super::catalog::{LocalCatalog, LocalCatalogDeps, LocalServer, local_browser, local_tester};
use crate::codec::Codec;
use crate::dialects::local::{LocalDialect, LocalReplay, local_chat};
use crate::models::view::codec_facts_of;
use crate::provider::disposition::disposition_of;
use crate::provider::kind::{
    Browser, CatalogDeps, EntryCatalog, KindDeps, KindError, KindField, ProviderKind, Route, Tester,
};
use crate::transport::{Auth, KeySource, Transport, TransportDeps};

/// What an entry costs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TermsPrice {
    Free,
    Unknown,
}

/// An entry's retention and price.
#[derive(Debug, Clone, PartialEq)]
pub struct Terms {
    /// Data may be kept.
    pub kept: bool,
    pub price: TermsPrice,
}

impl Terms {
    pub fn view_price(&self) -> ModelViewPrice {
        match self.price {
            TermsPrice::Free => ModelViewPrice::Free,
            TermsPrice::Unknown => ModelViewPrice::Unknown,
        }
    }
}

/// Data stays home; calls bill nothing.
pub const HOME: Terms = Terms {
    kept: false,
    price: TermsPrice::Free,
};

/// The count of models a check reads; none when the server is foreign.
pub type Reach =
    fn(Arc<Transport>, CancelToken) -> BoxFuture<'static, Result<Option<usize>, KindError>>;

/// A request root from a URL; refuses one it cannot read.
pub type Base = fn(&str) -> Result<String, String>;

/// One local server type.
pub struct LocalKindSpec {
    pub kind: ProviderKindName,
    pub label: &'static str,
    /// Default URL.
    pub url: Option<&'static str>,
    /// Requests carry a key when set.
    pub optional_key: bool,
    pub replay: Option<LocalReplay>,
    /// Defaults to `/v1/chat/completions`.
    pub chat_path: Option<&'static str>,
    /// Request root from the URL; refuses a URL it cannot read.
    pub base: Option<Base>,
    /// Defaults to `HOME`.
    pub terms: Option<fn(&EntryFields) -> Terms>,
    pub reach: Reach,
    pub server: fn(Arc<Transport>) -> Arc<dyn LocalServer>,
}

const DOT: &str = " \u{b7} ";

struct NoKey;

impl KeySource for NoKey {
    fn get(&self) -> BoxFuture<'_, Option<String>> {
        Box::pin(async { None })
    }
}

struct SecretKey(String);

impl KeySource for SecretKey {
    fn get(&self) -> BoxFuture<'_, Option<String>> {
        Box::pin(async move { Some(self.0.clone()) })
    }
}

/// The text without its trailing slashes.
fn without_slashes(text: &str) -> &str {
    text.trim_end_matches('/')
}

/// A server named by URL.
pub struct LocalKind {
    spec: LocalKindSpec,
}

pub fn local_kind(spec: LocalKindSpec) -> LocalKind {
    LocalKind { spec }
}

impl LocalKind {
    fn base_of(&self, entry: &EntryFields) -> Result<String, String> {
        let url = without_slashes(entry.url.as_deref().or(self.spec.url).unwrap_or_default());
        match self.spec.base {
            Some(base) => base(url),
            None => Ok(url.to_owned()),
        }
    }

    fn dialect(&self) -> LocalDialect {
        LocalDialect {
            label: self.spec.label.to_owned(),
            replay: self.spec.replay,
            chat_path: self.spec.chat_path.map(str::to_owned),
            optional_key: self.spec.optional_key,
        }
    }
}

fn entry_fields(entry: &ProviderEntry) -> EntryFields {
    EntryFields {
        kind: entry.kind,
        label: entry.label.clone(),
        plan: entry.plan,
        url: entry.url.clone(),
        extra_usage: entry.extra_usage,
    }
}

fn catalog_of(route: &Route) -> Option<LocalCatalog> {
    route
        .catalog
        .as_any()
        .downcast_ref::<LocalCatalog>()
        .cloned()
}

impl ProviderKind for LocalKind {
    fn kind(&self) -> ProviderKindName {
        self.spec.kind
    }

    fn label(&self) -> &str {
        self.spec.label
    }

    fn field(&self) -> KindField {
        KindField::Url
    }

    fn optional_key(&self) -> bool {
        self.spec.optional_key
    }

    fn url(&self) -> Option<&str> {
        self.spec.url
    }

    /// URL, then model count.
    fn facts(
        &self,
        state: &KeyState,
        entry: &EntryFields,
        _bill: Option<&Billed>,
    ) -> Option<String> {
        let url = self.base_of(entry).ok()?;
        let KeyState::Ok(KeyStateOk {
            models: Some(models),
            ..
        }) = state
        else {
            return Some(url);
        };
        let count = nyaaai_contract::number::number_text(*models);
        let noun = if *models == 1.0 { "model" } else { "models" };
        Some(format!("{url}{DOT}{count} {noun}"))
    }

    fn check<'a>(
        &'a self,
        entry: &'a EntryFields,
        secret: Option<&'a str>,
        deps: &'a KindDeps,
        cancel: &'a CancelToken,
    ) -> BoxFuture<'a, Result<KeyState, KindError>> {
        Box::pin(async move {
            let key: Arc<dyn KeySource> = match secret {
                None => Arc::new(NoKey),
                Some(secret) => Arc::new(SecretKey(secret.to_owned())),
            };
            let read = match self.transport(entry, key, deps) {
                Ok(transport) => (self.spec.reach)(transport, cancel.clone()).await,
                Err(message) => Err(KindError::Message(message)),
            };
            match read {
                Ok(None) => Ok(KeyState::Unreachable(KeyStateUnreachable {})),
                Ok(Some(models)) => Ok(KeyState::Ok(KeyStateOk {
                    usage: None,
                    limit: None,
                    models: Some(models as f64),
                })),
                Err(error) => {
                    if cancel.is_cancelled() {
                        return Err(KindError::Cancelled);
                    }
                    let refused = matches!(
                        &error,
                        KindError::Provider(held) if matches!(
                            held.failure.category,
                            FailureCategory::Refused
                                | FailureCategory::KeyRejected
                                | FailureCategory::KeyMissing
                        )
                    );
                    Ok(if refused {
                        KeyState::Rejected(KeyStateRejected {})
                    } else {
                        KeyState::Unreachable(KeyStateUnreachable {})
                    })
                }
            }
        })
    }

    fn transport(
        &self,
        entry: &EntryFields,
        key: Arc<dyn KeySource>,
        deps: &KindDeps,
    ) -> Result<Arc<Transport>, String> {
        let base_url = match &deps.base_url {
            Some(base) => base.clone(),
            None => self.base_of(entry)?,
        };
        let log = Arc::clone(&deps.log);
        Ok(Arc::new(Transport::new(TransportDeps {
            fetch: Arc::clone(&deps.fetch),
            scheduler: Arc::clone(&deps.scheduler),
            key,
            log: Box::new(move |text| log(text)),
            base_url,
            label: self.spec.label.to_owned(),
            user_agent: deps.user_agent.clone(),
            errors: Arc::new(local_chat(None, self.spec.label, self.dialect())),
            auth: if self.spec.optional_key {
                Auth::Optional
            } else {
                Auth::None
            },
            idle_ms: None,
            max_bytes: None,
        })))
    }

    fn catalog(
        &self,
        entry: &ProviderEntry,
        transport: Arc<Transport>,
        deps: CatalogDeps,
    ) -> Arc<dyn EntryCatalog> {
        let terms = self
            .spec
            .terms
            .map_or(HOME, |terms| terms(&entry_fields(entry)));
        let catalog = LocalCatalog::new(LocalCatalogDeps {
            entry: entry.id.clone(),
            server: (self.spec.server)(Arc::clone(&transport)),
            terms,
            scheduler: deps.scheduler,
            rows: deps.rows,
            refreshed: deps.refreshed,
        });
        transport.on_settled(catalog.listener());
        Arc::new(catalog)
    }

    fn browse(&self, _entry: &ProviderEntry, route: &Route) -> Option<Box<dyn Browser>> {
        catalog_of(route).map(|catalog| Box::new(local_browser(&catalog)) as Box<dyn Browser>)
    }

    fn tester(&self, _entry: &ProviderEntry, route: &Route) -> Option<Box<dyn Tester>> {
        catalog_of(route).map(|catalog| Box::new(local_tester(&catalog)) as Box<dyn Tester>)
    }

    fn codec(&self, view: Option<&ModelView>, entry: &EntryFields) -> Box<dyn Codec> {
        let facts = codec_facts_of(view);
        Box::new(local_chat(facts.as_ref(), &entry.label, self.dialect()))
    }

    /// `notFound` means unavailable.
    fn failure(&self, failure: &Failure, view: Option<&ModelView>) -> Disposition {
        if failure.category == FailureCategory::NotFound {
            Disposition::FallThrough(DispositionFallThrough {
                notice: NoticeKind::ModelUnavailable,
            })
        } else {
            disposition_of(failure, view)
        }
    }
}
