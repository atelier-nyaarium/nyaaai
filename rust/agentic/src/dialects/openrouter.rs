//! OpenRouter Chat dialect.

use nyaaai_contract::json::{Object, Value, number_to_string};
use nyaaai_contract::shapes::step::{
    CanonicalRequest, Failure, FailureCategory, NormalizedUsage, Serving, ThoughtDelta,
    ThoughtPart, ThoughtPartSegmentsItem, ThoughtPartSegmentsItemKind, Turn, TurnStep,
};
use nyaaai_contract::text::ascii_lower;

use crate::chat::codec::{ChatCodec, ChatDialect, chat_codec, code_of, failure_status, usable};
use crate::chat::wire::{Delta, ReasoningDetail, Usage};
use crate::codec::Mode;
use crate::failure::WireError;
use crate::native::HostStatus;
use crate::schema_projection::strict_schema;

const DETAIL_TYPES: [&str; 3] = ["reasoning.text", "reasoning.summary", "reasoning.encrypted"];

const ENCRYPTED: &str = "reasoning.encrypted";

/// Zero-retention routing.
pub fn provider() -> Object {
    let mut held = Object::new();
    held.insert("zdr", Value::Bool(true));
    held.insert("data_collection", Value::from("deny"));
    held.insert("require_parameters", Value::Bool(true));
    held
}

pub fn thoughts_of(details: &[ReasoningDetail]) -> Vec<ThoughtPart> {
    details
        .iter()
        .map(|detail| {
            let mut segments = Vec::new();
            if let Some(text) = &detail.text {
                segments.push(ThoughtPartSegmentsItem {
                    kind: ThoughtPartSegmentsItemKind::Content,
                    text: text.clone(),
                });
            }
            if let Some(text) = &detail.summary {
                segments.push(ThoughtPartSegmentsItem {
                    kind: ThoughtPartSegmentsItemKind::Summary,
                    text: text.clone(),
                });
            }
            let opaque = if detail.type_ == ENCRYPTED {
                &detail.data
            } else {
                &detail.signature
            };
            ThoughtPart {
                type_: detail.type_.clone(),
                segments: (!segments.is_empty()).then_some(segments),
                opaque: opaque.clone(),
                id: detail.id.clone(),
                format: detail.format.clone(),
                index: detail.index,
                anchor: None,
            }
        })
        .collect()
}

/// Thought parts OpenRouter reads back; others drop.
pub fn details_of<'a>(thoughts: impl IntoIterator<Item = &'a ThoughtPart>) -> Vec<ReasoningDetail> {
    thoughts
        .into_iter()
        .filter(|part| DETAIL_TYPES.contains(&part.type_.as_str()))
        .map(|part| {
            let of = |kind: ThoughtPartSegmentsItemKind| {
                let texts: Vec<&str> = part
                    .segments
                    .iter()
                    .flatten()
                    .filter(|segment| segment.kind == kind)
                    .map(|segment| segment.text.as_str())
                    .collect();
                (!texts.is_empty()).then(|| texts.concat())
            };
            let encrypted = part.type_ == ENCRYPTED;
            ReasoningDetail {
                type_: part.type_.clone(),
                id: part.id.clone(),
                format: part.format.clone(),
                index: part.index,
                text: of(ThoughtPartSegmentsItemKind::Content),
                summary: of(ThoughtPartSegmentsItemKind::Summary),
                data: part.opaque.clone().filter(|_| encrypted),
                signature: part.opaque.clone().filter(|_| !encrypted),
            }
        })
        .collect()
}

fn step_details(step: &TurnStep) -> Vec<ReasoningDetail> {
    details_of(step.parts.iter().filter_map(|part| match part {
        nyaaai_contract::shapes::step::TurnPart::Thought(thought) => Some(thought),
        _ => None,
    }))
}

/// Off allows hosts that keep or collect data, free models included.
fn provider_for(zdr_only: bool) -> Object {
    if zdr_only {
        return provider();
    }
    let mut held = Object::new();
    held.insert("require_parameters", Value::Bool(true));
    held
}

/// Host of the newest step whose thought blocks go back; `None` when unknown.
fn affinity_of(request: &CanonicalRequest) -> Option<String> {
    for turn in request.history.iter().rev() {
        if let Turn::Step(step) = turn
            && let Some(producer) = &step.producer
            && producer.model == request.model
            && !step_details(step).is_empty()
        {
            return producer.host.clone();
        }
    }
    None
}

/// A provider's capacity refusal, never the key's own limits.
fn upstream(error: &WireError) -> bool {
    let meta = error.metadata.as_ref();
    let held = |key: &str| meta.and_then(|meta| meta.get(key));
    held("error_type").and_then(Value::as_str) == Some("rate_limit_exceeded")
        || held("provider_name").is_some_and(|name| matches!(name, Value::String(_)))
        || ascii_lower(error.message.as_deref().unwrap_or("")).contains("upstream")
}

fn category_of(code: Option<f64>, error: &WireError) -> FailureCategory {
    let Some(code) = code else {
        return FailureCategory::Stream;
    };
    if code == 400.0 {
        return if ascii_lower(error.message.as_deref().unwrap_or(""))
            .contains("provider that created it")
        {
            FailureCategory::ReplayRejected
        } else {
            FailureCategory::BadRequest
        };
    }
    if code == 401.0 {
        FailureCategory::KeyRejected
    } else if code == 402.0 {
        FailureCategory::OutOfCredit
    } else if code == 403.0 {
        FailureCategory::Refused
    } else if code == 404.0 {
        FailureCategory::NotFound
    } else if code == 429.0 {
        if upstream(error) {
            FailureCategory::Overloaded
        } else {
            FailureCategory::RateLimited
        }
    } else if code == 408.0 || code >= 500.0 {
        FailureCategory::Unavailable
    } else {
        FailureCategory::BadRequest
    }
}

/// By HTTP status, else by an in-stream error's code.
fn failure_for(status: HostStatus, error: &WireError) -> Failure {
    let reasons = error
        .metadata
        .as_ref()
        .and_then(|meta| meta.get("reasons"))
        .and_then(Value::as_array)
        .map(|held| {
            held.iter()
                .filter_map(|reason| reason.as_str().map(str::to_owned))
                .collect()
        });
    Failure {
        category: category_of(code_of(status, error), error),
        status: failure_status(status),
        retry_after_ms: None,
        resets_at: None,
        message: usable(error.message.as_deref())
            .unwrap_or("OpenRouter reported an error.")
            .to_owned(),
        reasons,
        billed: None,
    }
}

fn normalized(usage: &Usage) -> NormalizedUsage {
    let prompt = usage.prompt_tokens_details.as_ref();
    NormalizedUsage {
        input: usage.prompt_tokens,
        output: usage.completion_tokens,
        cached: prompt.and_then(|held| held.cached_tokens),
        cache_write: prompt.and_then(|held| held.cache_write_tokens),
        reasoning: usage.reasoning_tokens.flatten(),
        cost: usage.cost,
        upstream_cost: usage.upstream_inference_cost.flatten(),
        byok: usage.is_byok,
        billing_source: None,
        list_cost: None,
        allowance: None,
    }
}

pub struct OpenRouterDialect;

impl ChatDialect for OpenRouterDialect {
    fn fields(&self, request: &CanonicalRequest, _mode: Mode) -> Object {
        let host = affinity_of(request);
        let mut provider = provider_for(request.zdr_only);
        if let Some(host) = host {
            provider.insert("order", Value::Array(vec![Value::String(host)]));
        }
        let mut fields = Object::new();
        fields.insert("session_id", Value::from(request.session.as_str()));
        fields.insert("provider", Value::Object(provider));
        if let Some(effort) = request.effort {
            let mut held = Object::new();
            held.insert("effort", Value::from(effort.as_str()));
            fields.insert("reasoning", Value::Object(held));
        }
        fields
    }

    // Preserve model-specific blocks.
    fn replay(&self, step: &TurnStep, request: &CanonicalRequest) -> Object {
        let own = step
            .producer
            .as_ref()
            .is_some_and(|producer| producer.model == request.model);
        let details = if own { step_details(step) } else { Vec::new() };
        let mut held = Object::new();
        if !details.is_empty() {
            held.insert(
                "reasoning_details",
                Value::Array(details.iter().map(ReasoningDetail::encode).collect()),
            );
        }
        held
    }

    // Block identity: type/index.
    fn thoughts(&self, delta: &Delta) -> Vec<ThoughtDelta> {
        if let Some(details) = delta
            .reasoning_details
            .as_ref()
            .filter(|held| !held.is_empty())
        {
            return thoughts_of(details)
                .into_iter()
                .map(|part| ThoughtDelta {
                    block: part
                        .index
                        .map(|index| format!("{}#{}", part.type_, number_to_string(index))),
                    part,
                    split: None,
                })
                .collect();
        }
        match delta.reasoning.as_deref() {
            Some(text) if !text.is_empty() => vec![ThoughtDelta {
                part: ThoughtPart {
                    type_: "reasoning".to_owned(),
                    segments: Some(vec![ThoughtPartSegmentsItem {
                        kind: ThoughtPartSegmentsItemKind::Content,
                        text: text.to_owned(),
                    }]),
                    opaque: None,
                    id: None,
                    format: None,
                    index: None,
                    anchor: None,
                },
                block: None,
                split: None,
            }],
            _ => Vec::new(),
        }
    }

    fn cached(&self, text: &str, long: bool) -> Option<Vec<Value>> {
        let mut control = Object::new();
        control.insert("type", Value::from("ephemeral"));
        if long {
            control.insert("ttl", Value::from("1h"));
        }
        let mut part = Object::new();
        part.insert("type", Value::from("text"));
        part.insert("text", Value::from(text));
        part.insert("cache_control", Value::Object(control));
        Some(vec![Value::Object(part)])
    }

    fn schema(&self, json: &Object) -> Result<Option<Object>, String> {
        strict_schema(json).map(Some)
    }

    fn usage(&self, usage: &Usage) -> NormalizedUsage {
        normalized(usage)
    }

    fn serving(&self, id: Option<&str>, provider: Option<&str>) -> Serving {
        Serving {
            host: provider.map(str::to_owned),
            generation_id: id.map(str::to_owned),
        }
    }

    fn failure(
        &self,
        status: HostStatus,
        error: &WireError,
        _headers: Option<&nyaaai_contract::shapes::step::HeaderList>,
    ) -> Failure {
        failure_for(status, error)
    }
}

pub const OPENROUTER_CHAT: ChatCodec<OpenRouterDialect> = chat_codec(OpenRouterDialect);
