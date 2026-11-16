//! Anthropic Messages codec.

use std::collections::{HashMap, HashSet};

use nyaaai_contract::json::{Object, Record, Value, stringify_json};
use nyaaai_contract::number::number_text;
use nyaaai_contract::shapes::step::{
    CanonicalRequest, CanonicalRequestCacheKind, CodecFacts, Encoded, Failure, FailureCategory,
    FinishReason, HeaderList, NormalizedUsage, Serving, SseItem, StepEvent, StepEventCall,
    StepEventFinish, StepEventText, StepEventThought, StepEventUsage, StepResult, ThoughtPart,
    ThoughtPartSegmentsItem, ThoughtPartSegmentsItemKind, ToolSpec, Turn, TurnPart, TurnStep,
};

use crate::codec::{Codec, Mode, StreamDecoder};
use crate::failure::{FailureExtra, ProviderError, WireError, failed};
use crate::loose::{
    amount_at, amount_of, error_body, errored, items_at, number_at, object_of, text_at, text_or,
};
use crate::native::{
    Host, HostStatus, arguments_of, assembled, parsed_chunk, schema_prompt, serving_events,
    stream_failure, system_texts, wire_error,
};
use crate::schema_projection::strict_schema;

const PATH: &str = "/messages";

const THINKING: &str = "thinking";
const REDACTED: &str = "redacted_thinking";

/// Default token ceiling.
const MAX_TOKENS: i64 = 32_000;

/// A role and its blocks.
struct Message {
    role: &'static str,
    content: Vec<Value>,
}

impl Message {
    fn encode(self) -> Value {
        object_of(vec![
            ("role", Value::from(self.role)),
            ("content", Value::Array(self.content)),
        ])
    }
}

fn finish_for(reason: &str) -> FinishReason {
    match reason {
        "tool_use" => FinishReason::ToolCalls,
        "max_tokens" | "model_context_window_exceeded" => FinishReason::Length,
        "refusal" => FinishReason::ContentFilter,
        _ => FinishReason::Stop,
    }
}

/// A text error is the message.
fn reply_error(body: &Object) -> WireError {
    match body.get("error") {
        Some(Value::String(text)) if !text.is_empty() => WireError {
            message: Some(text.clone()),
            ..WireError::default()
        },
        _ => wire_error(200, &Value::Object(body.clone())),
    }
}

fn claude(model: &str) -> bool {
    model
        .rfind('/')
        .map_or(model, |at| &model[at + 1..])
        .starts_with("claude-")
}

/// Allowed tool ID characters.
fn clean_id(wire_id: &str) -> String {
    let mut id = String::new();
    for char in wire_id.chars() {
        if char.is_ascii_alphanumeric() || char == '_' || char == '-' {
            id.push(char);
        } else {
            // One underscore per UTF-16 unit.
            id.extend(std::iter::repeat_n('_', char.len_utf16()));
        }
    }
    if id.is_empty() { "call".to_owned() } else { id }
}

/// One block id for each wire id of the request; ids that need no change keep theirs.
fn block_ids(history: &[Turn]) -> HashMap<String, String> {
    let wire_ids: Vec<&str> = history
        .iter()
        .flat_map(|turn| match turn {
            Turn::Step(step) => step
                .parts
                .iter()
                .filter_map(|part| match part {
                    TurnPart::Call(call) => Some(call.wire_id.as_str()),
                    _ => None,
                })
                .collect::<Vec<_>>(),
            Turn::User(_) => Vec::new(),
        })
        .collect();
    let mut mapped: HashMap<String, String> = HashMap::new();
    let mut taken: HashSet<String> = HashSet::new();
    for wire_id in &wire_ids {
        if !wire_id.is_empty() && clean_id(wire_id) == *wire_id && !mapped.contains_key(*wire_id) {
            mapped.insert((*wire_id).to_owned(), (*wire_id).to_owned());
            taken.insert((*wire_id).to_owned());
        }
    }
    for wire_id in &wire_ids {
        if mapped.contains_key(*wire_id) {
            continue;
        }
        let base = clean_id(wire_id);
        let mut id = base.clone();
        let mut count = 2;
        while taken.contains(&id) {
            id = format!("{base}_{count}");
            count += 1;
        }
        mapped.insert((*wire_id).to_owned(), id.clone());
        taken.insert(id);
    }
    mapped
}

fn block_of(blocks: &HashMap<String, String>, wire_id: &str) -> String {
    blocks
        .get(wire_id)
        .cloned()
        .unwrap_or_else(|| clean_id(wire_id))
}

fn signed(part: &ThoughtPart) -> bool {
    (part.type_ == THINKING || part.type_ == REDACTED) && part.opaque.is_some()
}

/// Skip unreadable thoughts locally.
fn readable(step: &TurnStep, request: &CanonicalRequest) -> bool {
    let Some(producer) = step
        .producer
        .as_ref()
        .map(|producer| producer.model.as_str())
    else {
        return false;
    };
    let signed_all = step.parts.iter().all(|part| match part {
        TurnPart::Thought(thought) => signed(thought),
        _ => true,
    });
    signed_all && (producer == request.model || (claude(producer) && claude(&request.model)))
}

fn thought_block(part: &ThoughtPart) -> Value {
    let opaque = part.opaque.clone().unwrap_or_default();
    if part.type_ == REDACTED {
        return object_of(vec![
            ("type", Value::from(REDACTED)),
            ("data", Value::String(opaque)),
        ]);
    }
    let text: String = part
        .segments
        .iter()
        .flatten()
        .map(|segment| segment.text.as_str())
        .collect();
    object_of(vec![
        ("type", Value::from(THINKING)),
        ("thinking", Value::String(text)),
        ("signature", Value::String(opaque)),
    ])
}

fn text_block(text: &str) -> Value {
    object_of(vec![
        ("type", Value::from("text")),
        ("text", Value::from(text)),
    ])
}

/// User-turn tool results.
fn step_messages(
    step: &TurnStep,
    request: &CanonicalRequest,
    blocks: &HashMap<String, String>,
) -> Vec<Message> {
    let sent = readable(step, request);
    let mut ids: Vec<(f64, &str)> = Vec::new();
    for part in &step.parts {
        if let TurnPart::Call(call) = part {
            match ids.iter_mut().find(|(index, _)| *index == call.index) {
                Some(held) => held.1 = &call.wire_id,
                None => ids.push((call.index, &call.wire_id)),
            }
        }
    }
    let mut content = Vec::new();
    for part in &step.parts {
        match part {
            TurnPart::Thought(thought) if sent => content.push(thought_block(thought)),
            TurnPart::Text(text) if !text.text.is_empty() => content.push(text_block(&text.text)),
            TurnPart::Call(call) => content.push(object_of(vec![
                ("type", Value::from("tool_use")),
                ("id", Value::String(block_of(blocks, &call.wire_id))),
                ("name", Value::from(call.name.as_str())),
                ("input", Value::Object(arguments_of(&call.arguments))),
            ])),
            _ => {}
        }
    }
    let mut results = Vec::new();
    for part in &step.parts {
        let TurnPart::Result(result) = part else {
            continue;
        };
        let Some((_, wire_id)) = ids.iter().find(|(index, _)| *index == result.index) else {
            continue;
        };
        let mut entries = vec![
            ("type", Value::from("tool_result")),
            ("tool_use_id", Value::String(block_of(blocks, wire_id))),
            ("content", Value::from(result.output.as_str())),
        ];
        if result.is_error {
            entries.push(("is_error", Value::Bool(true)));
        }
        results.push(object_of(entries));
    }
    let mut messages = Vec::new();
    if !content.is_empty() {
        messages.push(Message {
            role: "assistant",
            content,
        });
    }
    if !results.is_empty() {
        messages.push(Message {
            role: "user",
            content: results,
        });
    }
    messages
}

fn turn_messages(
    turn: &Turn,
    request: &CanonicalRequest,
    blocks: &HashMap<String, String>,
) -> Vec<Message> {
    match turn {
        Turn::User(user) => vec![Message {
            role: "user",
            content: vec![text_block(&user.text)],
        }],
        Turn::Step(step) => step_messages(step, request, blocks),
    }
}

/// Join messages by role.
fn joined(messages: Vec<Message>) -> Vec<Message> {
    let mut out: Vec<Message> = Vec::new();
    for message in messages {
        match out.last_mut() {
            Some(last) if last.role == message.role => last.content.extend(message.content),
            _ => out.push(message),
        }
    }
    out
}

fn with_cache(block: &Value, control: &Value) -> Value {
    let mut held = block.as_object().cloned().unwrap_or_default();
    held.insert("cache_control", control.clone());
    Value::Object(held)
}

/// Cache last user block.
fn with_last_cached(mut messages: Vec<Message>, control: &Value) -> Vec<Message> {
    let Some(at) = messages.iter().rposition(|message| message.role == "user") else {
        return messages;
    };
    let Some(last) = messages[at].content.last() else {
        return messages;
    };
    let type_ = last.get("type").and_then(Value::as_str);
    if type_ != Some("text") && type_ != Some("tool_result") {
        return messages;
    }
    let marked = with_cache(last, control);
    if let Some(slot) = messages[at].content.last_mut() {
        *slot = marked;
    }
    messages
}

/// State follows results.
fn with_state(mut messages: Vec<Message>, state: Option<&str>) -> Vec<Message> {
    let Some(state) = state else {
        return messages;
    };
    messages.push(Message {
        role: "user",
        content: vec![text_block(state)],
    });
    joined(messages)
}

fn tool_of(tool: &ToolSpec) -> Value {
    object_of(vec![
        ("name", Value::from(tool.name.as_str())),
        ("description", Value::from(tool.description.as_str())),
        ("input_schema", Value::Object(tool.parameters.clone())),
    ])
}

/// Tool-choice mode mapping.
fn choice_of(choice: &[String], name: &str) -> Option<Value> {
    if choice.iter().any(|held| held == "tool") {
        return Some(object_of(vec![
            ("type", Value::from("tool")),
            ("name", Value::from(name)),
        ]));
    }
    choice
        .iter()
        .any(|held| held == "required")
        .then(|| object_of(vec![("type", Value::from("any"))]))
}

/// Explicit prompt caching.
fn body_of(
    facts: Option<&CodecFacts>,
    request: &CanonicalRequest,
    mode: Mode,
) -> Result<Value, String> {
    let api = facts.map(|facts| &facts.api);
    let efforts = facts.map_or(&[][..], |facts| &facts.efforts);
    let enforced = match &request.schema {
        Some(schema) if api.is_some_and(|api| api.structured) => Some(strict_schema(&schema.json)?),
        _ => None,
    };
    let prompted = request
        .schema
        .as_ref()
        .filter(|_| enforced.is_none())
        .map(|schema| schema_prompt(&schema.json));
    let control = (request.cache.kind == CanonicalRequestCacheKind::Explicit).then(|| {
        let mut entries = vec![("type", Value::from("ephemeral"))];
        if request.cache.ttl.is_some() {
            entries.push(("ttl", Value::from("1h")));
        }
        object_of(entries)
    });
    let texts = system_texts(request, prompted.as_deref());
    let last_system = texts
        .iter()
        .enumerate()
        .rposition(|(at, text)| at < request.system.len() && !text.is_empty());
    let marked = |at: usize| {
        last_system == Some(at) || (request.summary.is_some() && at == request.system.len())
    };
    let mut system = Vec::new();
    for (at, text) in texts.iter().enumerate() {
        if text.is_empty() {
            continue;
        }
        let block = text_block(text);
        system.push(match &control {
            Some(control) if marked(at) => with_cache(&block, control),
            _ => block,
        });
    }
    let blocks = block_ids(&request.history);
    let history = joined(
        request
            .history
            .iter()
            .flat_map(|turn| turn_messages(turn, request, &blocks))
            .collect(),
    );
    let history = match &control {
        Some(control) => with_last_cached(history, control),
        None => history,
    };
    let messages = with_state(history, request.state.as_deref());
    let forced = request
        .tool_choice
        .as_ref()
        .and_then(|choice| choice_of(api.map_or(&[][..], |api| &api.choice), &choice.name));
    let listed = request.effort.filter(|effort| efforts.contains(effort));
    let mut output = Vec::new();
    if let Some(effort) = listed {
        output.push(("effort", Value::from(effort.as_str())));
    }
    if let Some(enforced) = enforced {
        output.push((
            "format",
            object_of(vec![
                ("type", Value::from("json_schema")),
                ("schema", Value::Object(enforced)),
            ]),
        ));
    }
    let mut body = Object::new();
    body.insert("model", Value::from(request.model.as_str()));
    body.insert(
        "max_tokens",
        Value::Number(
            request
                .max_tokens
                .or(facts.and_then(|facts| facts.output_limit))
                .unwrap_or(MAX_TOKENS) as f64,
        ),
    );
    if !system.is_empty() {
        body.insert("system", Value::Array(system));
    }
    body.insert(
        "messages",
        Value::Array(messages.into_iter().map(Message::encode).collect()),
    );
    if let Some(tools) = request.tools.as_ref().filter(|tools| !tools.is_empty()) {
        body.insert("tools", Value::Array(tools.iter().map(tool_of).collect()));
    }
    if let Some(forced) = forced {
        body.insert("tool_choice", forced);
    }
    // Claude 5 requires thinking.
    if !efforts.is_empty() && claude(&request.model) {
        body.insert(
            "thinking",
            object_of(vec![
                ("type", Value::from("adaptive")),
                ("display", Value::from("summarized")),
            ]),
        );
    }
    if !output.is_empty() {
        body.insert("output_config", object_of(output));
    }
    body.insert("stream", Value::Bool(mode == Mode::Stream));
    Ok(Value::Object(body))
}

/// Input includes cache.
fn usage_of(host: &dyn Host, raw: &Object) -> NormalizedUsage {
    let cached = amount_at(raw, "cache_read_input_tokens");
    let cache_write = amount_at(raw, "cache_creation_input_tokens");
    host.usage(
        NormalizedUsage {
            input: amount_at(raw, "input_tokens").unwrap_or(0.0)
                + cached.unwrap_or(0.0)
                + cache_write.unwrap_or(0.0),
            output: amount_at(raw, "output_tokens").unwrap_or(0.0),
            cached,
            cache_write,
            reasoning: None,
            cost: None,
            upstream_cost: None,
            byok: None,
            billing_source: None,
            list_cost: None,
            allowance: None,
        },
        raw,
    )
}

fn finish_of(reason: Option<&Value>) -> Option<FinishReason> {
    match reason {
        Some(Value::String(text)) if !text.is_empty() => Some(finish_for(text)),
        _ => None,
    }
}

fn thought_of(block: &str, part: ThoughtPart) -> StepEvent {
    StepEvent::Thought(StepEventThought {
        part,
        block: Some(block.to_owned()),
        split: None,
    })
}

fn part_of(type_: &str, text: Option<&str>, opaque: Option<String>) -> ThoughtPart {
    ThoughtPart {
        type_: type_.to_owned(),
        segments: text.map(|text| {
            vec![ThoughtPartSegmentsItem {
                kind: ThoughtPartSegmentsItemKind::Content,
                text: text.to_owned(),
            }]
        }),
        opaque,
        id: None,
        format: None,
        index: None,
        anchor: None,
    }
}

type Events<'e> = &'e mut Vec<StepEvent>;

/// Block-indexed message events.
struct Decoder<'a> {
    host: &'a dyn Host,
    finished: bool,
    calls: Vec<(f64, usize)>,
    called: usize,
    serving: Serving,
    raw: Object,
    usage: Option<NormalizedUsage>,
    finish: Option<FinishReason>,
}

impl<'a> Decoder<'a> {
    fn new(host: &'a dyn Host) -> Self {
        Self {
            host,
            finished: false,
            calls: Vec::new(),
            called: 0,
            serving: Serving {
                host: None,
                generation_id: None,
            },
            raw: Object::new(),
            usage: None,
            finish: None,
        }
    }

    fn event(
        &mut self,
        type_: Option<&str>,
        data: &Object,
        out: Events,
    ) -> Result<(), ProviderError> {
        if errored(data.get("error")) {
            return Err(self.failure(&error_body(data.get("error"))));
        }
        let index = number_at(data, "index").unwrap_or(0.0);
        match type_.unwrap_or("") {
            "message_start" => {
                if let Some(Value::Object(message)) = data.get("message") {
                    self.started(message, out);
                }
            }
            "content_block_start" => {
                if let Some(Value::Object(block)) = data.get("content_block") {
                    self.block(index, block, false, out);
                }
            }
            "content_block_delta" => {
                if let Some(Value::Object(delta)) = data.get("delta") {
                    self.delta(index, delta, out);
                }
            }
            "message_delta" => {
                self.counted(data.get("usage"), out);
                let reason = match data.get("delta") {
                    Some(Value::Object(delta)) => delta.get("stop_reason"),
                    _ => None,
                };
                self.stopped(reason, out);
            }
            "message_stop" => {
                if self.finish.is_none() {
                    return Err(failed(
                        FailureCategory::Stream,
                        "The provider ended the step.",
                        FailureExtra::default(),
                    ));
                }
                self.finished = true;
            }
            "error" => return Err(self.failure(&Object::new())),
            _ => {}
        }
        Ok(())
    }

    fn failure(&self, error: &Object) -> ProviderError {
        stream_failure(
            self.host,
            &WireError {
                message: Some(text_or(text_at(error, "message"), "The message failed.")),
                type_: error.get("type").and_then(Value::as_str).map(str::to_owned),
                ..WireError::default()
            },
            self.usage.clone(),
            &self.serving,
        )
    }

    /// Decode a complete message.
    fn whole(&mut self, message: &Object, out: Events) {
        let mut head = Object::new();
        if let Some(id) = message.get("id") {
            head.insert("id", id.clone());
        }
        self.started(&head, out);
        for (index, block) in items_at(message, "content").iter().enumerate() {
            if let Value::Object(block) = block {
                self.block(index as f64, block, true, out);
            }
        }
        self.counted(message.get("usage"), out);
        self.stopped(message.get("stop_reason"), out);
    }

    fn started(&mut self, message: &Object, out: Events) {
        let id = message.get("id").cloned().unwrap_or(Value::Null);
        for event in serving_events(&id) {
            if let StepEvent::Serving(held) = &event
                && held.serving.generation_id.is_some()
            {
                self.serving
                    .generation_id
                    .clone_from(&held.serving.generation_id);
            }
            out.push(event);
        }
        self.counted(message.get("usage"), out);
    }

    /// Merge cumulative usage.
    fn counted(&mut self, usage: Option<&Value>, out: Events) {
        let Some(Value::Object(usage)) = usage else {
            return;
        };
        for (key, value) in usage.iter() {
            if value.as_object().is_some() || amount_of(value).is_some() {
                self.raw.insert(key, value.clone());
            }
        }
        self.usage = Some(usage_of(self.host, &self.raw));
        if let Some(usage) = &self.usage {
            out.push(StepEvent::Usage(StepEventUsage {
                usage: usage.clone(),
            }));
        }
    }

    fn stopped(&mut self, reason: Option<&Value>, out: Events) {
        let Some(finish) = finish_of(reason) else {
            return;
        };
        self.finish = Some(finish);
        out.push(StepEvent::Finish(StepEventFinish { reason: finish }));
    }

    /// Deltas carry streamed content.
    fn block(&mut self, index: f64, block: &Object, whole: bool, out: Events) {
        let id = number_text(index);
        match block.get("type").and_then(Value::as_str) {
            Some("text") => out.push(StepEvent::Text(StepEventText {
                text: text_at(block, "text").to_owned(),
                phase: None,
            })),
            Some(THINKING) => {
                if !whole {
                    return;
                }
                let text = text_at(block, "thinking");
                let opaque = text_at(block, "signature");
                out.push(thought_of(
                    &id,
                    part_of(
                        THINKING,
                        (!text.is_empty()).then_some(text),
                        (!opaque.is_empty()).then(|| opaque.to_owned()),
                    ),
                ));
            }
            Some(REDACTED) => out.push(thought_of(
                &id,
                part_of(REDACTED, None, Some(text_at(block, "data").to_owned())),
            )),
            Some("tool_use") => {
                let distinct = self.called;
                self.called += 1;
                let call = match self.calls.iter_mut().find(|(held, _)| *held == index) {
                    Some(held) => {
                        held.1 = distinct;
                        distinct
                    }
                    None => {
                        self.calls.push((index, distinct));
                        distinct
                    }
                };
                let input = match block.get("input") {
                    Some(Value::Object(input)) => Value::Object(input.clone()),
                    _ => Value::Object(Object::new()),
                };
                out.push(StepEvent::Call(StepEventCall {
                    index: call as f64,
                    wire_id: text_at(block, "id").to_owned(),
                    name: text_at(block, "name").to_owned(),
                    arguments_text: if whole {
                        stringify_json(&input)
                    } else {
                        String::new()
                    },
                }));
            }
            _ => {}
        }
    }

    fn delta(&mut self, index: f64, delta: &Object, out: Events) {
        let id = number_text(index);
        match delta.get("type").and_then(Value::as_str) {
            Some("text_delta") => out.push(StepEvent::Text(StepEventText {
                text: text_at(delta, "text").to_owned(),
                phase: None,
            })),
            Some("thinking_delta") => {
                let text = text_at(delta, "thinking");
                if !text.is_empty() {
                    out.push(thought_of(&id, part_of(THINKING, Some(text), None)));
                }
            }
            Some("signature_delta") => out.push(thought_of(
                &id,
                part_of(THINKING, None, Some(text_at(delta, "signature").to_owned())),
            )),
            Some("input_json_delta") => {
                if let Some((_, call)) = self.calls.iter().find(|(held, _)| *held == index) {
                    out.push(StepEvent::Call(StepEventCall {
                        index: *call as f64,
                        wire_id: String::new(),
                        name: String::new(),
                        arguments_text: text_at(delta, "partial_json").to_owned(),
                    }));
                }
            }
            _ => {}
        }
    }
}

struct MessagesStream<'a> {
    decoder: Decoder<'a>,
    over: bool,
    failed: Option<ProviderError>,
}

impl StreamDecoder for MessagesStream<'_> {
    fn push(&mut self, item: &SseItem, events: &mut Vec<StepEvent>) -> Result<(), ProviderError> {
        if let Some(error) = &self.failed {
            return Err(error.clone());
        }
        if self.over {
            return Ok(());
        }
        let result = match item {
            SseItem::Comment(_) => Ok(()),
            SseItem::Done(_) => {
                self.over = true;
                Ok(())
            }
            SseItem::Data(data) => parsed_chunk(&data.data).and_then(|chunk| {
                let type_ = match chunk.get("type") {
                    None | Some(Value::Null) => data.event.as_deref(),
                    Some(Value::String(text)) => Some(text.as_str()),
                    Some(_) => None,
                };
                self.decoder.event(type_, &chunk, events)
            }),
        };
        if let Err(error) = &result {
            self.failed = Some(error.clone());
        }
        if self.failed.is_some() || self.decoder.finished {
            self.over = true;
        }
        result
    }

    fn end(&mut self, _events: &mut Vec<StepEvent>) -> Result<(), ProviderError> {
        if let Some(error) = &self.failed {
            return Err(error.clone());
        }
        if self.decoder.finished {
            return Ok(());
        }
        let error = failed(
            FailureCategory::Truncated,
            "The stream ended before message_stop.",
            FailureExtra::default(),
        );
        self.failed = Some(error.clone());
        Err(error)
    }
}

/// Anthropic Messages codec for one host.
pub struct MessagesCodec<H> {
    host: H,
    facts: Option<CodecFacts>,
}

pub fn messages_codec<H: Host>(host: H, facts: Option<&CodecFacts>) -> MessagesCodec<H> {
    MessagesCodec {
        host,
        facts: facts.cloned(),
    }
}

impl<H: Host> Codec for MessagesCodec<H> {
    fn encode(&self, request: &CanonicalRequest, mode: Mode) -> Result<Encoded, String> {
        let mut headers = Record::new();
        for (key, value) in self.host.headers(request) {
            headers.insert(key, value);
        }
        Ok(Encoded {
            path: format!("{}{PATH}", self.host.prefix()),
            body: body_of(self.facts.as_ref(), request, mode)?,
            headers: Some(headers),
        })
    }

    fn stream(&self) -> Box<dyn StreamDecoder + '_> {
        Box::new(MessagesStream {
            decoder: Decoder::new(&self.host),
            over: false,
            failed: None,
        })
    }

    fn complete(&self, body: &Value) -> Result<StepResult, ProviderError> {
        let Value::Object(body) = body else {
            return Err(failed(
                FailureCategory::Stream,
                "The reply was not JSON.",
                FailureExtra::default(),
            ));
        };
        if body.get("type").and_then(Value::as_str) == Some("error") || errored(body.get("error")) {
            return Err(stream_failure(
                &self.host,
                &reply_error(body),
                None,
                &Serving {
                    host: None,
                    generation_id: None,
                },
            ));
        }
        let mut decoder = Decoder::new(&self.host);
        let mut events = Vec::new();
        decoder.whole(body, &mut events);
        assembled(&events)
    }

    fn failure(&self, status: i64, body: &Value, headers: &HeaderList) -> Failure {
        self.host.failure(
            HostStatus::Http(status),
            &wire_error(status, body),
            Some(headers),
        )
    }
}
