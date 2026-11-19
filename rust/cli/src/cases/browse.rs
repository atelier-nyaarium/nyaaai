//! Browse cases.

use std::sync::Arc;

use nyaaai_agentic::codec::Mode;
use nyaaai_agentic::dialects::openrouter::OPENROUTER_CHAT;
use nyaaai_agentic::kinds::opencode::catalog::{
    OpenCodeCatalog, OpenCodeCatalogDeps, opencode_browser as make_opencode_browser,
    opencode_tester,
};
use nyaaai_agentic::kinds::openrouter::browse::{browse_models, parse_providers};
use nyaaai_agentic::kinds::openrouter::browser::{open_router_browser, open_router_tester};
use nyaaai_agentic::kinds::openrouter::catalog::{
    CatalogListing, Endpoint, Fetched, ModelCatalog, ModelCatalogDeps, Ordered, parse_endpoints,
};
use nyaaai_agentic::models::view::unknown_view;
use nyaaai_agentic::provider::kind::{Browser, CatalogDeps, KindError, Tester};
use nyaaai_contract::json::Value;
use nyaaai_contract::model_registry::ModelRegistry;
use nyaaai_contract::shapes::provider::ProviderPlan;
use nyaaai_testkit::cases::{Outcome, block_on};
use nyaaai_testkit::scripted_ports::ScriptedModelRegistry;

use super::catalog::{
    CALLER_REASON, Net, caller_token, convert, network_of, row_of_key, view_line,
};
use nyaaai_agentic::models::uses::CHAT;

use super::shapes::*;

pub fn openrouter_models(
    input: BrowseOpenrouterModelsInput,
) -> Outcome<BrowseOpenrouterModelsResult> {
    let endpoints = parse_endpoints(&input.zdr).unwrap_or_default();
    let listing = CatalogListing {
        fetched: Arc::new(Fetched {
            entries: Ordered::new(),
            endpoints,
            raw: input.raw,
        }),
    };
    Ok(BrowseOpenrouterModelsResult {
        models: browse_models(&listing, input.now_ms),
    })
}

pub fn openrouter_providers(
    input: BrowseOpenrouterProvidersInput,
) -> Outcome<BrowseOpenrouterProvidersResult> {
    let zdr: Vec<Endpoint> = parse_endpoints(&input.zdr)
        .map(|parsed| {
            parsed
                .iter()
                .flat_map(|(_, hosts)| hosts.iter().cloned())
                .collect()
        })
        .unwrap_or_default();
    Ok(BrowseOpenrouterProvidersResult {
        providers: parse_providers(&input.raw, &zdr),
    })
}

fn message_of(error: &KindError) -> String {
    match error {
        KindError::Cancelled => CALLER_REASON.to_owned(),
        KindError::TimedOut => "The operation timed out.".to_owned(),
        KindError::Provider(held) => held.failure.message.clone(),
        KindError::Message(text) => text.clone(),
    }
}

fn empty_registry() -> Arc<ScriptedModelRegistry> {
    Arc::new(ScriptedModelRegistry::empty())
}

/// Plays the steps on a browser and tester over `net`.
async fn drive(
    net: &Net,
    browser: &dyn Browser,
    tester: &dyn Tester,
    steps: &[BrowseOpenrouterBrowserInputStepsItem],
) -> Outcome<BrowseOpenrouterBrowserResult> {
    type Step = BrowseOpenrouterBrowserInputStepsItem;
    type Answer = BrowseOpenrouterBrowserResultStepsItemAnswer;
    let mut outcomes = Vec::new();
    let mut tags = 0;
    for step in steps {
        tags += 1;
        let tag = format!("caller-{tags}");
        let (after, before) = match step {
            Step::Detail(held) => (held.abort_after_ms, held.aborted_before),
            Step::Listed(held) => (held.abort_after_ms, held.aborted_before),
            Step::View(held) => (held.abort_after_ms, held.aborted_before),
            _ => (None, None),
        };
        let token = caller_token(&net.wheel, &tag, after, before);
        let answer: Result<Answer, KindError> = match step {
            Step::List(held) => browser
                .list(held.fresh.unwrap_or(false))
                .await
                .map(|models| {
                    Answer::List(BrowseOpenrouterBrowserResultStepsItemAnswerList { models })
                }),
            Step::Detail(held) => browser.detail(&held.id, &token).await.map(|detail| {
                Answer::Detail(BrowseOpenrouterBrowserResultStepsItemAnswerDetail { detail })
            }),
            Step::Listed(held) => tester.listed(&held.id, &token).await.map(|listed| {
                Answer::Listed(BrowseOpenrouterBrowserResultStepsItemAnswerListed {
                    listed: listed.map(|held| {
                        BrowseOpenrouterBrowserResultStepsItemAnswerListedListed {
                            label: held.label,
                            reasoning: held.reasoning,
                        }
                    }),
                })
            }),
            Step::View(held) => {
                let row = row_of_key(&held.row)?;
                let use_ = held.use_.clone().unwrap_or_else(|| CHAT.clone());
                match tester.view(&row, use_, &token).await {
                    Ok(view) => Ok(Answer::View(
                        BrowseOpenrouterBrowserResultStepsItemAnswerView {
                            view: view_line(&view)?,
                        },
                    )),
                    Err(error) => Err(error),
                }
            }
            Step::Advance(held) => {
                net.wheel.advance(held.ms as i64);
                Ok(Answer::Advanced(
                    BrowseOpenrouterBrowserResultStepsItemAnswerAdvanced {},
                ))
            }
        };
        let answer = answer.unwrap_or_else(|error| {
            Answer::Thrown(BrowseOpenrouterBrowserResultStepsItemAnswerThrown {
                message: message_of(&error),
            })
        });
        net.wheel.cancel(&tag);
        outcomes.push(BrowseOpenrouterBrowserResultStepsItem {
            now_ms: nyaaai_contract::time::Clock::now_ms(&*net.wheel),
            answer,
        });
    }
    Ok(BrowseOpenrouterBrowserResult {
        steps: outcomes,
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

fn catalog_of(net: &Net) -> Arc<ModelCatalog> {
    Arc::new(ModelCatalog::new(ModelCatalogDeps {
        transport: Arc::clone(&net.transport),
        deps: CatalogDeps {
            scheduler: net.scheduler.clone(),
            rows: empty_registry() as Arc<dyn ModelRegistry>,
            refreshed: Arc::new(|| {}),
        },
        ttl_ms: None,
    }))
}

pub fn openrouter_browser(
    input: BrowseOpenrouterBrowserInput,
) -> Outcome<BrowseOpenrouterBrowserResult> {
    let net = network_of(input.start_ms, &input.exchanges, None)?;
    let catalog = catalog_of(&net);
    let browser = open_router_browser(
        Arc::clone(&catalog),
        Arc::clone(&net.transport),
        net.scheduler.clone(),
    );
    let tester = open_router_tester(catalog, Arc::clone(&net.transport), net.scheduler.clone());
    block_on(drive(&net, &browser, &tester, &input.steps))?
}

pub fn openrouter_hop(input: BrowseOpenrouterHopInput) -> Outcome<BrowseOpenrouterHopResult> {
    let net = network_of(input.start_ms, &input.exchanges, None)?;
    let catalog = catalog_of(&net);
    let tester = open_router_tester(catalog, Arc::clone(&net.transport), net.scheduler.clone());
    let pins = tester
        .pins()
        .ok_or_else(|| "The tester has no pins.".to_owned())?;
    let mut view = unknown_view(&row_of_key("openai/gpt-6-luna")?);
    view.status = nyaaai_contract::shapes::model::ModelViewStatus::Ok;
    view.hosts = input.view_hosts.clone();
    let token = caller_token(
        &net.wheel,
        "caller",
        input.abort_after_ms,
        input.aborted_before,
    );
    let outcome = block_on(async { pins.hop(&input.id, &view, &token).await })?;
    net.wheel.cancel("caller");
    let (hosts, thrown) = match outcome {
        Ok(hosts) => (Some(hosts), None),
        Err(error) => (None, Some(message_of(&error))),
    };
    Ok(BrowseOpenrouterHopResult {
        hosts,
        thrown,
        requests: net.asked(),
        waiting: net.wheel.waiting(),
        now_ms: nyaaai_contract::time::Clock::now_ms(&*net.wheel),
    })
}

pub fn openrouter_pins(input: BrowseOpenrouterPinsInput) -> Outcome<BrowseOpenrouterPinsResult> {
    let net = network_of(0, &[], None)?;
    let catalog = catalog_of(&net);
    let tester = open_router_tester(catalog, Arc::clone(&net.transport), net.scheduler.clone());
    let pins = tester
        .pins()
        .ok_or_else(|| "The tester has no pins.".to_owned())?;
    let codec = pins.wrap(Box::new(OPENROUTER_CHAT));
    let mut answers = Vec::new();
    for call in &input.calls {
        answers.push(match call {
            BrowseOpenrouterPinsInputCallsItem::Pin(held) => {
                pins.pin(&held.hosts);
                BrowseOpenrouterPinsResultAnswersItem::Pinned(
                    BrowseOpenrouterPinsResultAnswersItemPinned {},
                )
            }
            BrowseOpenrouterPinsInputCallsItem::Encode(held) => {
                let mode = match held.mode {
                    BrowseOpenrouterPinsInputCallsItemEncodeMode::Stream => Mode::Stream,
                    BrowseOpenrouterPinsInputCallsItemEncodeMode::Complete => Mode::Complete,
                };
                let encoded = codec.encode(&held.request, mode)?;
                let replayed = encoded
                    .body
                    .get("messages")
                    .and_then(Value::as_array)
                    .is_some_and(|messages| {
                        messages.iter().any(|message| {
                            message
                                .get("reasoning_details")
                                .and_then(Value::as_array)
                                .is_some_and(|held| !held.is_empty())
                        })
                    });
                BrowseOpenrouterPinsResultAnswersItem::Encoded(
                    BrowseOpenrouterPinsResultAnswersItemEncoded {
                        provider: encoded.body.get("provider").cloned().unwrap_or(Value::Null),
                        replayed,
                    },
                )
            }
            BrowseOpenrouterPinsInputCallsItem::PassedBack(_) => {
                BrowseOpenrouterPinsResultAnswersItem::PassedBack(
                    BrowseOpenrouterPinsResultAnswersItemPassedBack {
                        value: pins.passed_back(),
                    },
                )
            }
        });
    }
    Ok(BrowseOpenrouterPinsResult { answers })
}

pub fn opencode_browser(input: BrowseOpencodeBrowserInput) -> Outcome<BrowseOpencodeBrowserResult> {
    let net = network_of(input.start_ms, &input.exchanges, None)?;
    let plan: ProviderPlan = input.plan;
    let catalog = Arc::new(OpenCodeCatalog::new(OpenCodeCatalogDeps {
        plan,
        transport: Arc::clone(&net.transport),
        deps: CatalogDeps {
            scheduler: net.scheduler.clone(),
            rows: empty_registry() as Arc<dyn ModelRegistry>,
            refreshed: Arc::new(|| {}),
        },
        ttl_ms: None,
    }));
    let browser = make_opencode_browser(Arc::clone(&catalog), plan, net.scheduler.clone());
    let tester = opencode_tester(catalog);
    let steps = input
        .steps
        .iter()
        .map(convert::<_, BrowseOpenrouterBrowserInputStepsItem>)
        .collect::<Outcome<Vec<_>>>()?;
    let driven = block_on(drive(&net, &browser, &tester, &steps))??;
    Ok(BrowseOpencodeBrowserResult {
        steps: driven
            .steps
            .iter()
            .map(|step| {
                Ok(BrowseOpencodeBrowserResultStepsItem {
                    now_ms: step.now_ms,
                    answer: convert(&step.answer)?,
                })
            })
            .collect::<Outcome<Vec<_>>>()?,
        requests: driven.requests,
        read: driven.read,
        waiting: driven.waiting,
    })
}
