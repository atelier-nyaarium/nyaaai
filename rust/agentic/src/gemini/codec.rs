//! Google generateContent codec.

use nyaaai_contract::json::{Object, Record, Value, number_to_string, stringify_json};
use nyaaai_contract::shapes::step::{
    CanonicalRequest, CodecFacts, Effort, Encoded, Failure, FailureCategory, FinishReason,
    HeaderList, NormalizedUsage, Serving, SseItem, StepEvent, StepEventCall, StepEventFinish,
    StepEventText, StepEventThought, StepEventUsage, StepResult, ThoughtPart,
    ThoughtPartSegmentsItem, ThoughtPartSegmentsItemKind, ToolSpec, Turn, TurnPart, TurnStep,
};
use nyaaai_contract::text::encode_path_segment;

use crate::codec::{Codec, Mode, StreamDecoder};
use crate::failure::{FailureExtra, NumberOrText, ProviderError, WireError, failed};
use crate::loose::{
    amount_at, error_body, errored, items_at, number_at, object_at, object_of, text_at, text_or,
};
use crate::native::{
    Host, HostStatus, arguments_of, assembled, current_turn_from, parsed_chunk, schema_prompt,
    serving_events, stream_failure, system_texts, wire_error,
};
use crate::schema_projection::strict_schema;

const THOUGHT: &str = "thought";

/// Text signature anchor.
const ON_TEXT: &str = "text";

/// Sentinel bypasses signature validation.
pub const SKIP_SIGNATURE: &str = "skip_thought_signature_validator";

const FILTERED: [&str; 9] = [
    "SAFETY",
    "RECITATION",
    "LANGUAGE",
    "BLOCKLIST",
    "PROHIBITED_CONTENT",
    "SPII",
    "IMAGE_SAFETY",
    "IMAGE_PROHIBITED_CONTENT",
    "IMAGE_RECITATION",
];

fn level_of(effort: Effort) -> Option<&'static str> {
    match effort {
        Effort::Minimal => Some("MINIMAL"),
        Effort::Low => Some("LOW"),
        Effort::Medium => Some("MEDIUM"),
        Effort::High => Some("HIGH"),
        _ => None,
    }
}

fn on_call(index: f64) -> String {
    format!("call:{}", number_to_string(index))
}

/// Replay only to producer.
fn readable(step: &TurnStep, request: &CanonicalRequest) -> bool {
    step.producer
        .as_ref()
        .is_some_and(|producer| producer.model == request.model)
        && step.parts.iter().all(|part| match part {
            TurnPart::Thought(thought) => thought.type_ == THOUGHT,
            _ => true,
        })
}

fn thought_part(part: &ThoughtPart) -> Vec<Value> {
    if part.anchor.as_deref() == Some(ON_TEXT) {
        return match &part.opaque {
            None => Vec::new(),
            Some(opaque) => vec![object_of(vec![
                ("text", Value::from("")),
                ("thoughtSignature", Value::from(opaque.as_str())),
            ])],
        };
    }
    if part.anchor.is_some() {
        return Vec::new();
    }
    let text: String = part
        .segments
        .iter()
        .flatten()
        .map(|segment| segment.text.as_str())
        .collect();
    let mut entries = vec![
        ("text", Value::String(text)),
        ("thought", Value::Bool(true)),
    ];
    if let Some(opaque) = &part.opaque {
        entries.push(("thoughtSignature", Value::from(opaque.as_str())));
    }
    vec![object_of(entries)]
}

fn content(role: &str, parts: Vec<Value>) -> Value {
    object_of(vec![
        ("role", Value::from(role)),
        ("parts", Value::Array(parts)),
    ])
}

fn text_part(text: &str) -> Value {
    object_of(vec![("text", Value::from(text))])
}

/// First unsigned current call gets sentinel.
fn step_contents(step: &TurnStep, request: &CanonicalRequest, current: bool) -> Vec<Value> {
    let sent = readable(step, request);
    let signature_of = |anchor: &str| -> Option<&str> {
        if !sent {
            return None;
        }
        step.parts.iter().rev().find_map(|part| match part {
            TurnPart::Thought(thought) if thought.anchor.as_deref() == Some(anchor) => {
                thought.opaque.as_deref()
            }
            _ => None,
        })
    };
    let mut names: Vec<(f64, &str)> = Vec::new();
    for part in &step.parts {
        if let TurnPart::Call(call) = part {
            match names.iter_mut().find(|(index, _)| *index == call.index) {
                Some(held) => held.1 = &call.name,
                None => names.push((call.index, &call.name)),
            }
        }
    }
    let mut parts: Vec<Value> = Vec::new();
    for part in &step.parts {
        match part {
            TurnPart::Thought(thought) if sent => parts.extend(thought_part(thought)),
            TurnPart::Text(text) if !text.text.is_empty() => parts.push(text_part(&text.text)),
            TurnPart::Call(call) => {
                let mut entries = vec![(
                    "functionCall",
                    object_of(vec![
                        ("name", Value::from(call.name.as_str())),
                        ("args", Value::Object(arguments_of(&call.arguments))),
                    ]),
                )];
                if let Some(signature) = signature_of(&on_call(call.index)) {
                    entries.push(("thoughtSignature", Value::from(signature)));
                }
                parts.push(object_of(entries));
            }
            _ => {}
        }
    }
    let first = parts
        .iter()
        .position(|part| part.get("functionCall").is_some());
    if let Some(first) = first
        && current
        && let Value::Object(call) = &mut parts[first]
        && call.get("thoughtSignature").is_none()
    {
        call.insert("thoughtSignature", Value::from(SKIP_SIGNATURE));
    }
    let mut responses = Vec::new();
    for part in &step.parts {
        let TurnPart::Result(result) = part else {
            continue;
        };
        let Some((_, name)) = names.iter().find(|(index, _)| *index == result.index) else {
            continue;
        };
        let key = if result.is_error { "error" } else { "output" };
        responses.push(object_of(vec![(
            "functionResponse",
            object_of(vec![
                ("name", Value::from(*name)),
                (
                    "response",
                    object_of(vec![(key, Value::from(result.output.as_str()))]),
                ),
            ]),
        )]));
    }
    let mut out = Vec::new();
    if !parts.is_empty() {
        out.push(content("model", parts));
    }
    if !responses.is_empty() {
        out.push(content("user", responses));
    }
    out
}

fn contents_of(request: &CanonicalRequest) -> Vec<Value> {
    let current = current_turn_from(&request.history);
    let mut out = Vec::new();
    for (at, turn) in request.history.iter().enumerate() {
        match turn {
            Turn::User(user) => out.push(content("user", vec![text_part(&user.text)])),
            Turn::Step(step) => out.extend(step_contents(step, request, at >= current)),
        }
    }
    if let Some(state) = &request.state {
        out.push(content("user", vec![text_part(state)]));
    }
    out
}

fn declaration_of(tool: &ToolSpec) -> Value {
    object_of(vec![
        ("name", Value::from(tool.name.as_str())),
        ("description", Value::from(tool.description.as_str())),
        (
            "parametersJsonSchema",
            Value::Object(tool.parameters.clone()),
        ),
    ])
}

/// Named or any-tool choice.
fn choice_of(choice: &[String], name: &str) -> Option<Value> {
    let config = |entries: Vec<(&str, Value)>| {
        object_of(vec![("functionCallingConfig", object_of(entries))])
    };
    if choice.iter().any(|held| held == "tool") {
        return Some(config(vec![
            ("mode", Value::from("ANY")),
            (
                "allowedFunctionNames",
                Value::Array(vec![Value::from(name)]),
            ),
        ]));
    }
    choice
        .iter()
        .any(|held| held == "required")
        .then(|| config(vec![("mode", Value::from("ANY"))]))
}

/// Catalog gates optional fields.
fn body_of(facts: Option<&CodecFacts>, request: &CanonicalRequest) -> Result<Value, String> {
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
    let system: Vec<Value> = system_texts(request, prompted.as_deref())
        .iter()
        .filter(|text| !text.is_empty())
        .map(|text| text_part(text))
        .collect();
    let forced = request
        .tool_choice
        .as_ref()
        .and_then(|choice| choice_of(api.map_or(&[][..], |api| &api.choice), &choice.name));
    let level = request
        .effort
        .filter(|effort| efforts.contains(effort))
        .and_then(level_of);
    let mut generation = Object::new();
    if let Some(enforced) = enforced {
        generation.insert("responseMimeType", Value::from("application/json"));
        generation.insert("responseJsonSchema", Value::Object(enforced));
    }
    if !efforts.is_empty() {
        let mut entries = vec![("includeThoughts", Value::Bool(true))];
        if let Some(level) = level {
            entries.push(("thinkingLevel", Value::from(level)));
        }
        generation.insert("thinkingConfig", object_of(entries));
    }
    if let Some(max) = request.max_tokens {
        generation.insert("maxOutputTokens", Value::Number(max as f64));
    }
    let mut body = Object::new();
    body.insert("contents", Value::Array(contents_of(request)));
    if !system.is_empty() {
        body.insert(
            "systemInstruction",
            object_of(vec![("parts", Value::Array(system))]),
        );
    }
    if let Some(tools) = request.tools.as_ref().filter(|tools| !tools.is_empty()) {
        body.insert(
            "tools",
            Value::Array(vec![object_of(vec![(
                "functionDeclarations",
                Value::Array(tools.iter().map(declaration_of).collect()),
            )])]),
        );
    }
    if let Some(forced) = forced {
        body.insert("toolConfig", forced);
    }
    if !generation.is_empty() {
        body.insert("generationConfig", Value::Object(generation));
    }
    Ok(Value::Object(body))
}

/// Output includes thoughts.
fn usage_of(host: &dyn Host, raw: Option<&Value>) -> Option<NormalizedUsage> {
    let raw = raw?.as_object()?;
    let reasoning = amount_at(raw, "thoughtsTokenCount");
    Some(host.usage(
        NormalizedUsage {
            input: amount_at(raw, "promptTokenCount").unwrap_or(0.0)
                + amount_at(raw, "toolUsePromptTokenCount").unwrap_or(0.0),
            output: amount_at(raw, "candidatesTokenCount").unwrap_or(0.0)
                + reasoning.unwrap_or(0.0),
            cached: amount_at(raw, "cachedContentTokenCount"),
            cache_write: None,
            reasoning,
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

fn finish_of(reason: &str, called: bool) -> FinishReason {
    if reason == "STOP" {
        return if called {
            FinishReason::ToolCalls
        } else {
            FinishReason::Stop
        };
    }
    if reason == "MAX_TOKENS" || reason == "CONTINUATION" {
        return FinishReason::Length;
    }
    if FILTERED.contains(&reason) {
        FinishReason::ContentFilter
    } else {
        FinishReason::Error
    }
}

fn thought_of(
    segments: Option<Vec<ThoughtPartSegmentsItem>>,
    opaque: Option<String>,
    anchor: Option<String>,
) -> StepEvent {
    StepEvent::Thought(StepEventThought {
        part: ThoughtPart {
            type_: THOUGHT.to_owned(),
            segments,
            opaque,
            id: None,
            format: None,
            index: None,
            anchor,
        },
        block: None,
        split: None,
    })
}

type Events<'e> = &'e mut Vec<StepEvent>;

/// Calls arrive whole.
struct Decoder<'a> {
    host: &'a dyn Host,
    finished: bool,
    calls: usize,
    serving: Serving,
    usage: Option<NormalizedUsage>,
}

impl<'a> Decoder<'a> {
    fn new(host: &'a dyn Host) -> Self {
        Self {
            host,
            finished: false,
            calls: 0,
            serving: Serving {
                host: None,
                generation_id: None,
            },
            usage: None,
        }
    }

    fn chunk(&mut self, data: &Object, out: Events) -> Result<(), ProviderError> {
        self.usage = usage_of(self.host, data.get("usageMetadata")).or(self.usage.take());
        let usage = match data.get("usageMetadata") {
            Some(Value::Object(_)) => self.usage.clone(),
            _ => None,
        };
        let push_usage = |usage: Option<NormalizedUsage>, out: &mut Vec<StepEvent>| {
            if let Some(usage) = usage {
                out.push(StepEvent::Usage(StepEventUsage { usage }));
            }
        };
        if self.finished {
            push_usage(usage, out);
            return Ok(());
        }
        if self.serving.generation_id.is_none() {
            let id = data.get("responseId").cloned().unwrap_or(Value::Null);
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
        if errored(data.get("error")) {
            let error = error_body(data.get("error"));
            return Err(stream_failure(
                self.host,
                &WireError {
                    message: Some(text_or(text_at(&error, "message"), "The response failed.")),
                    code: number_at(&error, "code").map(NumberOrText::Number),
                    type_: error
                        .get("status")
                        .and_then(Value::as_str)
                        .map(str::to_owned),
                    ..WireError::default()
                },
                self.usage.clone(),
                &self.serving,
            ));
        }
        let candidate = match items_at(data, "candidates").first() {
            Some(Value::Object(candidate)) => Some(candidate),
            _ => None,
        };
        let feedback = object_at(data, "promptFeedback");
        if candidate.is_none()
            && let Some(Value::String(reason)) = feedback.and_then(|held| held.get("blockReason"))
        {
            return Err(failed(
                FailureCategory::Refused,
                &format!("The prompt was blocked: {reason}."),
                FailureExtra {
                    status: Some(nyaaai_contract::shapes::step::FailureStatus::Text(
                        "stream".to_owned(),
                    )),
                    billed: (self.usage.is_some()
                        || self.serving.generation_id.is_some()
                        || self.serving.host.is_some())
                    .then(|| nyaaai_contract::shapes::step::FailureBilled {
                        usage: self.usage.clone(),
                        host: self.serving.host.clone(),
                        generation_id: self.serving.generation_id.clone(),
                    }),
                    ..FailureExtra::default()
                },
            ));
        }
        let held = candidate.and_then(|candidate| object_at(candidate, "content"));
        for part in held.map_or(&[][..], |held| items_at(held, "parts")) {
            if let Value::Object(part) = part {
                self.part(part, out);
            }
        }
        push_usage(usage, out);
        let reason = candidate.map_or("", |candidate| text_at(candidate, "finishReason"));
        if reason.is_empty() {
            return Ok(());
        }
        if reason == "MISSING_THOUGHT_SIGNATURE" {
            return Err(failed(
                FailureCategory::ReplayRejected,
                "A thought signature was missing.",
                FailureExtra {
                    status: Some(nyaaai_contract::shapes::step::FailureStatus::Text(
                        "stream".to_owned(),
                    )),
                    ..FailureExtra::default()
                },
            ));
        }
        self.finished = true;
        out.push(StepEvent::Finish(StepEventFinish {
            reason: finish_of(reason, self.calls > 0),
        }));
        Ok(())
    }

    /// Call/text signature anchors.
    fn part(&mut self, part: &Object, out: Events) {
        let signature = text_at(part, "thoughtSignature");
        if let Some(Value::Object(call)) = part.get("functionCall") {
            let index = self.calls as f64;
            self.calls += 1;
            if !signature.is_empty() {
                out.push(thought_of(
                    None,
                    Some(signature.to_owned()),
                    Some(on_call(index)),
                ));
            }
            let args = match call.get("args") {
                Some(Value::Object(args)) => Value::Object(args.clone()),
                _ => Value::Object(Object::new()),
            };
            out.push(StepEvent::Call(StepEventCall {
                index,
                wire_id: text_at(call, "id").to_owned(),
                name: text_at(call, "name").to_owned(),
                arguments_text: stringify_json(&args),
            }));
            return;
        }
        let Some(Value::String(text)) = part.get("text") else {
            return;
        };
        if part.get("thought") == Some(&Value::Bool(true)) {
            if !text.is_empty() || !signature.is_empty() {
                out.push(thought_of(
                    (!text.is_empty()).then(|| {
                        vec![ThoughtPartSegmentsItem {
                            kind: ThoughtPartSegmentsItemKind::Content,
                            text: text.clone(),
                        }]
                    }),
                    (!signature.is_empty()).then(|| signature.to_owned()),
                    None,
                ));
            }
            return;
        }
        if !text.is_empty() {
            out.push(StepEvent::Text(StepEventText {
                text: text.clone(),
                phase: None,
            }));
        }
        if !signature.is_empty() {
            out.push(thought_of(
                None,
                Some(signature.to_owned()),
                Some(ON_TEXT.to_owned()),
            ));
        }
    }
}

struct GeminiStream<'a> {
    decoder: Decoder<'a>,
    over: bool,
    failed: Option<ProviderError>,
}

impl StreamDecoder for GeminiStream<'_> {
    fn over(&self) -> bool {
        self.over
    }

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
            SseItem::Data(data) => {
                parsed_chunk(&data.data).and_then(|chunk| self.decoder.chunk(&chunk, events))
            }
        };
        if let Err(error) = &result {
            self.failed = Some(error.clone());
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
            "The stream ended before a finish reason.",
            FailureExtra::default(),
        );
        self.failed = Some(error.clone());
        Err(error)
    }
}

/// Google generateContent codec for one host.
pub struct GeminiCodec<H> {
    host: H,
    facts: Option<CodecFacts>,
}

pub fn gemini_codec<H: Host>(host: H, facts: Option<&CodecFacts>) -> GeminiCodec<H> {
    GeminiCodec {
        host,
        facts: facts.cloned(),
    }
}

impl<H: Host> Codec for GeminiCodec<H> {
    fn encode(&self, request: &CanonicalRequest, mode: Mode) -> Result<Encoded, String> {
        let model = encode_path_segment(&request.model);
        let method = if mode == Mode::Stream {
            "streamGenerateContent?alt=sse"
        } else {
            "generateContent"
        };
        let mut headers = Record::new();
        for (key, value) in self.host.headers(request) {
            headers.insert(key, value);
        }
        Ok(Encoded {
            path: format!("{}/models/{model}:{method}", self.host.prefix()),
            body: body_of(self.facts.as_ref(), request)?,
            headers: Some(headers),
        })
    }

    fn stream(&self) -> Box<dyn StreamDecoder + '_> {
        Box::new(GeminiStream {
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
        decoder.chunk(body, &mut events)?;
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
