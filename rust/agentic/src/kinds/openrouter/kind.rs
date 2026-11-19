//! The OpenRouter kind.

use std::sync::Arc;

use nyaaai_contract::cancel::CancelToken;
use nyaaai_contract::future::BoxFuture;
use nyaaai_contract::number::fixed_text;
use nyaaai_contract::shapes::model::ModelView;
use nyaaai_contract::shapes::provider::{
    Billed, Disposition, EntryFields, KeyState, KeyStateNone, KeyStateOk, KeyStateRejected,
    KeyStateUnreachable, Limits, ProviderEntry, ProviderKindName,
};
use nyaaai_contract::shapes::step::Failure;

use super::browser::{open_router_browser, open_router_tester};
use super::catalog::{ModelCatalog, ModelCatalogDeps};
use super::key::{KeyCheck, KeyFailure, check_key};
use crate::codec::Codec;
use crate::dialects::openrouter::OPENROUTER_CHAT;
use crate::provider::disposition::disposition_of;
use crate::provider::kind::{
    Browser, CatalogDeps, EntryCatalog, KindDeps, KindError, KindField, ProviderKind, Route, Tester,
};
use crate::transport::{KeyAuth, KeySource, Transport, TransportDeps, static_key};

/// The entry scripts and tests use.
pub fn openrouter_entry() -> EntryFields {
    EntryFields {
        kind: ProviderKindName::Openrouter,
        label: "OpenRouter".to_owned(),
        plan: None,
        url: None,
        extra_usage: None,
    }
}

const BASE_URL: &str = "https://openrouter.ai/api/v1";

const DOT: &str = " \u{b7} ";

fn usd(value: f64) -> String {
    format!("${}", fixed_text(value, 2))
}

/// No limit means unlimited.
fn usage_facts(state: &KeyState) -> Option<String> {
    let KeyState::Ok(KeyStateOk {
        usage: Some(usage),
        limit,
        ..
    }) = state
    else {
        return None;
    };
    let used = format!("{} used", usd(*usage));
    Some(match limit {
        None => used,
        Some(limit) => format!("{used}{DOT}{} key limit", usd(*limit)),
    })
}

fn key_state_of(check: KeyCheck) -> KeyState {
    match check {
        KeyCheck::Failed(KeyFailure::Rejected) => KeyState::Rejected(KeyStateRejected {}),
        KeyCheck::Failed(KeyFailure::Unreachable) => KeyState::Unreachable(KeyStateUnreachable {}),
        KeyCheck::Ok { usage, limit, .. } => KeyState::Ok(KeyStateOk {
            usage,
            limit,
            models: None,
        }),
    }
}

/// Every OpenRouter route holds one.
fn catalog_of(route: &Route) -> Option<Arc<ModelCatalog>> {
    Arc::clone(&route.catalog)
        .arc_any()
        .downcast::<ModelCatalog>()
        .ok()
}

/// The OpenRouter kind; it reads no clock of its own.
pub struct OpenRouterKind;

impl ProviderKind for OpenRouterKind {
    fn kind(&self) -> ProviderKindName {
        ProviderKindName::Openrouter
    }

    fn label(&self) -> &str {
        "OpenRouter"
    }

    fn field(&self) -> KindField {
        KindField::Key
    }

    fn balance(&self) -> Option<&str> {
        Some("https://openrouter.ai/settings/credits")
    }

    fn stats(&self) -> bool {
        true
    }

    fn facts(
        &self,
        state: &KeyState,
        _entry: &EntryFields,
        _bill: Option<&Billed>,
    ) -> Option<String> {
        usage_facts(state)
    }

    fn check<'a>(
        &'a self,
        entry: &'a EntryFields,
        secret: Option<&'a str>,
        deps: &'a KindDeps,
        cancel: &'a CancelToken,
    ) -> BoxFuture<'a, Result<KeyState, KindError>> {
        Box::pin(async move {
            let Some(secret) = secret else {
                return Ok(KeyState::None(KeyStateNone {}));
            };
            let transport = self
                .transport(entry, Arc::new(static_key(secret)), deps)
                .map_err(KindError::Message)?;
            let checked = check_key(&transport, &KeyAuth::Stored, cancel).await?;
            Ok(key_state_of(checked))
        })
    }

    fn transport(
        &self,
        _entry: &EntryFields,
        key: Arc<dyn KeySource>,
        deps: &KindDeps,
    ) -> Result<Arc<Transport>, String> {
        let log = Arc::clone(&deps.log);
        Ok(Arc::new(Transport::new(TransportDeps {
            fetch: Arc::clone(&deps.fetch),
            scheduler: Arc::clone(&deps.scheduler),
            key,
            log: Box::new(move |text| log(text)),
            base_url: deps.base_url.clone().unwrap_or_else(|| BASE_URL.to_owned()),
            label: "OpenRouter".to_owned(),
            user_agent: deps.user_agent.clone(),
            errors: Arc::new(OPENROUTER_CHAT),
            auth: crate::transport::Auth::Bearer,
            idle_ms: None,
            max_bytes: None,
        })))
    }

    fn catalog(
        &self,
        _entry: &ProviderEntry,
        transport: Arc<Transport>,
        deps: CatalogDeps,
    ) -> Arc<dyn EntryCatalog> {
        Arc::new(ModelCatalog::new(ModelCatalogDeps {
            transport,
            deps,
            ttl_ms: None,
        }))
    }

    fn browse(
        &self,
        _entry: &ProviderEntry,
        route: &Route,
        deps: &KindDeps,
    ) -> Option<Box<dyn Browser>> {
        let catalog = catalog_of(route)?;
        Some(Box::new(open_router_browser(
            catalog,
            Arc::clone(&route.transport),
            Arc::clone(&deps.scheduler),
        )))
    }

    fn tester(
        &self,
        _entry: &ProviderEntry,
        route: &Route,
        deps: &KindDeps,
    ) -> Option<Box<dyn Tester>> {
        let catalog = catalog_of(route)?;
        Some(Box::new(open_router_tester(
            catalog,
            Arc::clone(&route.transport),
            Arc::clone(&deps.scheduler),
        )))
    }

    fn codec(
        &self,
        _view: Option<&ModelView>,
        _entry: &EntryFields,
        _deps: &KindDeps,
    ) -> Box<dyn Codec> {
        Box::new(OPENROUTER_CHAT)
    }

    fn failure(&self, failure: &Failure, view: Option<&ModelView>) -> Disposition {
        disposition_of(failure, view)
    }

    fn has_limits(&self) -> bool {
        true
    }

    fn limits<'a>(
        &'a self,
        transport: &'a Transport,
        cancel: &'a CancelToken,
    ) -> BoxFuture<'a, Option<Limits>> {
        Box::pin(async move {
            match check_key(transport, &KeyAuth::Stored, cancel).await {
                Ok(KeyCheck::Ok { limit, reset, .. }) => Some(Limits { limit, reset }),
                _ => None,
            }
        })
    }
}

pub static OPENROUTER: OpenRouterKind = OpenRouterKind;
