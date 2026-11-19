//! OpenRouter model browser and tester.

use std::sync::{Arc, Mutex, MutexGuard};

use nyaaai_contract::cancel::CancelToken;
use nyaaai_contract::future::BoxFuture;
use nyaaai_contract::json::{Object, Value};
use nyaaai_contract::shapes::model::{BrowseDetail, BrowseModel, ModelRow, ModelUse, ModelView};
use nyaaai_contract::shapes::step::{CanonicalRequest, Encoded, Failure, HeaderList, StepResult};
use nyaaai_contract::text::{ascii_lower, encode_path_segment, trim_text};
use nyaaai_contract::time::{Ended, Scheduler, deadline};

use super::browse::{browse_models, parse_providers};
use super::catalog::ModelCatalog;
use crate::codec::{Codec, Mode, StreamDecoder};
use crate::failure::ProviderError;
use crate::kinds::fetcher::FETCH_MS;
use crate::provider::kind::{Browser, KindError, Listed, Pins, Tester};
use crate::transport::{KeyAuth, Transport};

const DETAIL_MS: i64 = FETCH_MS;

/// Provider filter tag, the one rule for a tag and a host name.
fn slug_of(tag: &str) -> String {
    ascii_lower(trim_text(tag.split('/').next().unwrap_or(tag)))
}

/// A path segment; the dot segments are written out so none climbs a path.
fn segment_of(part: &str) -> String {
    match part {
        "." => "%2E".to_owned(),
        ".." => "%2E%2E".to_owned(),
        other => encode_path_segment(other),
    }
}

/// An id's endpoints path, its slashes kept.
pub fn endpoints_path(id: &str) -> String {
    let parts: Vec<String> = id.split('/').map(segment_of).collect();
    format!("/models/{}/endpoints", parts.join("/"))
}

/// Catalog and provider endpoints.
pub struct OpenRouterBrowser {
    catalog: Arc<ModelCatalog>,
    transport: Arc<Transport>,
    clock: Arc<dyn Scheduler>,
}

pub fn open_router_browser(
    catalog: Arc<ModelCatalog>,
    transport: Arc<Transport>,
    clock: Arc<dyn Scheduler>,
) -> OpenRouterBrowser {
    OpenRouterBrowser {
        catalog,
        transport,
        clock,
    }
}

impl Browser for OpenRouterBrowser {
    fn list(&self, fresh: bool) -> BoxFuture<'_, Result<Option<Vec<BrowseModel>>, KindError>> {
        let listing = self.catalog.listing(None, fresh);
        Box::pin(async move {
            let listing = listing.await?;
            Ok(listing.map(|held| browse_models(&held, self.clock.now_ms())))
        })
    }

    fn detail<'a>(
        &'a self,
        id: &'a str,
        cancel: &'a CancelToken,
    ) -> BoxFuture<'a, Result<Option<BrowseDetail>, KindError>> {
        let listing = self.catalog.listing(Some(cancel), false);
        Box::pin(async move {
            let listing = listing.await?;
            let raw = self
                .transport
                .get_json(&endpoints_path(id), cancel, &KeyAuth::None)
                .await?;
            let known: Vec<_> = listing
                .as_ref()
                .and_then(|held| held.endpoints().get(id))
                .cloned()
                .unwrap_or_default();
            Ok(parse_providers(&raw, &known).map(|hosts| BrowseDetail {
                hosts: Some(hosts),
                max_output: None,
                efforts: None,
                choice: None,
                cache: None,
                retention: None,
            }))
        })
    }
}

pub struct OpenRouterTester {
    catalog: Arc<ModelCatalog>,
    transport: Arc<Transport>,
    scheduler: Arc<dyn Scheduler>,
}

pub fn open_router_tester(
    catalog: Arc<ModelCatalog>,
    transport: Arc<Transport>,
    scheduler: Arc<dyn Scheduler>,
) -> OpenRouterTester {
    OpenRouterTester {
        catalog,
        transport,
        scheduler,
    }
}

impl Tester for OpenRouterTester {
    fn listed<'a>(
        &'a self,
        id: &'a str,
        cancel: &'a CancelToken,
    ) -> BoxFuture<'a, Result<Option<Listed>, KindError>> {
        let entry = self.catalog.entry(id, Some(cancel));
        Box::pin(async move {
            let Some(entry) = entry.await? else {
                return Ok(None);
            };
            let efforts = entry
                .reasoning
                .and_then(|held| held.supported_efforts)
                .unwrap_or_default();
            Ok(Some(Listed {
                label: entry.name.unwrap_or_else(|| id.to_owned()),
                reasoning: !efforts.is_empty(),
            }))
        })
    }

    fn view<'a>(
        &'a self,
        row: &'a ModelRow,
        use_: ModelUse,
        cancel: &'a CancelToken,
    ) -> BoxFuture<'a, Result<ModelView, KindError>> {
        let view = self.catalog.view_for(row, use_, Some(cancel));
        Box::pin(async move { Ok(view.await?) })
    }

    fn pins(&self) -> Option<Box<dyn Pins>> {
        Some(Box::new(OpenRouterPins {
            transport: Arc::clone(&self.transport),
            scheduler: Arc::clone(&self.scheduler),
            state: Arc::new(Mutex::new(PinState::default())),
        }))
    }
}

#[derive(Default)]
struct PinState {
    hosts: Vec<String>,
    step: usize,
    sent: Vec<Value>,
}

/// Pin and record tests.
struct OpenRouterPins {
    transport: Arc<Transport>,
    scheduler: Arc<dyn Scheduler>,
    state: Arc<Mutex<PinState>>,
}

fn locked(state: &Mutex<PinState>) -> MutexGuard<'_, PinState> {
    state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

impl Pins for OpenRouterPins {
    fn hop<'a>(
        &'a self,
        id: &'a str,
        view: &'a ModelView,
        cancel: &'a CancelToken,
    ) -> BoxFuture<'a, Result<Vec<String>, KindError>> {
        Box::pin(async move {
            let limit = deadline(&*self.scheduler, DETAIL_MS, cancel);
            let read = self
                .transport
                .get_json(&endpoints_path(id), limit.token(), &KeyAuth::None)
                .await;
            let ended = limit.ended();
            limit.clear();
            let raw = match read {
                Ok(raw) => raw,
                Err(error) => {
                    return Err(match (error, ended) {
                        (crate::transport::TransportError::Cancelled, Some(Ended::Deadline)) => {
                            KindError::TimedOut
                        }
                        (error, _) => error.into(),
                    });
                }
            };
            let serving = parse_providers(&raw, &[]).unwrap_or_default();
            let zdr: Vec<String> = view.hosts.iter().map(|tag| slug_of(tag)).collect();
            let mut both: Vec<String> = Vec::new();
            for provider in &serving {
                let slug = slug_of(&provider.tag);
                if !both.contains(&slug) {
                    both.push(slug);
                }
            }
            both.retain(|slug| !slug.is_empty() && zdr.contains(slug));
            both.truncate(2);
            Ok(if both.len() < 2 { Vec::new() } else { both })
        })
    }

    fn wrap(&self, codec: Box<dyn Codec>) -> Box<dyn Codec> {
        Box::new(Pinned {
            base: codec,
            state: Arc::clone(&self.state),
        })
    }

    fn pin(&self, hosts: &[String]) {
        let mut state = locked(&self.state);
        state.hosts = hosts.to_vec();
        state.step = 0;
        state.sent = Vec::new();
    }

    fn passed_back(&self) -> bool {
        locked(&self.state).sent.iter().skip(1).any(|body| {
            body.get("messages")
                .and_then(Value::as_array)
                .is_some_and(|messages| {
                    messages.iter().any(|message| {
                        message.get("role").and_then(Value::as_str) == Some("assistant")
                            && message
                                .get("reasoning_details")
                                .and_then(Value::as_array)
                                .is_some_and(|held| !held.is_empty())
                    })
                })
        })
    }
}

/// A codec that pins each stream request to the next host and records it.
struct Pinned {
    base: Box<dyn Codec>,
    state: Arc<Mutex<PinState>>,
}

impl Codec for Pinned {
    fn encode(&self, request: &CanonicalRequest, mode: Mode) -> Result<Encoded, String> {
        let encoded = self.base.encode(request, mode)?;
        if mode != Mode::Stream {
            return Ok(encoded);
        }
        let mut state = locked(&self.state);
        state.sent.push(encoded.body.clone());
        let at = state.step % state.hosts.len().max(1);
        state.step += 1;
        let Some(host) = state.hosts.get(at).cloned() else {
            return Ok(encoded);
        };
        let Value::Object(body) = &encoded.body else {
            return Ok(encoded);
        };
        let mut provider = Object::new();
        provider.insert("require_parameters", Value::Bool(true));
        if let Some(Value::Object(held)) = body.get("provider") {
            for (key, value) in held.iter() {
                provider.insert(key, value.clone());
            }
        }
        provider.insert("only", Value::Array(vec![Value::String(host)]));
        let mut pinned = body.clone();
        pinned.insert("provider", Value::Object(provider));
        Ok(Encoded {
            body: Value::Object(pinned),
            ..encoded
        })
    }

    fn stream(&self) -> Box<dyn StreamDecoder + '_> {
        self.base.stream()
    }

    fn complete(&self, body: &Value) -> Result<StepResult, ProviderError> {
        self.base.complete(body)
    }

    fn failure(&self, status: i64, body: &Value, headers: &HeaderList) -> Failure {
        self.base.failure(status, body, headers)
    }
}
