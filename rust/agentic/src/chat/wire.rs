//! Chat Completions wire types and readers.

use nyaaai_contract::json::{Object, Value};

use crate::failure::{NumberOrText, WireError};
use crate::native::number_of;

/// OpenRouter's thought blocks.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ReasoningDetail {
    pub type_: String,
    pub id: Option<String>,
    pub format: Option<String>,
    pub index: Option<f64>,
    pub text: Option<String>,
    pub summary: Option<String>,
    pub data: Option<String>,
    pub signature: Option<String>,
}

impl ReasoningDetail {
    /// The request's spelling.
    pub fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("type", Value::from(self.type_.as_str()));
        let text = |key: &str, held: &Option<String>, object: &mut Object| {
            if let Some(held) = held {
                object.insert(key, Value::from(held.as_str()));
            }
        };
        text("id", &self.id, &mut object);
        text("format", &self.format, &mut object);
        if let Some(index) = self.index {
            object.insert("index", Value::Number(index));
        }
        text("text", &self.text, &mut object);
        text("summary", &self.summary, &mut object);
        text("data", &self.data, &mut object);
        text("signature", &self.signature, &mut object);
        Value::Object(object)
    }
}

/// OpenCode's token block.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BillingTokens {
    pub input: Option<f64>,
    pub output: Option<f64>,
    pub reasoning: Option<f64>,
    pub cache_read: Option<f64>,
    pub cache_write_5m: Option<f64>,
    pub cache_write_1h: Option<f64>,
}

/// OpenCode's billing.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Billing {
    pub billing_source: Option<String>,
    pub list_cost: Option<f64>,
    pub allowance: Option<f64>,
    pub tokens: Option<BillingTokens>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PromptDetails {
    pub cached_tokens: Option<f64>,
    pub cache_write_tokens: Option<f64>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Usage {
    pub prompt_tokens: f64,
    pub completion_tokens: f64,
    pub total_tokens: Option<f64>,
    pub cost: Option<f64>,
    pub is_byok: Option<bool>,
    /// `Some(None)`: the block held no usable number.
    pub upstream_inference_cost: Option<Option<f64>>,
    pub prompt_tokens_details: Option<PromptDetails>,
    /// `Some(None)`: the block held no reasoning count.
    pub reasoning_tokens: Option<Option<f64>>,
    pub opencode: Option<Billing>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ToolCallDelta {
    /// Absent when not a number.
    pub index: Option<f64>,
    pub id: Option<String>,
    pub name: Option<String>,
    pub arguments: Option<String>,
}

/// A whole reply's call.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct WireCall {
    pub id: String,
    pub name: String,
    pub arguments: String,
}

/// A streamed delta or a whole reply message.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Delta {
    pub role: Option<String>,
    pub content: Option<Value>,
    pub reasoning: Option<String>,
    pub reasoning_details: Option<Vec<ReasoningDetail>>,
    pub reasoning_content: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StreamedChoice {
    /// The delta and its calls.
    pub delta: Option<(Delta, Option<Vec<ToolCallDelta>>)>,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReplyChoice {
    /// The message and its calls.
    pub message: Option<(Delta, Option<Vec<WireCall>>)>,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Chunk {
    pub id: Option<String>,
    pub provider: Option<String>,
    pub choices: Option<Vec<StreamedChoice>>,
    pub usage: Option<Usage>,
    pub error: Option<WireError>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Completion {
    pub id: Option<String>,
    pub provider: Option<String>,
    pub choices: Option<Vec<ReplyChoice>>,
    pub usage: Option<Usage>,
    pub error: Option<WireError>,
}

fn string_at(raw: &Object, key: &str) -> Option<String> {
    raw.get(key).and_then(Value::as_str).map(str::to_owned)
}

fn number_at(raw: &Object, key: &str) -> Option<f64> {
    raw.get(key).and_then(number_of)
}

/// A count or an amount: negative reads as mistyped, negative zero as zero.
fn amount_at(raw: &Object, key: &str) -> Option<f64> {
    number_at(raw, key)
        .filter(|value| *value >= 0.0)
        .map(|value| value + 0.0)
}

fn object_at<'a>(raw: &'a Object, key: &str) -> Option<&'a Object> {
    raw.get(key).and_then(Value::as_object)
}

/// Absent, null, false, zero and empty text are no error; a text is the message; other non-objects have none.
fn read_reply_error(raw: Option<&Value>) -> Option<WireError> {
    match raw {
        None | Some(Value::Null | Value::Bool(false)) => None,
        Some(Value::Number(number)) if *number == 0.0 => None,
        Some(Value::String(text)) if text.is_empty() => None,
        Some(Value::Object(_) | Value::String(_)) => raw.and_then(read_error),
        Some(_) => Some(WireError::default()),
    }
}

/// A text is the message.
pub fn read_error(raw: &Value) -> Option<WireError> {
    let raw = match raw {
        Value::String(text) => {
            return Some(WireError {
                message: Some(text.clone()),
                ..WireError::default()
            });
        }
        Value::Object(raw) => raw,
        _ => return None,
    };
    let code = match raw.get("code") {
        Some(Value::String(text)) => Some(NumberOrText::Text(text.clone())),
        Some(held) => number_of(held).map(NumberOrText::Number),
        None => None,
    };
    let resets_at = match raw.get("resetsAt") {
        Some(Value::String(text)) => Some(NumberOrText::Text(text.clone())),
        Some(Value::Number(number)) => Some(NumberOrText::Number(*number)),
        _ => None,
    };
    Some(WireError {
        code,
        type_: string_at(raw, "type"),
        message: string_at(raw, "message"),
        metadata: object_at(raw, "metadata").cloned(),
        resets_at,
    })
}

/// Mistyped counts become zero.
pub fn read_usage(raw: Option<&Value>) -> Option<Usage> {
    let Some(Value::Object(raw)) = raw else {
        return None;
    };
    let prompt = object_at(raw, "prompt_tokens_details");
    let completion = object_at(raw, "completion_tokens_details");
    Some(Usage {
        prompt_tokens: amount_at(raw, "prompt_tokens").unwrap_or(0.0),
        completion_tokens: amount_at(raw, "completion_tokens").unwrap_or(0.0),
        total_tokens: amount_at(raw, "total_tokens"),
        cost: amount_at(raw, "cost"),
        is_byok: raw.get("is_byok").and_then(Value::as_bool),
        upstream_inference_cost: object_at(raw, "cost_details")
            .map(|costs| amount_at(costs, "upstream_inference_cost")),
        prompt_tokens_details: prompt.map(|held| PromptDetails {
            cached_tokens: amount_at(held, "cached_tokens"),
            cache_write_tokens: amount_at(held, "cache_write_tokens"),
        }),
        reasoning_tokens: completion.map(|held| amount_at(held, "reasoning_tokens")),
        opencode: object_at(raw, "opencode").map(read_billing),
    })
}

pub fn read_billing(block: &Object) -> Billing {
    Billing {
        billing_source: string_at(block, "billing_source"),
        list_cost: amount_at(block, "list_cost"),
        allowance: amount_at(block, "allowance"),
        tokens: object_at(block, "tokens").map(|tokens| BillingTokens {
            input: amount_at(tokens, "input"),
            output: amount_at(tokens, "output"),
            reasoning: amount_at(tokens, "reasoning"),
            cache_read: amount_at(tokens, "cache_read"),
            cache_write_5m: amount_at(tokens, "cache_write_5m"),
            cache_write_1h: amount_at(tokens, "cache_write_1h"),
        }),
    }
}

fn read_detail(raw: &Value) -> Option<ReasoningDetail> {
    let raw = raw.as_object()?;
    let type_ = string_at(raw, "type")?;
    Some(ReasoningDetail {
        type_,
        id: string_at(raw, "id"),
        format: string_at(raw, "format"),
        index: number_at(raw, "index"),
        text: string_at(raw, "text"),
        summary: string_at(raw, "summary"),
        data: string_at(raw, "data"),
        signature: string_at(raw, "signature"),
    })
}

fn read_delta(raw: Option<&Value>) -> Option<Delta> {
    let raw = raw?.as_object()?;
    Some(Delta {
        role: string_at(raw, "role"),
        content: raw.get("content").cloned(),
        reasoning: string_at(raw, "reasoning"),
        reasoning_details: match raw.get("reasoning_details") {
            Some(Value::Array(details)) => Some(details.iter().filter_map(read_detail).collect()),
            _ => None,
        },
        reasoning_content: string_at(raw, "reasoning_content"),
    })
}

fn read_call_delta(raw: &Value) -> Option<ToolCallDelta> {
    let raw = raw.as_object()?;
    let held = object_at(raw, "function");
    Some(ToolCallDelta {
        index: number_at(raw, "index"),
        id: string_at(raw, "id"),
        name: held.and_then(|held| string_at(held, "name")),
        arguments: held.and_then(|held| string_at(held, "arguments")),
    })
}

fn read_wire_call(raw: &Value) -> Option<WireCall> {
    let raw = raw.as_object()?;
    let held = object_at(raw, "function");
    Some(WireCall {
        id: string_at(raw, "id").unwrap_or_default(),
        name: held
            .and_then(|held| string_at(held, "name"))
            .unwrap_or_default(),
        arguments: held
            .and_then(|held| string_at(held, "arguments"))
            .unwrap_or_default(),
    })
}

/// A mistyped reason is an error.
fn read_finish(raw: Option<&Value>) -> Option<String> {
    match raw {
        Some(Value::String(text)) => Some(text.clone()),
        None | Some(Value::Null) => None,
        Some(_) => Some("error".to_owned()),
    }
}

fn calls_of<T>(raw: &Object, key: &str, read: fn(&Value) -> Option<T>) -> Option<Vec<T>> {
    match object_at(raw, key)?.get("tool_calls") {
        Some(Value::Array(calls)) => Some(calls.iter().filter_map(read).collect()),
        _ => None,
    }
}

/// One streamed chunk.
pub fn read_chunk(raw: &Object) -> Chunk {
    let choices = match raw.get("choices") {
        Some(Value::Array(choices)) => Some(
            choices
                .iter()
                .filter_map(Value::as_object)
                .map(|each| StreamedChoice {
                    delta: read_delta(each.get("delta"))
                        .map(|delta| (delta, calls_of(each, "delta", read_call_delta))),
                    finish_reason: read_finish(each.get("finish_reason")),
                })
                .collect(),
        ),
        _ => None,
    };
    Chunk {
        id: string_at(raw, "id"),
        provider: string_at(raw, "provider"),
        choices,
        usage: read_usage(raw.get("usage")),
        error: read_reply_error(raw.get("error")),
    }
}

/// One whole reply.
pub fn read_completion(raw: &Object) -> Completion {
    let choices = match raw.get("choices") {
        Some(Value::Array(choices)) => Some(
            choices
                .iter()
                .filter_map(Value::as_object)
                .map(|each| ReplyChoice {
                    message: read_delta(each.get("message"))
                        .map(|message| (message, calls_of(each, "message", read_wire_call))),
                    finish_reason: read_finish(each.get("finish_reason")),
                })
                .collect(),
        ),
        _ => None,
    };
    Completion {
        id: string_at(raw, "id"),
        provider: string_at(raw, "provider"),
        choices,
        usage: read_usage(raw.get("usage")),
        error: read_reply_error(raw.get("error")),
    }
}
