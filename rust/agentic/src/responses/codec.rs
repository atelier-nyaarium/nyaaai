//! OpenAI Responses codec.

use std::collections::HashMap;

use nyaaai_contract::json::{Object, Record, Value};
use nyaaai_contract::number::number_text;
use nyaaai_contract::shapes::step::{
    CanonicalRequest, CodecFacts, Encoded, Failure, FailureCategory, FinishReason, HeaderList,
    NormalizedUsage, Serving, SseItem, StepEvent, StepEventCall, StepEventFinish, StepEventText,
    StepEventThought, StepEventUsage, StepResult, ThoughtPart, ThoughtPartSegmentsItem,
    ThoughtPartSegmentsItemKind, ToolSpec, Turn, TurnPart, TurnStep,
};

use crate::codec::{Codec, Mode, StreamDecoder};
use crate::failure::{FailureExtra, ProviderError, WireError, failed};
use crate::loose::{
    amount_at, error_body, errored, items_at, number_at, object_at, object_of, text_at, text_or,
    type_of,
};
use crate::native::{
    Host, HostStatus, assembled, parsed_chunk, schema_prompt, serving_events, stream_failure,
    system_texts, wire_error,
};
use crate::schema_projection::strict_schema;

const PATH: &str = "/responses";

const REASONING: &str = "reasoning";

const MIN_TOKENS: i64 = 16;

/// Shared model family IDs.
fn family_of(model: &str) -> String {
    let name = model.rfind('/').map_or(model, |at| &model[at + 1..]);
    name.split('-').take(2).collect::<Vec<_>>().join("-")
}

fn replayable(part: &ThoughtPart) -> bool {
    part.type_ == REASONING && part.id.is_some() && part.opaque.is_some()
}

fn reasoning_item(part: &ThoughtPart) -> Value {
    let segments = part.segments.as_deref().unwrap_or(&[]);
    let of = |kind: ThoughtPartSegmentsItemKind, type_: &str| -> Vec<Value> {
        segments
            .iter()
            .filter(|segment| segment.kind == kind)
            .map(|segment| {
                object_of(vec![
                    ("type", Value::from(type_)),
                    ("text", Value::from(segment.text.as_str())),
                ])
            })
            .collect()
    };
    let content = of(ThoughtPartSegmentsItemKind::Content, "reasoning_text");
    let mut entries = vec![
        ("type", Value::from("reasoning")),
        ("id", Value::from(part.id.clone().unwrap_or_default())),
        (
            "summary",
            Value::Array(of(ThoughtPartSegmentsItemKind::Summary, "summary_text")),
        ),
    ];
    if !content.is_empty() {
        entries.push(("content", Value::Array(content)));
    }
    entries.push((
        "encrypted_content",
        Value::from(part.opaque.clone().unwrap_or_default()),
    ));
    object_of(entries)
}

/// Same-family reasoning replay.
fn step_items(step: &TurnStep, request: &CanonicalRequest) -> Vec<Value> {
    let thoughts: Vec<&ThoughtPart> = step
        .parts
        .iter()
        .filter_map(|part| match part {
            TurnPart::Thought(thought) => Some(thought),
            _ => None,
        })
        .collect();
    let sent = step
        .producer
        .as_ref()
        .is_some_and(|producer| family_of(&producer.model) == family_of(&request.model))
        && thoughts.iter().all(|part| replayable(part));
    let mut wire_of: Vec<(f64, &str)> = Vec::new();
    for part in &step.parts {
        if let TurnPart::Call(call) = part {
            match wire_of.iter_mut().find(|(index, _)| *index == call.index) {
                Some(held) => held.1 = &call.wire_id,
                None => wire_of.push((call.index, &call.wire_id)),
            }
        }
    }
    let phase = if wire_of.is_empty() {
        "final_answer"
    } else {
        "commentary"
    };
    let mut items = Vec::new();
    for part in &step.parts {
        match part {
            TurnPart::Thought(thought) if sent && replayable(thought) => {
                items.push(reasoning_item(thought));
            }
            TurnPart::Text(text) if !text.text.is_empty() => items.push(object_of(vec![
                ("type", Value::from("message")),
                ("role", Value::from("assistant")),
                ("content", Value::from(text.text.as_str())),
                ("phase", Value::from(text.phase.as_deref().unwrap_or(phase))),
            ])),
            TurnPart::Call(call) => items.push(object_of(vec![
                ("type", Value::from("function_call")),
                ("call_id", Value::from(call.wire_id.as_str())),
                ("name", Value::from(call.name.as_str())),
                ("arguments", Value::from(call.arguments.as_str())),
            ])),
            _ => {}
        }
    }
    for part in &step.parts {
        let TurnPart::Result(result) = part else {
            continue;
        };
        if let Some((_, call_id)) = wire_of.iter().find(|(index, _)| *index == result.index) {
            items.push(object_of(vec![
                ("type", Value::from("function_call_output")),
                ("call_id", Value::from(*call_id)),
                ("output", Value::from(result.output.as_str())),
            ]));
        }
    }
    items
}

fn turn_items(turn: &Turn, request: &CanonicalRequest) -> Vec<Value> {
    match turn {
        Turn::User(user) => vec![object_of(vec![
            ("type", Value::from("message")),
            ("role", Value::from("user")),
            ("content", Value::from(user.text.as_str())),
        ])],
        Turn::Step(step) => step_items(step, request),
    }
}

fn tool_of(tool: &ToolSpec) -> Value {
    object_of(vec![
        ("type", Value::from("function")),
        ("name", Value::from(tool.name.as_str())),
        ("description", Value::from(tool.description.as_str())),
        ("parameters", Value::Object(tool.parameters.clone())),
    ])
}

/// Named or any-tool choice.
fn choice_of(choice: &[String], name: &str) -> Option<Value> {
    if choice.iter().any(|held| held == "tool") {
        return Some(object_of(vec![
            ("type", Value::from("function")),
            ("name", Value::from(name)),
        ]));
    }
    choice
        .iter()
        .any(|held| held == "required")
        .then(|| Value::from("required"))
}

/// Catalog gates optional fields.
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
    let instructions = system_texts(request, prompted.as_deref()).join("\n\n");
    let forced = request
        .tool_choice
        .as_ref()
        .and_then(|choice| choice_of(api.map_or(&[][..], |api| &api.choice), &choice.name));
    let reasoning = request
        .effort
        .filter(|effort| efforts.contains(effort))
        .map(|effort| {
            let mut entries = vec![("effort", Value::from(effort.as_str()))];
            if effort.as_str() != "none" {
                entries.push(("summary", Value::from("auto")));
            }
            object_of(entries)
        });
    let mut input: Vec<Value> = request
        .history
        .iter()
        .flat_map(|turn| turn_items(turn, request))
        .collect();
    if let Some(state) = &request.state {
        input.push(object_of(vec![
            ("type", Value::from("message")),
            ("role", Value::from("developer")),
            ("content", Value::from(state.as_str())),
        ]));
    }
    let mut body = Object::new();
    body.insert("model", Value::from(request.model.as_str()));
    if !instructions.is_empty() {
        body.insert("instructions", Value::String(instructions));
    }
    body.insert("input", Value::Array(input));
    if let Some(tools) = request.tools.as_ref().filter(|tools| !tools.is_empty()) {
        body.insert("tools", Value::Array(tools.iter().map(tool_of).collect()));
    }
    if let Some(forced) = forced {
        body.insert("tool_choice", forced);
    }
    if let Some(reasoning) = reasoning {
        body.insert("reasoning", reasoning);
    }
    if let (Some(schema), Some(enforced)) = (&request.schema, enforced) {
        body.insert(
            "text",
            object_of(vec![(
                "format",
                object_of(vec![
                    ("type", Value::from("json_schema")),
                    ("name", Value::from(schema.name.as_str())),
                    ("schema", Value::Object(enforced)),
                    ("strict", Value::Bool(true)),
                ]),
            )]),
        );
    }
    body.insert("store", Value::Bool(false));
    body.insert(
        "include",
        Value::Array(vec![Value::from("reasoning.encrypted_content")]),
    );
    if let Some(max) = request.max_tokens {
        body.insert(
            "max_output_tokens",
            Value::Number(max.max(MIN_TOKENS) as f64),
        );
    }
    body.insert("stream", Value::Bool(mode == Mode::Stream));
    Ok(Value::Object(body))
}

fn usage_of(host: &dyn Host, raw: Option<&Value>) -> Option<NormalizedUsage> {
    let raw = raw?.as_object()?;
    let empty = Object::new();
    let prompt = object_at(raw, "input_tokens_details").unwrap_or(&empty);
    let completion = object_at(raw, "output_tokens_details").unwrap_or(&empty);
    Some(host.usage(
        NormalizedUsage {
            input: amount_at(raw, "input_tokens").unwrap_or(0.0),
            output: amount_at(raw, "output_tokens").unwrap_or(0.0),
            cached: amount_at(prompt, "cached_tokens"),
            cache_write: amount_at(prompt, "cache_write_tokens"),
            reasoning: amount_at(completion, "reasoning_tokens"),
            cost: None,
            upstream_cost: None,
            byok: None,
            billing_source: None,
            list_cost: None,
            allowance: None,
        },
        raw,
    ))
}

/// Output text and refusals.
fn message_text(item: &Object) -> String {
    items_at(item, "content")
        .iter()
        .map(|part| match part {
            Value::Object(part) => {
                let text = text_at(part, "text");
                if text.is_empty() {
                    text_at(part, "refusal")
                } else {
                    text
                }
            }
            _ => "",
        })
        .collect()
}

fn segments_of(item: &Object) -> Vec<ThoughtPartSegmentsItem> {
    let of = |key: &str, kind: ThoughtPartSegmentsItemKind| {
        items_at(item, key)
            .iter()
            .filter_map(move |part| match part {
                Value::Object(part) => match part.get("text") {
                    Some(Value::String(text)) => Some(ThoughtPartSegmentsItem {
                        kind,
                        text: text.clone(),
                    }),
                    _ => None,
                },
                _ => None,
            })
            .collect::<Vec<_>>()
    };
    let mut out = of("summary", ThoughtPartSegmentsItemKind::Summary);
    out.extend(of("content", ThoughtPartSegmentsItemKind::Content));
    out
}

struct CallState {
    index: usize,
    arguments_text: String,
}

/// What one output item has shown so far.
#[derive(Default)]
struct ItemState {
    call: Option<CallState>,
    thought: Option<ThoughtState>,
    phase: Option<String>,
    texted: bool,
}

#[derive(Default)]
struct ThoughtState {
    summary: Option<f64>,
    content: Option<f64>,
    streamed: bool,
}

fn thought_event(part: ThoughtPart, block: &str, split: bool) -> StepEvent {
    StepEvent::Thought(StepEventThought {
        part,
        block: Some(block.to_owned()),
        split: split.then_some(true),
    })
}

fn bare_thought(
    segments: Option<Vec<ThoughtPartSegmentsItem>>,
    opaque: Option<String>,
    id: &str,
) -> ThoughtPart {
    ThoughtPart {
        type_: REASONING.to_owned(),
        segments,
        opaque,
        id: Some(id.to_owned()),
        format: None,
        index: None,
        anchor: None,
    }
}

fn finish_of(status: &str, details: Option<&Value>, called: bool) -> FinishReason {
    if status == "completed" {
        return if called {
            FinishReason::ToolCalls
        } else {
            FinishReason::Stop
        };
    }
    match details {
        Some(Value::Object(held))
            if held.get("reason").and_then(Value::as_str) == Some("content_filter") =>
        {
            FinishReason::ContentFilter
        }
        _ => FinishReason::Length,
    }
}

/// Where an item sits: its id, its call id, its place in the output.
fn keys_of(id: &str, call_id: &str, place: Option<f64>) -> Vec<String> {
    let mut keys = Vec::new();
    if !id.is_empty() {
        keys.push(format!("id:{id}"));
    }
    if !call_id.is_empty() {
        keys.push(format!("call:{call_id}"));
    }
    if let Some(place) = place {
        keys.push(format!("at:{}", number_text(place)));
    }
    keys
}

/// A label for a thought block.
fn label_of(id: &str, place: Option<f64>) -> String {
    match (id.is_empty(), place) {
        (false, _) => id.to_owned(),
        (true, Some(place)) => format!("at:{}", number_text(place)),
        (true, None) => String::new(),
    }
}

/// Final text counts only when there is some.
fn final_of(value: Option<&Value>) -> Option<&str> {
    match value {
        Some(Value::String(text)) if !text.is_empty() => Some(text.as_str()),
        _ => None,
    }
}

/// Response items keyed by id, else by place.
struct Decoder<'a> {
    host: &'a dyn Host,
    finished: bool,
    items: Vec<ItemState>,
    keys: HashMap<String, usize>,
    called: usize,
    serving: Serving,
    usage: Option<NormalizedUsage>,
}

type Events<'e> = &'e mut Vec<StepEvent>;

impl<'a> Decoder<'a> {
    fn new(host: &'a dyn Host) -> Self {
        Self {
            host,
            finished: false,
            items: Vec::new(),
            keys: HashMap::new(),
            called: 0,
            serving: Serving {
                host: None,
                generation_id: None,
            },
            usage: None,
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
        let item_id = text_at(data, "item_id").to_owned();
        let place = number_at(data, "output_index");
        let keys = keys_of(&item_id, "", place);
        let label = label_of(&item_id, place);
        let delta = text_at(data, "delta").to_owned();
        match type_.unwrap_or("") {
            "response.created" | "response.in_progress" => self.served(data.get("response"), out),
            "response.output_item.added" => {
                if let Some(Value::Object(item)) = data.get("item") {
                    self.added(item, place, out)?;
                }
            }
            "response.output_item.done" => {
                if let Some(Value::Object(item)) = data.get("item") {
                    self.done(item, place, out)?;
                }
            }
            "response.output_text.delta" | "response.refusal.delta" => {
                let at = self.state(&keys);
                self.items[at].texted = true;
                out.push(StepEvent::Text(StepEventText {
                    text: delta,
                    phase: self.items[at].phase.clone(),
                }));
            }
            "response.function_call_arguments.delta" => {
                let at = self.state(&keys);
                self.argued(at, None, Some(&delta), None, out)?;
            }
            "response.function_call_arguments.done" => {
                let at = self.state(&keys);
                self.argued(at, None, None, final_of(data.get("arguments")), out)?;
            }
            "response.reasoning_summary_text.delta" => {
                let at = number_at(data, "summary_index").unwrap_or(0.0);
                let state = self.state(&keys);
                out.push(self.thought(state, &item_id, &label, true, at, &delta));
            }
            "response.reasoning_text.delta" => {
                let at = number_at(data, "content_index").unwrap_or(0.0);
                let state = self.state(&keys);
                out.push(self.thought(state, &item_id, &label, false, at, &delta));
            }
            "response.completed" => return self.ended(data.get("response"), "completed", out),
            "response.incomplete" => return self.ended(data.get("response"), "incomplete", out),
            "response.failed" => return self.ended(data.get("response"), "failed", out),
            "error" => return Err(self.failure(data)),
            _ => {}
        }
        Ok(())
    }

    /// Decode a complete response.
    fn whole(&mut self, response: &Object, out: Events) -> Result<(), ProviderError> {
        self.served(Some(&Value::Object(response.clone())), out);
        for (place, item) in items_at(response, "output").iter().enumerate() {
            if let Value::Object(item) = item {
                self.done(item, Some(place as f64), out)?;
            }
        }
        self.ended(Some(&Value::Object(response.clone())), "", out)
    }

    fn failure(&self, error: &Object) -> ProviderError {
        stream_failure(
            self.host,
            &WireError {
                message: Some(text_or(text_at(error, "message"), "The response failed.")),
                type_: type_of(error.get("code")),
                ..WireError::default()
            },
            self.usage.clone(),
            &self.serving,
        )
    }

    /// The state every key names, made when none has one.
    fn state(&mut self, keys: &[String]) -> usize {
        let held = keys.iter().find_map(|key| self.keys.get(key).copied());
        let at = held.unwrap_or_else(|| {
            self.items.push(ItemState::default());
            self.items.len() - 1
        });
        for key in keys {
            self.keys.entry(key.clone()).or_insert(at);
        }
        at
    }

    fn served(&mut self, response: Option<&Value>, out: Events) {
        let Some(Value::Object(response)) = response else {
            return;
        };
        if self.serving.generation_id.is_some() {
            return;
        }
        let id = response.get("id").cloned().unwrap_or(Value::Null);
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
    }

    fn added(
        &mut self,
        item: &Object,
        place: Option<f64>,
        out: Events,
    ) -> Result<(), ProviderError> {
        let type_ = item.get("type").and_then(Value::as_str);
        let call = type_ == Some("function_call");
        let call_id = if call { text_at(item, "call_id") } else { "" };
        let at = self.state(&keys_of(text_at(item, "id"), call_id, place));
        if type_ == Some("message")
            && let Some(Value::String(phase)) = item.get("phase")
        {
            self.items[at].phase = Some(phase.clone());
        }
        if call {
            self.argued(at, Some(item), Some(text_at(item, "arguments")), None, out)?;
        }
        Ok(())
    }

    fn done(
        &mut self,
        item: &Object,
        place: Option<f64>,
        out: Events,
    ) -> Result<(), ProviderError> {
        let id = text_at(item, "id");
        let type_ = item.get("type").and_then(Value::as_str);
        let call = type_ == Some("function_call");
        let call_id = if call { text_at(item, "call_id") } else { "" };
        let at = self.state(&keys_of(id, call_id, place));
        if call {
            self.argued(at, Some(item), None, final_of(item.get("arguments")), out)?;
        }
        if type_ == Some("message") && !self.items[at].texted {
            if let Some(Value::String(phase)) = item.get("phase")
                && self.items[at].phase.is_none()
            {
                self.items[at].phase = Some(phase.clone());
            }
            self.items[at].texted = true;
            out.push(StepEvent::Text(StepEventText {
                text: message_text(item),
                phase: self.items[at].phase.clone(),
            }));
        }
        if type_ == Some(REASONING) {
            self.reasoned(at, id, &label_of(id, place), item, out);
        }
        Ok(())
    }

    /// Final arguments backfill deltas.
    fn argued(
        &mut self,
        at: usize,
        item: Option<&Object>,
        delta: Option<&str>,
        whole: Option<&str>,
        out: Events,
    ) -> Result<(), ProviderError> {
        if self.items[at].call.is_none() {
            self.items[at].call = Some(CallState {
                index: self.called,
                arguments_text: String::new(),
            });
            self.called += 1;
        }
        let Some(call) = self.items[at].call.as_mut() else {
            return Ok(());
        };
        let more = match (delta, whole) {
            (Some(delta), _) => delta.to_owned(),
            (None, Some(whole)) => match whole.strip_prefix(call.arguments_text.as_str()) {
                Some(rest) => rest.to_owned(),
                None => {
                    return Err(failed(
                        FailureCategory::Stream,
                        "A tool call's final arguments contradict the ones streamed.",
                        FailureExtra::default(),
                    ));
                }
            },
            (None, None) => String::new(),
        };
        call.arguments_text.push_str(&more);
        out.push(StepEvent::Call(StepEventCall {
            index: call.index as f64,
            wire_id: item.map_or("", |item| text_at(item, "call_id")).to_owned(),
            name: item.map_or("", |item| text_at(item, "name")).to_owned(),
            arguments_text: more,
        }));
        Ok(())
    }

    fn thought(
        &mut self,
        at: usize,
        id: &str,
        label: &str,
        summary: bool,
        index: f64,
        delta: &str,
    ) -> StepEvent {
        let state = self.items[at].thought.get_or_insert_with(Default::default);
        let slot = if summary {
            &mut state.summary
        } else {
            &mut state.content
        };
        let split = slot.is_some_and(|held| held != index);
        *slot = Some(index);
        state.streamed = true;
        let kind = if summary {
            ThoughtPartSegmentsItemKind::Summary
        } else {
            ThoughtPartSegmentsItemKind::Content
        };
        let segment = ThoughtPartSegmentsItem {
            kind,
            text: delta.to_owned(),
        };
        thought_event(bare_thought(Some(vec![segment]), None, id), label, split)
    }

    /// Done events add reasoning.
    fn reasoned(&mut self, at: usize, id: &str, label: &str, item: &Object, out: Events) {
        let opaque = text_at(item, "encrypted_content");
        let streamed = self.items[at]
            .thought
            .as_ref()
            .is_some_and(|state| state.streamed);
        let segments = if streamed {
            Vec::new()
        } else {
            segments_of(item)
        };
        if segments.is_empty() && opaque.is_empty() {
            return;
        }
        out.push(thought_event(
            bare_thought(
                (!segments.is_empty()).then_some(segments),
                (!opaque.is_empty()).then(|| opaque.to_owned()),
                id,
            ),
            label,
            false,
        ));
    }

    /// Status has precedence.
    fn ended(
        &mut self,
        response: Option<&Value>,
        fallback: &str,
        out: Events,
    ) -> Result<(), ProviderError> {
        let empty = Value::Object(Object::new());
        let body = match response {
            Some(held @ Value::Object(_)) => held,
            _ => &empty,
        };
        let Value::Object(body) = body else {
            return Ok(());
        };
        self.served(Some(&Value::Object(body.clone())), out);
        self.usage = usage_of(self.host, body.get("usage")).or(self.usage.take());
        let status = match body.get("status") {
            Some(Value::String(status)) => status.as_str(),
            _ => fallback,
        };
        if status == "failed" || errored(body.get("error")) {
            return Err(self.failure(&error_body(body.get("error"))));
        }
        if status != "completed" && status != "incomplete" {
            return Err(failed(
                FailureCategory::Stream,
                "The provider ended the step.",
                FailureExtra::default(),
            ));
        }
        if let Some(usage) = &self.usage {
            out.push(StepEvent::Usage(StepEventUsage {
                usage: usage.clone(),
            }));
        }
        out.push(StepEvent::Finish(StepEventFinish {
            reason: finish_of(status, body.get("incomplete_details"), self.called > 0),
        }));
        self.finished = true;
        Ok(())
    }
}

struct ResponsesStream<'a> {
    decoder: Decoder<'a>,
    over: bool,
    failed: Option<ProviderError>,
}

impl StreamDecoder for ResponsesStream<'_> {
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
            "The stream ended before the response completed.",
            FailureExtra::default(),
        );
        self.failed = Some(error.clone());
        Err(error)
    }
}

/// OpenAI Responses codec for one host.
pub struct ResponsesCodec<H> {
    host: H,
    facts: Option<CodecFacts>,
}

pub fn responses_codec<H: Host>(host: H, facts: Option<&CodecFacts>) -> ResponsesCodec<H> {
    ResponsesCodec {
        host,
        facts: facts.cloned(),
    }
}

impl<H: Host> Codec for ResponsesCodec<H> {
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
        Box::new(ResponsesStream {
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
        let mut decoder = Decoder::new(&self.host);
        let mut events = Vec::new();
        decoder.whole(body, &mut events)?;
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
