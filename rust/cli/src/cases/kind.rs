//! Kind cases.

use nyaaai_agentic::kinds::openrouter::key::{KeyCheck, KeyFailure, check_key};
use nyaaai_agentic::transport::{KeyAuth, TransportError};
use nyaaai_testkit::cases::{Outcome, block_on};

use std::sync::Arc;

use nyaaai_agentic::codec::Mode;
use nyaaai_agentic::models::view::{codec_facts_of, unknown_view};
use nyaaai_agentic::provider::kind::{
    CatalogDeps, KINDS, ProviderKind, Route, kind_lines, provider_view as view_of,
};
use nyaaai_contract::model_registry::ModelRegistry;
use nyaaai_contract::shapes::model::{ModelUse, ModelViewPrice, ModelViewStatus};
use nyaaai_contract::shapes::provider::{
    EntryFields, ProviderEntry, ProviderKindName, ProviderPlan,
};
use nyaaai_contract::time::Clock;
use nyaaai_testkit::scheduler::{FakeScheduler, TimerWheel};
use nyaaai_testkit::scripted_ports::ScriptedModelRegistry;

use super::catalog::{
    BASE_URL, CALLER_REASON, caller_token, network_of, row_of_key, rows_of_keys, view_line,
    view_lines,
};
use super::local::{keys_of, message_of, net_of};
use super::shapes::*;

macro_rules! hosted_of {
    ($($shape:ident),* $(,)?) => {$(
        impl From<&$shape> for ProviderKindName {
            fn from(held: &$shape) -> Self {
                match held {
                    $shape::Openrouter => Self::Openrouter,
                    $shape::Opencode => Self::Opencode,
                }
            }
        }
    )*};
}

hosted_of!(
    KindFactsInputKind,
    KindCheckInputKind,
    KindLimitsInputKind,
    KindCodecInputKind,
    KindDispositionInputKind,
    KindRouteInputKind,
);

pub fn openrouter_key(input: KindOpenrouterKeyInput) -> Outcome<KindOpenrouterKeyResult> {
    let net = network_of(input.start_ms, &input.exchanges, input.stored.as_deref())?;
    let mut checks = Vec::new();
    let mut tags = 0;
    for held in &input.checks {
        tags += 1;
        let (auth, after, before) = match held {
            KindOpenrouterKeyInputChecksItem::Stored(held) => {
                (KeyAuth::Stored, held.abort_after_ms, held.aborted_before)
            }
            KindOpenrouterKeyInputChecksItem::Key(held) => (
                KeyAuth::Key(held.key.clone()),
                held.abort_after_ms,
                held.aborted_before,
            ),
        };
        let token = caller_token(&net.wheel, &format!("caller-{tags}"), after, before);
        let result = block_on(check_key(&net.transport, &auth, &token))?;
        checks.push(match result {
            Ok(KeyCheck::Ok {
                usage,
                limit,
                reset,
            }) => KindOpenrouterKeyResultChecksItem {
                ok: Some(true),
                reason: None,
                usage,
                limit,
                reset,
                thrown: None,
            },
            Ok(KeyCheck::Failed(reason)) => KindOpenrouterKeyResultChecksItem {
                ok: Some(false),
                reason: Some(match reason {
                    KeyFailure::Rejected => KindOpenrouterKeyResultChecksItemReason::Rejected,
                    KeyFailure::Unreachable => KindOpenrouterKeyResultChecksItemReason::Unreachable,
                }),
                usage: None,
                limit: None,
                reset: None,
                thrown: None,
            },
            Err(error) => KindOpenrouterKeyResultChecksItem {
                ok: None,
                reason: None,
                usage: None,
                limit: None,
                reset: None,
                thrown: Some(match error {
                    TransportError::Cancelled => CALLER_REASON.to_owned(),
                    TransportError::Provider(held) => held.failure.message.clone(),
                }),
            },
        });
        net.wheel.cancel(&format!("caller-{tags}"));
    }
    Ok(KindOpenrouterKeyResult {
        checks,
        requests: net
            .played
            .requests()
            .iter()
            .map(|sent| KindOpenrouterKeyResultRequestsItem {
                path: sent.url[BASE_URL.len()..].to_owned(),
                authorization: sent
                    .headers
                    .iter()
                    .find(|(name, _)| name == "Authorization")
                    .map(|(_, value)| value.clone()),
            })
            .collect(),
        waiting: net.wheel.waiting(),
    })
}

/// A scheduler that stands at `now_ms` and moves for nobody.
fn standing_at(now_ms: i64) -> Arc<FakeScheduler> {
    Arc::new(FakeScheduler::new(Arc::new(TimerWheel::new(now_ms))))
}

/// A hosted kind of the table.
fn hosted_kind(name: ProviderKindName) -> Arc<dyn ProviderKind> {
    KINDS
        .iter()
        .find(|held| held.kind() == name)
        .map(Arc::clone)
        .expect("the table holds both hosted kinds")
}

fn hosted_entry(name: ProviderKindName, label: &str, plan: Option<ProviderPlan>) -> EntryFields {
    EntryFields {
        kind: name,
        label: label.to_owned(),
        plan,
        url: None,
        extra_usage: None,
    }
}

/// The kinds of a table, in the order given; none given reads `KINDS`.
fn table_of(names: &Option<Vec<ProviderKindName>>) -> Vec<Arc<dyn ProviderKind>> {
    let Some(names) = names else {
        return KINDS.clone();
    };
    let mut table: Vec<Arc<dyn ProviderKind>> = Vec::new();
    for name in names {
        if table.iter().any(|held| held.kind() == *name) {
            continue;
        }
        if let Some(held) = KINDS.iter().find(|held| held.kind() == *name) {
            table.push(Arc::clone(held));
        }
    }
    table
}

pub fn lines(input: KindLinesInput) -> Outcome<KindLinesResult> {
    Ok(KindLinesResult {
        lines: kind_lines(&table_of(&input.kinds)),
    })
}

pub fn provider_view(input: KindProviderViewInput) -> Outcome<KindProviderViewResult> {
    Ok(KindProviderViewResult {
        view: view_of(
            &input.entry,
            &input.status,
            &table_of(&input.kinds),
            input.bill.as_ref(),
        ),
    })
}

pub fn facts(input: KindFactsInput) -> Outcome<KindFactsResult> {
    let name = ProviderKindName::from(&input.kind);
    let named = hosted_kind(name);
    let entry = hosted_entry(name, &input.entry.label, input.entry.plan);
    Ok(KindFactsResult {
        facts: input
            .rows
            .iter()
            .map(|row| named.facts(&row.state, &entry, row.bill.as_ref()))
            .collect(),
    })
}

fn request_lines(
    played: &nyaaai_testkit::scripted_fetch::ScriptedFetch,
) -> Vec<(String, String, Option<String>, Option<String>)> {
    played
        .requests()
        .iter()
        .map(|sent| {
            let header = |name: &str| {
                sent.headers
                    .iter()
                    .find(|(each, _)| each == name)
                    .map(|(_, value)| value.clone())
            };
            (
                match sent.method {
                    nyaaai_contract::fetch::Method::Get => "GET",
                    nyaaai_contract::fetch::Method::Post => "POST",
                }
                .to_owned(),
                sent.url.clone(),
                header("Authorization"),
                header("User-Agent"),
            )
        })
        .collect()
}

pub fn check(input: KindCheckInput) -> Outcome<KindCheckResult> {
    let name = ProviderKindName::from(&input.kind);
    let net = net_of(input.start_ms, &input.exchanges);
    let named = hosted_kind(name);
    let entry = hosted_entry(name, &input.entry.label, input.entry.plan);
    let mut deps = net.deps.clone();
    if let Some(base) = &input.base_url {
        deps.base_url = Some(base.clone());
    }
    let token = caller_token(
        &net.wheel,
        "caller",
        input.abort_after_ms,
        input.aborted_before,
    );
    let checked = block_on(named.check(&entry, input.secret.as_deref(), &deps, &token))?;
    net.wheel.cancel("caller");
    let (state, thrown) = match checked {
        Ok(state) => (Some(state), None),
        Err(error) => (None, Some(message_of(&error))),
    };
    Ok(KindCheckResult {
        facts: state
            .as_ref()
            .and_then(|state| named.facts(state, &entry, None)),
        state,
        thrown,
        requests: request_lines(&net.played)
            .into_iter()
            .map(
                |(method, url, authorization, user_agent)| KindCheckResultRequestsItem {
                    method,
                    url,
                    authorization,
                    user_agent,
                },
            )
            .collect(),
        waiting: net.wheel.waiting(),
    })
}

pub fn limits(input: KindLimitsInput) -> Outcome<KindLimitsResult> {
    let name = ProviderKindName::from(&input.kind);
    let net = net_of(input.start_ms, &input.exchanges);
    let named = hosted_kind(name);
    let entry = hosted_entry(name, &input.entry.label, input.entry.plan);
    let transport = named.transport(&entry, keys_of(&input.stored)?, &net.deps)?;
    let token = caller_token(
        &net.wheel,
        "caller",
        input.abort_after_ms,
        input.aborted_before,
    );
    let present = named.has_limits();
    let limits = if present {
        block_on(named.limits(&transport, &token))?
    } else {
        None
    };
    net.wheel.cancel("caller");
    Ok(KindLimitsResult {
        present,
        limits,
        requests: request_lines(&net.played)
            .into_iter()
            .map(
                |(method, url, authorization, user_agent)| KindLimitsResultRequestsItem {
                    method,
                    url,
                    authorization,
                    user_agent,
                },
            )
            .collect(),
        waiting: net.wheel.waiting(),
    })
}

pub fn codec(input: KindCodecInput) -> Outcome<KindCodecResult> {
    let name = ProviderKindName::from(&input.kind);
    let named = hosted_kind(name);
    let mut deps = net_of(0, &[]).deps;
    deps.scheduler = standing_at(input.now_ms);
    let view = match &input.view {
        None => None,
        Some(held) => {
            let mut view = unknown_view(&row_of_key("openai/gpt-6-luna")?);
            view.status = ModelViewStatus::Ok;
            view.efforts = held.efforts.clone();
            view.api = held.api.clone();
            view.output_limit = held.output_limit;
            view.price = held.price.as_ref().map(|price| match price {
                KindCodecInputViewPrice::Free => ModelViewPrice::Free,
                KindCodecInputViewPrice::Unknown => ModelViewPrice::Unknown,
            });
            Some(view)
        }
    };
    let codec = named.codec(
        view.as_ref(),
        &hosted_entry(name, &input.label, None),
        &deps,
    );
    let facts = codec_facts_of(view.as_ref());
    let mode = match input.mode {
        KindCodecInputMode::Stream => Mode::Stream,
        KindCodecInputMode::Complete => Mode::Complete,
    };
    let (encoded, refusal) = match codec.encode(&input.request, mode) {
        Ok(encoded) => (Some(encoded), None),
        Err(message) => (None, Some(message)),
    };
    Ok(KindCodecResult {
        encoded,
        refusal,
        facts: facts.map(|facts| KindCodecResultFacts {
            efforts: facts.efforts,
            free: facts.free == Some(true),
            output_limit: facts.output_limit.map(|limit| limit as f64),
        }),
        failures: input
            .failures
            .iter()
            .map(|each| {
                codec.failure(
                    each.status,
                    &each.body,
                    &each.headers.clone().unwrap_or_default(),
                )
            })
            .collect(),
    })
}

pub fn disposition(input: KindDispositionInput) -> Outcome<KindDispositionResult> {
    let name = ProviderKindName::from(&input.kind);
    let named = hosted_kind(name);
    let row = row_of_key("openai/gpt-6-luna")?;
    Ok(KindDispositionResult {
        dispositions: input
            .rows
            .iter()
            .map(|each| {
                let view = each.status.as_ref().map(|status| {
                    let mut view = unknown_view(&row);
                    view.status = match status {
                        KindDispositionInputRowsItemStatus::Ok => ModelViewStatus::Ok,
                        KindDispositionInputRowsItemStatus::Retired => ModelViewStatus::Retired,
                        KindDispositionInputRowsItemStatus::NoZdrHost => ModelViewStatus::NoZdrHost,
                        KindDispositionInputRowsItemStatus::NoTools => ModelViewStatus::NoTools,
                        KindDispositionInputRowsItemStatus::Unknown => ModelViewStatus::Unknown,
                    };
                    view
                });
                named.failure(&each.failure, view.as_ref())
            })
            .collect(),
    })
}

type Answer = KindRouteResultStepsItemAnswer;

pub fn route(input: KindRouteInput) -> Outcome<KindRouteResult> {
    let name = ProviderKindName::from(&input.kind);
    let net = network_of(input.start_ms, &input.exchanges, None)?;
    let named = hosted_kind(name);
    let fields = hosted_entry(name, &input.entry.label, input.entry.plan);
    let entry = ProviderEntry {
        id: "e1".to_owned(),
        kind: name,
        label: input.entry.label.clone(),
        plan: input.entry.plan,
        url: None,
        extra_usage: None,
    };
    let mut deps = net_of(input.start_ms, &[]).deps;
    deps.fetch = net.played.clone();
    deps.scheduler = net.scheduler.clone();
    deps.base_url = Some(BASE_URL.to_owned());
    let key = keys_of(&Some("key-0123456789abcdef".to_owned()))?;
    let transport = named.transport(&fields, key, &deps)?;
    let registry = Arc::new(ScriptedModelRegistry::empty());
    registry.set_rows(rows_of_keys(&input.rows)?);
    let catalog = named.catalog(
        &entry,
        Arc::clone(&transport),
        CatalogDeps {
            scheduler: net.scheduler.clone(),
            rows: registry.clone() as Arc<dyn ModelRegistry>,
            refreshed: Arc::new(|| {}),
        },
    );
    let route = Route {
        transport: Arc::clone(&transport),
        catalog: Arc::clone(&catalog),
    };
    let (Some(browser), Some(tester)) = (
        named.browse(&entry, &route, &deps),
        named.tester(&entry, &route, &deps),
    ) else {
        return Err("The kind has no browser or tester.".to_owned());
    };
    let mut steps = Vec::new();
    let mut tags = 0;
    block_on(async {
        for step in &input.steps {
            tags += 1;
            let tag = format!("caller-{tags}");
            let (after, before) = match step {
                KindRouteInputStepsItem::Views(held) => (held.abort_after_ms, held.aborted_before),
                KindRouteInputStepsItem::Detail(held) => (held.abort_after_ms, held.aborted_before),
                KindRouteInputStepsItem::Listed(held) => (held.abort_after_ms, held.aborted_before),
                KindRouteInputStepsItem::View(held) => (held.abort_after_ms, held.aborted_before),
                _ => (None, None),
            };
            let token = caller_token(&net.wheel, &tag, after, before);
            let thrown =
                |message: String| Answer::Thrown(KindRouteResultStepsItemAnswerThrown { message });
            let answer: Answer = match step {
                KindRouteInputStepsItem::Views(held) => {
                    match catalog
                        .views(
                            held.use_.unwrap_or(ModelUse::Chat),
                            held.zdr_only.unwrap_or(true),
                            Some(&token),
                        )
                        .await
                    {
                        Ok(views) => match view_lines(&views) {
                            Ok(views) => {
                                Answer::Views(KindRouteResultStepsItemAnswerViews { views })
                            }
                            Err(message) => thrown(message),
                        },
                        Err(_) => thrown(CALLER_REASON.to_owned()),
                    }
                }
                KindRouteInputStepsItem::List(held) => {
                    match browser.list(held.fresh.unwrap_or(false)).await {
                        Ok(models) => Answer::List(KindRouteResultStepsItemAnswerList { models }),
                        Err(error) => thrown(message_of(&error)),
                    }
                }
                KindRouteInputStepsItem::Detail(held) => {
                    match browser.detail(&held.id, &token).await {
                        Ok(detail) => {
                            Answer::Detail(KindRouteResultStepsItemAnswerDetail { detail })
                        }
                        Err(error) => thrown(message_of(&error)),
                    }
                }
                KindRouteInputStepsItem::Listed(held) => {
                    match tester.listed(&held.id, &token).await {
                        Ok(listed) => Answer::Listed(KindRouteResultStepsItemAnswerListed {
                            listed: listed.map(|held| KindRouteResultStepsItemAnswerListedListed {
                                label: held.label,
                                reasoning: held.reasoning,
                            }),
                        }),
                        Err(error) => thrown(message_of(&error)),
                    }
                }
                KindRouteInputStepsItem::View(held) => {
                    let row = rows_of_keys(std::slice::from_ref(&held.row))?
                        .into_iter()
                        .next()
                        .ok_or_else(|| "no row".to_owned())?;
                    match tester
                        .view(&row, held.use_.unwrap_or(ModelUse::Chat), &token)
                        .await
                    {
                        Ok(view) => match view_line(&view) {
                            Ok(view) => Answer::View(KindRouteResultStepsItemAnswerView { view }),
                            Err(message) => thrown(message),
                        },
                        Err(error) => thrown(message_of(&error)),
                    }
                }
                KindRouteInputStepsItem::Advance(held) => {
                    net.wheel.advance(held.ms as i64);
                    Answer::Advanced(KindRouteResultStepsItemAnswerAdvanced {})
                }
            };
            net.wheel.cancel(&tag);
            steps.push(KindRouteResultStepsItem {
                now_ms: Clock::now_ms(&*net.wheel),
                answer,
            });
        }
        Ok::<(), String>(())
    })??;
    Ok(KindRouteResult {
        steps,
        requests: net.asked(),
        read: net
            .played
            .read()
            .into_iter()
            .map(|read| read as i64)
            .collect(),
        waiting: net.wheel.waiting(),
    })
}
