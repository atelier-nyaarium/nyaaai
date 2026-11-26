//! The OpenCode kind.

use std::sync::Arc;

use nyaaai_contract::fetch::Span;
use nyaaai_contract::future::BoxFuture;
use nyaaai_contract::number::fixed_text;
use nyaaai_contract::shapes::model::ModelView;
use nyaaai_contract::shapes::provider::{
    Billed, Disposition, DispositionFallThrough, EntryFields, KeyState, KeyStateNone, KeyStateOk,
    KeyStateRejected, KeyStateUnreachable, NoticeKind, ProviderEntry, ProviderKindName,
    ProviderPlan,
};
use nyaaai_contract::shapes::step::{ApiFamily, Failure, FailureCategory};
use nyaaai_contract::time::{Clock, Scheduler};

use super::catalog::{OpenCodeCatalog, OpenCodeCatalogDeps, opencode_browser, opencode_tester};
use crate::codec::Codec;
use crate::dialects::opencode::{opencode_chat, opencode_errors};
use crate::gemini::codec::gemini_codec;
use crate::hosts::opencode::opencode_host;
use crate::messages::codec::messages_codec;
use crate::models::view::codec_facts_of;
use crate::provider::disposition::disposition_of;
use crate::provider::kind::{
    Browser, CatalogDeps, EntryCatalog, KindDeps, KindError, KindField, ProviderKind, Route, Tester,
};
use crate::responses::codec::responses_codec;
use crate::transport::{KeyAuth, KeySource, Transport, TransportDeps, static_key};

const PLANS: [(ProviderPlan, &str); 2] = [
    (ProviderPlan::Go, "OpenCode Go"),
    (ProviderPlan::Payg, "OpenCode (pay-as-you-go)"),
];

const EXTRA_USAGE: [ProviderPlan; 1] = [ProviderPlan::Go];

const DOT: &str = " \u{b7} ";

/// A scheduler read as a clock.
#[derive(Clone)]
struct ArcClock(Arc<dyn Scheduler>);

impl Clock for ArcClock {
    fn now_ms(&self) -> i64 {
        self.0.now_ms()
    }

    fn utc_offset_minutes(&self, at_ms: i64) -> i64 {
        self.0.utc_offset_minutes(at_ms)
    }
}

/// Missing plan: Go.
fn plan_of(entry: &EntryFields) -> ProviderPlan {
    entry.plan.unwrap_or(ProviderPlan::Go)
}

fn label_of(entry: &EntryFields) -> &'static str {
    PLANS
        .iter()
        .find(|(plan, _)| *plan == plan_of(entry))
        .map_or("OpenCode", |(_, label)| label)
}

fn base_url_of(plan: ProviderPlan) -> &'static str {
    match plan {
        ProviderPlan::Go => "https://opencode.ai/inference/go",
        ProviderPlan::Payg => "https://opencode.ai/inference",
    }
}

fn catalog_of(route: &Route) -> Option<Arc<OpenCodeCatalog>> {
    Arc::clone(&route.catalog)
        .arc_any()
        .downcast::<OpenCodeCatalog>()
        .ok()
}

fn usd(value: f64) -> String {
    format!("${}", fixed_text(value, 2))
}

/// Extra usage is Go-only.
fn billing_facts(entry: &EntryFields, bill: Option<&Billed>) -> Option<String> {
    let bill = bill?;
    let spent = bill.cost.unwrap_or(0.0);
    if plan_of(entry) == ProviderPlan::Payg {
        return bill.cost.map(|_| format!("{} spent", usd(spent)));
    }
    let mut facts = Vec::new();
    if let Some(allowance) = bill.allowance {
        facts.push(format!("Allowance {} used", usd(allowance)));
    }
    if bill.billing == "balance" || spent > 0.0 {
        facts.push(format!("Extra usage {}", usd(spent)));
    }
    (!facts.is_empty()).then(|| facts.join(DOT))
}

/// The OpenCode kind; every date comes from the route's deps.
pub struct OpenCodeKind;

impl ProviderKind for OpenCodeKind {
    fn kind(&self) -> ProviderKindName {
        ProviderKindName::Opencode
    }

    fn label(&self) -> &str {
        "OpenCode"
    }

    fn plans(&self) -> &[(ProviderPlan, &'static str)] {
        &PLANS
    }

    fn field(&self) -> KindField {
        KindField::Key
    }

    fn balance(&self) -> Option<&str> {
        Some("https://opencode.ai/console")
    }

    fn extra_usage(&self) -> &[ProviderPlan] {
        &EXTRA_USAGE
    }

    fn facts(
        &self,
        _state: &KeyState,
        entry: &EntryFields,
        bill: Option<&Billed>,
    ) -> Option<String> {
        billing_facts(entry, bill)
    }

    fn check<'a>(
        &'a self,
        entry: &'a EntryFields,
        secret: Option<&'a str>,
        deps: &'a KindDeps,
        span: &'a Span,
    ) -> BoxFuture<'a, Result<KeyState, KindError>> {
        Box::pin(async move {
            let Some(secret) = secret else {
                return Ok(KeyState::None(KeyStateNone {}));
            };
            let transport = self
                .transport(entry, Arc::new(static_key(secret)), deps)
                .map_err(KindError::Message)?;
            // 200 clears rejection only.
            match transport
                .get_json("/v1/models", span, &KeyAuth::Stored)
                .await
            {
                Ok(_) => Ok(KeyState::Ok(KeyStateOk {
                    usage: None,
                    limit: None,
                    models: None,
                })),
                Err(error) => {
                    if span.token().is_cancelled() {
                        return Err(error.into());
                    }
                    let rejected = matches!(
                        &error,
                        crate::transport::TransportError::Provider(held)
                            if matches!(
                                held.failure.category,
                                FailureCategory::KeyRejected | FailureCategory::KeyMissing
                            )
                    );
                    Ok(if rejected {
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
        let log = Arc::clone(&deps.log);
        Ok(Arc::new(Transport::new(TransportDeps {
            fetch: Arc::clone(&deps.fetch),
            scheduler: Arc::clone(&deps.scheduler),
            key,
            log: Box::new(move |text| log(text)),
            base_url: deps
                .base_url
                .clone()
                .unwrap_or_else(|| base_url_of(plan_of(entry)).to_owned()),
            label: label_of(entry).to_owned(),
            user_agent: deps.user_agent.clone(),
            errors: Arc::new(opencode_errors(ArcClock(Arc::clone(&deps.scheduler)))),
            auth: crate::transport::Auth::Bearer,
            budgets: deps.budgets.clone(),
        })))
    }

    fn catalog(
        &self,
        entry: &ProviderEntry,
        transport: Arc<Transport>,
        deps: CatalogDeps,
    ) -> Arc<dyn EntryCatalog> {
        Arc::new(OpenCodeCatalog::new(OpenCodeCatalogDeps {
            plan: entry.plan.unwrap_or(ProviderPlan::Go),
            transport,
            deps,
            ttl_ms: None,
        }))
    }

    fn browse(
        &self,
        entry: &ProviderEntry,
        route: &Route,
        deps: &KindDeps,
    ) -> Option<Box<dyn Browser>> {
        let catalog = catalog_of(route)?;
        Some(Box::new(opencode_browser(
            catalog,
            entry.plan.unwrap_or(ProviderPlan::Go),
            Arc::clone(&deps.scheduler),
        )))
    }

    fn tester(
        &self,
        _entry: &ProviderEntry,
        route: &Route,
        _deps: &KindDeps,
    ) -> Option<Box<dyn Tester>> {
        Some(Box::new(opencode_tester(catalog_of(route)?)))
    }

    /// Unknown views use Chat.
    fn codec(
        &self,
        view: Option<&ModelView>,
        _entry: &EntryFields,
        deps: &KindDeps,
    ) -> Box<dyn Codec> {
        let facts = codec_facts_of(view);
        let clock = || ArcClock(Arc::clone(&deps.scheduler));
        match view
            .and_then(|held| held.api.as_ref())
            .map(|api| api.family)
        {
            Some(ApiFamily::Responses) => Box::new(responses_codec(
                opencode_host("/openai/v1", clock()),
                facts.as_ref(),
            )),
            Some(ApiFamily::Messages) => Box::new(messages_codec(
                opencode_host("/anthropic/v1", clock()),
                facts.as_ref(),
            )),
            Some(ApiFamily::GenerateContent) => Box::new(gemini_codec(
                opencode_host("/google/v1beta", clock()),
                facts.as_ref(),
            )),
            _ => Box::new(opencode_chat(facts.as_ref(), clock())),
        }
    }

    /// `notFound` means unavailable.
    fn failure(&self, failure: &Failure, view: Option<&ModelView>) -> Disposition {
        match disposition_of(failure, view) {
            Disposition::FallThrough(held) if held.notice == NoticeKind::BadRequest => {
                Disposition::FallThrough(DispositionFallThrough {
                    notice: NoticeKind::ModelUnavailable,
                })
            }
            other => other,
        }
    }
}

pub static OPENCODE: OpenCodeKind = OpenCodeKind;
