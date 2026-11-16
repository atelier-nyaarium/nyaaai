//! Shared Chat Completions codec.

use nyaaai_contract::json::{Object, Record, Value, stringify_json};
use nyaaai_contract::shapes::step::{
    CanonicalRequest, CanonicalRequestCacheKind, Encoded, Failure, FailureBilled, FailureCategory,
    FailureStatus, FinishReason, HeaderList, NormalizedUsage, Serving, SseItem, StepEvent,
    StepEventCall, StepEventFinish, StepEventServing, StepEventText, StepEventThought,
    StepEventUsage, StepResult, ThoughtDelta, ToolSpec, Turn, TurnPart, TurnStep,
};
use nyaaai_contract::text::trim_text;

use super::text::{normalize_response_value, text_of};
use super::wire::{Chunk, Completion, Delta, Usage, read_chunk, read_completion, read_error};
use crate::codec::{Codec, Mode, StreamDecoder};
use crate::failure::{FailureExtra, ProviderError, WireError, failed};
use crate::native::{HostStatus, is_json, parsed_chunk, schema_prompt};
use crate::step::StepAssembler;

/// What a dialect says about `tool_choice`.
pub enum ToolChoiceRule {
    /// No say: the named tool is forced.
    Forced,
    /// The key is left out.
    Omit,
    Send(Value),
}

/// Host-specific Chat Completions extensions.
pub trait ChatDialect {
    /// Defaults to `/chat/completions`.
    fn path(&self) -> &str {
        PATH
    }

    fn headers(&self, _request: &CanonicalRequest) -> Option<Record<String>> {
        None
    }

    /// Provider-specific request fields.
    fn fields(&self, request: &CanonicalRequest, mode: Mode) -> Object;

    /// Thought replay fields.
    fn replay(&self, step: &TurnStep, request: &CanonicalRequest) -> Object;

    /// Thought block identities.
    fn thoughts(&self, delta: &Delta) -> Vec<ThoughtDelta>;

    /// Explicit cache control; `None`: the dialect has none.
    fn cached(&self, _text: &str, _long: bool) -> Option<Vec<Value>> {
        None
    }

    /// Enforced vs prompt-only schema; `Err`: refused.
    fn schema(&self, json: &Object) -> Result<Option<Object>, String>;

    /// Enforcement hides descriptions.
    fn prompt_schema(&self) -> bool {
        false
    }

    fn tool_choice(&self, _name: &str) -> ToolChoiceRule {
        ToolChoiceRule::Forced
    }

    /// Minimum `max_tokens`.
    fn min_tokens(&self) -> i64 {
        0
    }

    fn usage(&self, usage: &Usage) -> NormalizedUsage;

    fn serving(&self, id: Option<&str>, provider: Option<&str>) -> Serving;

    /// HTTP 200 stream errors.
    fn failure(
        &self,
        status: HostStatus,
        error: &WireError,
        headers: Option<&HeaderList>,
    ) -> Failure;
}

const PATH: &str = "/chat/completions";

/// A failure's status as the dialects write it.
pub fn failure_status(status: HostStatus) -> FailureStatus {
    match status {
        HostStatus::Http(code) => FailureStatus::Number(code as f64),
        HostStatus::Stream => FailureStatus::Text("stream".to_owned()),
    }
}

/// The code that picks a category: the HTTP status, else an error's number.
pub fn code_of(status: HostStatus, error: &WireError) -> Option<f64> {
    match status {
        HostStatus::Http(code) => Some(code as f64),
        HostStatus::Stream => match &error.code {
            Some(crate::failure::NumberOrText::Number(code)) => Some(*code),
            _ => None,
        },
    }
}

/// Blank counts as missing.
pub fn usable(text: Option<&str>) -> Option<&str> {
    text.filter(|text| !trim_text(text).is_empty())
}

fn finish_of(reason: &str) -> FinishReason {
    let named = match reason {
        "function_call" => "tool_calls",
        "max_tokens" => "length",
        "insufficient_system_resource" | "network_error" => "error",
        other => other,
    };
    FinishReason::parse(named).unwrap_or(FinishReason::Stop)
}

fn object_of(entries: Vec<(&str, Value)>) -> Value {
    let mut object = Object::new();
    for (key, value) in entries {
        object.insert(key, value);
    }
    Value::Object(object)
}

fn tool_of(tool: &ToolSpec) -> Value {
    object_of(vec![
        ("type", Value::from("function")),
        (
            "function",
            object_of(vec![
                ("name", Value::from(tool.name.as_str())),
                ("description", Value::from(tool.description.as_str())),
                ("parameters", Value::Object(tool.parameters.clone())),
            ]),
        ),
    ])
}

fn turn_messages(d: &dyn ChatDialect, request: &CanonicalRequest, turn: &Turn) -> Vec<Value> {
    let step = match turn {
        Turn::User(user) => {
            return vec![object_of(vec![
                ("role", Value::from("user")),
                ("content", Value::from(user.text.as_str())),
            ])];
        }
        Turn::Step(step) => step,
    };
    let text: String = step
        .parts
        .iter()
        .filter_map(|part| match part {
            TurnPart::Text(text) => Some(text.text.as_str()),
            _ => None,
        })
        .collect();
    let calls: Vec<_> = step
        .parts
        .iter()
        .filter_map(|part| match part {
            TurnPart::Call(call) => Some(call),
            _ => None,
        })
        .collect();
    let mut message = Object::new();
    message.insert("role", Value::from("assistant"));
    message.insert(
        "content",
        if text.is_empty() && !calls.is_empty() {
            Value::Null
        } else {
            Value::String(text)
        },
    );
    if !calls.is_empty() {
        message.insert(
            "tool_calls",
            Value::Array(
                calls
                    .iter()
                    .map(|call| {
                        object_of(vec![
                            ("id", Value::from(call.wire_id.as_str())),
                            ("type", Value::from("function")),
                            (
                                "function",
                                object_of(vec![
                                    ("name", Value::from(call.name.as_str())),
                                    ("arguments", Value::from(call.arguments.as_str())),
                                ]),
                            ),
                        ])
                    })
                    .collect(),
            ),
        );
    }
    for (key, value) in d.replay(step, request).iter() {
        message.insert(key, value.clone());
    }
    let mut messages = vec![Value::Object(message)];
    for part in &step.parts {
        let TurnPart::Result(result) = part else {
            continue;
        };
        // The last call of an index names it.
        let wire = calls.iter().rev().find(|call| call.index == result.index);
        if let Some(call) = wire {
            messages.push(object_of(vec![
                ("role", Value::from("tool")),
                ("tool_call_id", Value::from(call.wire_id.as_str())),
                ("content", Value::from(result.output.as_str())),
            ]));
        }
    }
    messages
}

/// Marks the last cacheable message.
fn with_last_cached(history: Vec<Value>, mark: &dyn Fn(&str) -> Value) -> Vec<Value> {
    let mut marked = history;
    for message in marked.iter_mut().rev() {
        let Value::Object(object) = message else {
            continue;
        };
        if object.get("role").and_then(Value::as_str) == Some("assistant") {
            continue;
        }
        let Some(Value::String(content)) = object.get("content") else {
            continue;
        };
        let content = mark(content);
        object.insert("content", content);
        break;
    }
    marked
}

// State stays last.
// Implicit caching marks system.
// Explicit caching marks history.
// Explicit state is user-role.
fn messages_of(
    d: &dyn ChatDialect,
    request: &CanonicalRequest,
    prompted: Option<&str>,
) -> Vec<Value> {
    let explicit = request.cache.kind == CanonicalRequestCacheKind::Explicit;
    let long = request.cache.ttl.is_some();
    let mark = |text: &str| -> Value {
        if explicit && let Some(parts) = d.cached(text, long) {
            return Value::Array(parts);
        }
        Value::from(text)
    };
    let message = |role: &str, content: Value| {
        object_of(vec![("role", Value::from(role)), ("content", content)])
    };
    let mut messages: Vec<Value> = Vec::new();
    for (at, text) in request.system.iter().enumerate() {
        let content = if at + 1 == request.system.len() {
            mark(text)
        } else {
            Value::from(text.as_str())
        };
        messages.push(message("system", content));
    }
    if let Some(summary) = &request.summary {
        messages.push(message("system", mark(summary)));
    }
    if let Some(prompted) = prompted {
        messages.push(message("system", Value::from(prompted)));
    }
    let history: Vec<Value> = request
        .history
        .iter()
        .flat_map(|turn| turn_messages(d, request, turn))
        .collect();
    messages.extend(if explicit {
        with_last_cached(history, &mark)
    } else {
        history
    });
    if let Some(state) = &request.state {
        let role = if explicit { "user" } else { "system" };
        messages.push(message(role, Value::from(state.as_str())));
    }
    messages
}

fn body_of(d: &dyn ChatDialect, request: &CanonicalRequest, mode: Mode) -> Result<Value, String> {
    let enforced = match &request.schema {
        None => None,
        Some(schema) => d.schema(&schema.json)?,
    };
    let prompted = request.schema.as_ref().and_then(|schema| {
        (enforced.is_none() || d.prompt_schema()).then(|| schema_prompt(&schema.json))
    });
    let forced =
        request
            .tool_choice
            .as_ref()
            .and_then(|choice| match d.tool_choice(&choice.name) {
                ToolChoiceRule::Forced => Some(object_of(vec![
                    ("type", Value::from("function")),
                    (
                        "function",
                        object_of(vec![("name", Value::from(choice.name.as_str()))]),
                    ),
                ])),
                ToolChoiceRule::Omit => None,
                ToolChoiceRule::Send(value) => Some(value),
            });
    let mut body = Object::new();
    body.insert("model", Value::from(request.model.as_str()));
    body.insert(
        "messages",
        Value::Array(messages_of(d, request, prompted.as_deref())),
    );
    body.insert("stream", Value::Bool(mode == Mode::Stream));
    for (key, value) in d.fields(request, mode).iter() {
        body.insert(key, value.clone());
    }
    if let Some(tools) = request.tools.as_ref().filter(|tools| !tools.is_empty()) {
        body.insert("tools", Value::Array(tools.iter().map(tool_of).collect()));
    }
    if let Some(forced) = forced {
        body.insert("tool_choice", forced);
    }
    if let (Some(schema), Some(enforced)) = (&request.schema, enforced) {
        body.insert(
            "response_format",
            object_of(vec![
                ("type", Value::from("json_schema")),
                (
                    "json_schema",
                    object_of(vec![
                        ("name", Value::from(schema.name.as_str())),
                        ("strict", Value::Bool(true)),
                        ("schema", Value::Object(enforced)),
                    ]),
                ),
            ]),
        );
    }
    if let Some(max) = request.max_tokens {
        body.insert("max_tokens", Value::Number(max.max(d.min_tokens()) as f64));
    }
    Ok(Value::Object(body))
}

fn serving_event(
    d: &dyn ChatDialect,
    id: Option<&str>,
    provider: Option<&str>,
    events: &mut Vec<StepEvent>,
) {
    let serving = d.serving(id, provider);
    if serving.host.is_some() || serving.generation_id.is_some() {
        events.push(StepEvent::Serving(StepEventServing { serving }));
    }
}

/// Thoughts precede text.
fn delta_events(d: &dyn ChatDialect, delta: &Delta, events: &mut Vec<StepEvent>) {
    for thought in d.thoughts(delta) {
        events.push(StepEvent::Thought(StepEventThought {
            part: thought.part,
            block: thought.block,
            split: thought.split,
        }));
    }
    let value = delta.content.as_ref().and_then(normalize_response_value);
    let text = match &value {
        Some(held) if is_json(held) => stringify_json(held),
        Some(held) => text_of(held),
        None => String::new(),
    };
    if !text.is_empty() {
        events.push(StepEvent::Text(StepEventText { text, phase: None }));
    }
}

fn call_failure() -> ProviderError {
    failed(
        FailureCategory::Stream,
        "A tool call had no valid index.",
        FailureExtra::default(),
    )
}

fn chunk_events(
    d: &dyn ChatDialect,
    chunk: &Chunk,
    events: &mut Vec<StepEvent>,
    finished: &mut bool,
) -> Result<(), ProviderError> {
    serving_event(d, chunk.id.as_deref(), chunk.provider.as_deref(), events);
    for choice in chunk.choices.iter().flatten() {
        if let Some((delta, calls)) = &choice.delta {
            delta_events(d, delta, events);
            for call in calls.iter().flatten() {
                let Some(index) = call.index else {
                    return Err(call_failure());
                };
                events.push(StepEvent::Call(StepEventCall {
                    index,
                    wire_id: call.id.clone().unwrap_or_default(),
                    name: call.name.clone().unwrap_or_default(),
                    arguments_text: call.arguments.clone().unwrap_or_default(),
                }));
            }
        }
        if let Some(reason) = &choice.finish_reason {
            *finished = true;
            events.push(StepEvent::Finish(StepEventFinish {
                reason: finish_of(reason),
            }));
        }
    }
    if let Some(usage) = &chunk.usage {
        events.push(StepEvent::Usage(StepEventUsage {
            usage: d.usage(usage),
        }));
    }
    Ok(())
}

fn reply_events(d: &dyn ChatDialect, reply: &Completion, events: &mut Vec<StepEvent>) {
    serving_event(d, reply.id.as_deref(), reply.provider.as_deref(), events);
    let choice = reply.choices.iter().flatten().next();
    if let Some((message, calls)) = choice.and_then(|choice| choice.message.as_ref()) {
        delta_events(d, message, events);
        for (index, call) in calls.iter().flatten().enumerate() {
            events.push(StepEvent::Call(StepEventCall {
                index: index as f64,
                wire_id: call.id.clone(),
                name: call.name.clone(),
                arguments_text: call.arguments.clone(),
            }));
        }
    }
    if let Some(reason) = choice.and_then(|choice| choice.finish_reason.as_ref()) {
        events.push(StepEvent::Finish(StepEventFinish {
            reason: finish_of(reason),
        }));
    }
    if let Some(usage) = &reply.usage {
        events.push(StepEvent::Usage(StepEventUsage {
            usage: d.usage(usage),
        }));
    }
}

/// HTTP 200 error details.
fn reply_failure(
    d: &dyn ChatDialect,
    id: Option<&str>,
    provider: Option<&str>,
    usage: Option<&Usage>,
    error: &WireError,
) -> ProviderError {
    let failure = d.failure(HostStatus::Stream, error, None);
    let serving = d.serving(id, provider);
    if usage.is_none() && serving.host.is_none() && serving.generation_id.is_none() {
        return ProviderError::new(failure);
    }
    ProviderError::new(Failure {
        billed: Some(FailureBilled {
            usage: usage.map(|usage| d.usage(usage)),
            host: serving.host,
            generation_id: serving.generation_id,
        }),
        ..failure
    })
}

struct ChatStream<'a> {
    dialect: &'a dyn ChatDialect,
    finished: bool,
    over: bool,
    failed: Option<ProviderError>,
}

impl StreamDecoder for ChatStream<'_> {
    fn push(&mut self, item: &SseItem, events: &mut Vec<StepEvent>) -> Result<(), ProviderError> {
        if let Some(error) = &self.failed {
            return Err(error.clone());
        }
        if self.over {
            return Ok(());
        }
        let result = self.read(item, events);
        if let Err(error) = &result {
            self.over = true;
            self.failed = Some(error.clone());
        }
        result
    }

    fn end(&mut self, _events: &mut Vec<StepEvent>) -> Result<(), ProviderError> {
        if let Some(error) = &self.failed {
            return Err(error.clone());
        }
        if self.over {
            return Ok(());
        }
        self.over = true;
        let error = failed(
            FailureCategory::Truncated,
            "The stream ended before [DONE].",
            FailureExtra::default(),
        );
        self.failed = Some(error.clone());
        Err(error)
    }
}

impl ChatStream<'_> {
    fn read(&mut self, item: &SseItem, events: &mut Vec<StepEvent>) -> Result<(), ProviderError> {
        let d = self.dialect;
        match item {
            SseItem::Comment(_) => Ok(()),
            SseItem::Done(_) => {
                self.over = true;
                if self.finished {
                    return Ok(());
                }
                Err(failed(
                    FailureCategory::Stream,
                    "The provider ended the step.",
                    FailureExtra::default(),
                ))
            }
            SseItem::Data(data) => {
                let chunk = read_chunk(&parsed_chunk(&data.data)?);
                if let Some(error) = &chunk.error {
                    return Err(reply_failure(
                        d,
                        chunk.id.as_deref(),
                        chunk.provider.as_deref(),
                        chunk.usage.as_ref(),
                        error,
                    ));
                }
                chunk_events(d, &chunk, events, &mut self.finished)
            }
        }
    }
}

fn completed(d: &dyn ChatDialect, body: &Value) -> Result<StepResult, ProviderError> {
    let raw = match body {
        Value::Object(object) => object.clone(),
        Value::Array(_) => Object::new(),
        _ => {
            return Err(failed(
                FailureCategory::Stream,
                "The reply was not JSON.",
                FailureExtra::default(),
            ));
        }
    };
    let reply = read_completion(&raw);
    if let Some(error) = &reply.error {
        return Err(reply_failure(
            d,
            reply.id.as_deref(),
            reply.provider.as_deref(),
            reply.usage.as_ref(),
            error,
        ));
    }
    let mut events = Vec::new();
    reply_events(d, &reply, &mut events);
    let mut assembler = StepAssembler::new();
    for event in &events {
        assembler.add(event)?;
    }
    let result = assembler.result();
    if result.finish == FinishReason::Empty {
        return Err(failed(
            FailureCategory::Stream,
            "The provider ended the step.",
            FailureExtra::default(),
        ));
    }
    Ok(result)
}

/// Chat Completions codec for one dialect.
pub struct ChatCodec<D> {
    dialect: D,
}

pub const fn chat_codec<D: ChatDialect>(dialect: D) -> ChatCodec<D> {
    ChatCodec { dialect }
}

impl<D: ChatDialect> Codec for ChatCodec<D> {
    fn encode(&self, request: &CanonicalRequest, mode: Mode) -> Result<Encoded, String> {
        let d = &self.dialect;
        Ok(Encoded {
            path: d.path().to_owned(),
            body: body_of(d, request, mode)?,
            headers: d.headers(request),
        })
    }

    fn stream(&self) -> Box<dyn StreamDecoder + '_> {
        Box::new(ChatStream {
            dialect: &self.dialect,
            finished: false,
            over: false,
            failed: None,
        })
    }

    fn complete(&self, body: &Value) -> Result<StepResult, ProviderError> {
        completed(&self.dialect, body)
    }

    fn failure(&self, status: i64, body: &Value, headers: &HeaderList) -> Failure {
        let mut error = body.get("error").and_then(read_error).unwrap_or_default();
        let message = usable(error.message.as_deref())
            .map_or_else(|| format!("HTTP {status}"), str::to_owned);
        error.message = Some(message);
        self.dialect
            .failure(HostStatus::Http(status), &error, Some(headers))
    }
}
