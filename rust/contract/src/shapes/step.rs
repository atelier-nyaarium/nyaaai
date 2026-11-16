//! Generated from schema/step.output.json by scripts/contract.ts. Do not edit.

#![allow(
    clippy::enum_variant_names,
    clippy::large_enum_variant,
    clippy::ptr_arg
)]

use crate::json::{Object, Record, Value};
use crate::shape::{self, Path, Refused, Shape};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Effort {
    None,
    Minimal,
    Low,
    Medium,
    High,
    Xhigh,
    Max,
}

impl Effort {
    pub const ALL: [Self; 7] = [
        Self::None,
        Self::Minimal,
        Self::Low,
        Self::Medium,
        Self::High,
        Self::Xhigh,
        Self::Max,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Minimal => "minimal",
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Xhigh => "xhigh",
            Self::Max => "max",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for Effort {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ToolSpec {
    pub name: String,
    pub description: String,
    pub parameters: Record<Value>,
}

impl Shape for ToolSpec {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let name = shape::required(object, "name", at, shape::text)?;
        let description = shape::required(object, "description", at, shape::text)?;
        let parameters = shape::required(object, "parameters", at, |value, at| {
            shape::record(value, at, shape::any)
        })?;
        Ok(Self {
            name,
            description,
            parameters,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("name", Value::from(self.name.as_str()));
        object.insert("description", Value::from(self.description.as_str()));
        object.insert(
            "parameters",
            shape::record_value(&self.parameters, |held| held.clone()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextPart {
    pub text: String,
    pub phase: Option<String>,
}

impl Shape for TextPart {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "text")
        })?;
        let text = shape::required(object, "text", at, shape::text)?;
        let phase = shape::optional(object, "phase", at, shape::text)?;
        Ok(Self { text, phase })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("text"));
        object.insert("text", Value::from(self.text.as_str()));
        if let Some(held) = &self.phase {
            object.insert("phase", Value::from(held.as_str()));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ThoughtPart {
    pub type_: String,
    pub segments: Option<Vec<ThoughtPartSegmentsItem>>,
    pub opaque: Option<String>,
    pub id: Option<String>,
    pub format: Option<String>,
    pub index: Option<f64>,
    pub anchor: Option<String>,
}

impl Shape for ThoughtPart {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "thought")
        })?;
        let type_ = shape::required(object, "type", at, shape::text)?;
        let segments = shape::optional(object, "segments", at, |value, at| {
            shape::list(value, at, ThoughtPartSegmentsItem::decode_at)
        })?;
        let opaque = shape::optional(object, "opaque", at, shape::text)?;
        let id = shape::optional(object, "id", at, shape::text)?;
        let format = shape::optional(object, "format", at, shape::text)?;
        let index = shape::optional(object, "index", at, shape::number)?;
        let anchor = shape::optional(object, "anchor", at, shape::text)?;
        Ok(Self {
            type_,
            segments,
            opaque,
            id,
            format,
            index,
            anchor,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("thought"));
        object.insert("type", Value::from(self.type_.as_str()));
        if let Some(held) = &self.segments {
            object.insert(
                "segments",
                Value::Array(held.iter().map(|held| held.encode()).collect()),
            );
        }
        if let Some(held) = &self.opaque {
            object.insert("opaque", Value::from(held.as_str()));
        }
        if let Some(held) = &self.id {
            object.insert("id", Value::from(held.as_str()));
        }
        if let Some(held) = &self.format {
            object.insert("format", Value::from(held.as_str()));
        }
        if let Some(held) = &self.index {
            object.insert("index", Value::Number(*held));
        }
        if let Some(held) = &self.anchor {
            object.insert("anchor", Value::from(held.as_str()));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ThoughtPartSegmentsItem {
    pub kind: ThoughtPartSegmentsItemKind,
    pub text: String,
}

impl Shape for ThoughtPartSegmentsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let kind = shape::required(object, "kind", at, ThoughtPartSegmentsItemKind::decode_at)?;
        let text = shape::required(object, "text", at, shape::text)?;
        Ok(Self { kind, text })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", self.kind.encode());
        object.insert("text", Value::from(self.text.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ThoughtPartSegmentsItemKind {
    Summary,
    Content,
}

impl ThoughtPartSegmentsItemKind {
    pub const ALL: [Self; 2] = [Self::Summary, Self::Content];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Summary => "summary",
            Self::Content => "content",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for ThoughtPartSegmentsItemKind {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TurnPart {
    Text(TextPart),
    Thought(ThoughtPart),
    Call(TurnPartCall),
    Result(TurnPartResult),
}

impl Shape for TurnPart {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        match shape::tag(object, "kind", at)? {
            "text" => TextPart::decode_at(value, at).map(Self::Text),
            "thought" => ThoughtPart::decode_at(value, at).map(Self::Thought),
            "call" => TurnPartCall::decode_at(value, at).map(Self::Call),
            "result" => TurnPartResult::decode_at(value, at).map(Self::Result),
            _ => Err(shape::no_branch(at, "kind")),
        }
    }

    fn encode(&self) -> Value {
        match self {
            Self::Text(held) => held.encode(),
            Self::Thought(held) => held.encode(),
            Self::Call(held) => held.encode(),
            Self::Result(held) => held.encode(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TurnPartCall {
    pub index: f64,
    pub wire_id: String,
    pub name: String,
    pub arguments: String,
}

impl Shape for TurnPartCall {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "call")
        })?;
        let index = shape::required(object, "index", at, shape::number)?;
        let wire_id = shape::required(object, "wireId", at, shape::text)?;
        let name = shape::required(object, "name", at, shape::text)?;
        let arguments = shape::required(object, "arguments", at, shape::text)?;
        Ok(Self {
            index,
            wire_id,
            name,
            arguments,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("call"));
        object.insert("index", Value::Number(self.index));
        object.insert("wireId", Value::from(self.wire_id.as_str()));
        object.insert("name", Value::from(self.name.as_str()));
        object.insert("arguments", Value::from(self.arguments.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TurnPartResult {
    pub index: f64,
    pub output: String,
    pub is_error: bool,
}

impl Shape for TurnPartResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "result")
        })?;
        let index = shape::required(object, "index", at, shape::number)?;
        let output = shape::required(object, "output", at, shape::text)?;
        let is_error = shape::required(object, "isError", at, shape::flag)?;
        Ok(Self {
            index,
            output,
            is_error,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("result"));
        object.insert("index", Value::Number(self.index));
        object.insert("output", Value::from(self.output.as_str()));
        object.insert("isError", Value::Bool(self.is_error));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Turn {
    User(TurnUser),
    Step(TurnStep),
}

impl Shape for Turn {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        match shape::tag(object, "role", at)? {
            "user" => TurnUser::decode_at(value, at).map(Self::User),
            "step" => TurnStep::decode_at(value, at).map(Self::Step),
            _ => Err(shape::no_branch(at, "role")),
        }
    }

    fn encode(&self) -> Value {
        match self {
            Self::User(held) => held.encode(),
            Self::Step(held) => held.encode(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TurnUser {
    pub text: String,
}

impl Shape for TurnUser {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "role", at, |value, at| {
            shape::is_text(value, at, "user")
        })?;
        let text = shape::required(object, "text", at, shape::text)?;
        Ok(Self { text })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("role", Value::from("user"));
        object.insert("text", Value::from(self.text.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TurnStep {
    pub producer: Option<TurnStepProducer>,
    pub parts: Vec<TurnPart>,
}

impl Shape for TurnStep {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "role", at, |value, at| {
            shape::is_text(value, at, "step")
        })?;
        let producer = shape::optional(object, "producer", at, TurnStepProducer::decode_at)?;
        let parts = shape::required(object, "parts", at, |value, at| {
            shape::list(value, at, TurnPart::decode_at)
        })?;
        Ok(Self { producer, parts })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("role", Value::from("step"));
        if let Some(held) = &self.producer {
            object.insert("producer", held.encode());
        }
        object.insert(
            "parts",
            Value::Array(self.parts.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TurnStepProducer {
    pub model: String,
    pub host: Option<String>,
}

impl Shape for TurnStepProducer {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let model = shape::required(object, "model", at, shape::text)?;
        let host = shape::optional(object, "host", at, shape::text)?;
        Ok(Self { model, host })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("model", Value::from(self.model.as_str()));
        if let Some(held) = &self.host {
            object.insert("host", Value::from(held.as_str()));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanonicalRequest {
    pub model: String,
    pub system: Vec<String>,
    pub summary: Option<String>,
    pub history: Vec<Turn>,
    pub state: Option<String>,
    pub tools: Option<Vec<ToolSpec>>,
    pub tool_choice: Option<CanonicalRequestToolChoice>,
    pub schema: Option<CanonicalRequestSchema>,
    pub effort: Option<Effort>,
    pub cache: CanonicalRequestCache,
    pub max_tokens: Option<i64>,
    pub session: String,
    pub zdr_only: bool,
}

impl Shape for CanonicalRequest {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let model = shape::required(object, "model", at, shape::text)?;
        let system = shape::required(object, "system", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let summary = shape::optional(object, "summary", at, shape::text)?;
        let history = shape::required(object, "history", at, |value, at| {
            shape::list(value, at, Turn::decode_at)
        })?;
        let state = shape::optional(object, "state", at, shape::text)?;
        let tools = shape::optional(object, "tools", at, |value, at| {
            shape::list(value, at, ToolSpec::decode_at)
        })?;
        let tool_choice = shape::optional(
            object,
            "toolChoice",
            at,
            CanonicalRequestToolChoice::decode_at,
        )?;
        let schema = shape::optional(object, "schema", at, CanonicalRequestSchema::decode_at)?;
        let effort = shape::optional(object, "effort", at, Effort::decode_at)?;
        let cache = shape::required(object, "cache", at, CanonicalRequestCache::decode_at)?;
        let max_tokens = shape::optional(object, "maxTokens", at, shape::integer)?;
        let session = shape::required(object, "session", at, shape::text)?;
        let zdr_only = shape::required(object, "zdrOnly", at, shape::flag)?;
        Ok(Self {
            model,
            system,
            summary,
            history,
            state,
            tools,
            tool_choice,
            schema,
            effort,
            cache,
            max_tokens,
            session,
            zdr_only,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("model", Value::from(self.model.as_str()));
        object.insert(
            "system",
            Value::Array(
                self.system
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        if let Some(held) = &self.summary {
            object.insert("summary", Value::from(held.as_str()));
        }
        object.insert(
            "history",
            Value::Array(self.history.iter().map(|held| held.encode()).collect()),
        );
        if let Some(held) = &self.state {
            object.insert("state", Value::from(held.as_str()));
        }
        if let Some(held) = &self.tools {
            object.insert(
                "tools",
                Value::Array(held.iter().map(|held| held.encode()).collect()),
            );
        }
        if let Some(held) = &self.tool_choice {
            object.insert("toolChoice", held.encode());
        }
        if let Some(held) = &self.schema {
            object.insert("schema", held.encode());
        }
        if let Some(held) = &self.effort {
            object.insert("effort", held.encode());
        }
        object.insert("cache", self.cache.encode());
        if let Some(held) = &self.max_tokens {
            object.insert("maxTokens", Value::Number(*held as f64));
        }
        object.insert("session", Value::from(self.session.as_str()));
        object.insert("zdrOnly", Value::Bool(self.zdr_only));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanonicalRequestToolChoice {
    pub name: String,
}

impl Shape for CanonicalRequestToolChoice {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let name = shape::required(object, "name", at, shape::text)?;
        Ok(Self { name })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("name", Value::from(self.name.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanonicalRequestSchema {
    pub name: String,
    pub json: Record<Value>,
}

impl Shape for CanonicalRequestSchema {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let name = shape::required(object, "name", at, shape::text)?;
        let json = shape::required(object, "json", at, |value, at| {
            shape::record(value, at, shape::any)
        })?;
        Ok(Self { name, json })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("name", Value::from(self.name.as_str()));
        object.insert("json", shape::record_value(&self.json, |held| held.clone()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanonicalRequestCache {
    pub kind: CanonicalRequestCacheKind,
    pub ttl: Option<()>,
}

impl Shape for CanonicalRequestCache {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let kind = shape::required(object, "kind", at, CanonicalRequestCacheKind::decode_at)?;
        let ttl = shape::optional(object, "ttl", at, |value, at| {
            shape::is_text(value, at, "1h")
        })?;
        Ok(Self { kind, ttl })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", self.kind.encode());
        if self.ttl.is_some() {
            object.insert("ttl", Value::from("1h"));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CanonicalRequestCacheKind {
    None,
    Implicit,
    Explicit,
}

impl CanonicalRequestCacheKind {
    pub const ALL: [Self; 3] = [Self::None, Self::Implicit, Self::Explicit];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Implicit => "implicit",
            Self::Explicit => "explicit",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for CanonicalRequestCacheKind {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ThoughtDelta {
    pub part: ThoughtPart,
    pub block: Option<String>,
    pub split: Option<bool>,
}

impl Shape for ThoughtDelta {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let part = shape::required(object, "part", at, ThoughtPart::decode_at)?;
        let block = shape::optional(object, "block", at, shape::text)?;
        let split = shape::optional(object, "split", at, shape::flag)?;
        Ok(Self { part, block, split })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("part", self.part.encode());
        if let Some(held) = &self.block {
            object.insert("block", Value::from(held.as_str()));
        }
        if let Some(held) = &self.split {
            object.insert("split", Value::Bool(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum StepEvent {
    Text(StepEventText),
    Thought(StepEventThought),
    Call(StepEventCall),
    Serving(StepEventServing),
    Usage(StepEventUsage),
    Finish(StepEventFinish),
}

impl Shape for StepEvent {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        match shape::tag(object, "kind", at)? {
            "text" => StepEventText::decode_at(value, at).map(Self::Text),
            "thought" => StepEventThought::decode_at(value, at).map(Self::Thought),
            "call" => StepEventCall::decode_at(value, at).map(Self::Call),
            "serving" => StepEventServing::decode_at(value, at).map(Self::Serving),
            "usage" => StepEventUsage::decode_at(value, at).map(Self::Usage),
            "finish" => StepEventFinish::decode_at(value, at).map(Self::Finish),
            _ => Err(shape::no_branch(at, "kind")),
        }
    }

    fn encode(&self) -> Value {
        match self {
            Self::Text(held) => held.encode(),
            Self::Thought(held) => held.encode(),
            Self::Call(held) => held.encode(),
            Self::Serving(held) => held.encode(),
            Self::Usage(held) => held.encode(),
            Self::Finish(held) => held.encode(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StepEventText {
    pub text: String,
    pub phase: Option<String>,
}

impl Shape for StepEventText {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "text")
        })?;
        let text = shape::required(object, "text", at, shape::text)?;
        let phase = shape::optional(object, "phase", at, shape::text)?;
        Ok(Self { text, phase })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("text"));
        object.insert("text", Value::from(self.text.as_str()));
        if let Some(held) = &self.phase {
            object.insert("phase", Value::from(held.as_str()));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StepEventThought {
    pub part: ThoughtPart,
    pub block: Option<String>,
    pub split: Option<bool>,
}

impl Shape for StepEventThought {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "thought")
        })?;
        let part = shape::required(object, "part", at, ThoughtPart::decode_at)?;
        let block = shape::optional(object, "block", at, shape::text)?;
        let split = shape::optional(object, "split", at, shape::flag)?;
        Ok(Self { part, block, split })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("thought"));
        object.insert("part", self.part.encode());
        if let Some(held) = &self.block {
            object.insert("block", Value::from(held.as_str()));
        }
        if let Some(held) = &self.split {
            object.insert("split", Value::Bool(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StepEventCall {
    pub index: f64,
    pub wire_id: String,
    pub name: String,
    pub arguments_text: String,
}

impl Shape for StepEventCall {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "call")
        })?;
        let index = shape::required(object, "index", at, shape::number)?;
        let wire_id = shape::required(object, "wireId", at, shape::text)?;
        let name = shape::required(object, "name", at, shape::text)?;
        let arguments_text = shape::required(object, "argumentsText", at, shape::text)?;
        Ok(Self {
            index,
            wire_id,
            name,
            arguments_text,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("call"));
        object.insert("index", Value::Number(self.index));
        object.insert("wireId", Value::from(self.wire_id.as_str()));
        object.insert("name", Value::from(self.name.as_str()));
        object.insert("argumentsText", Value::from(self.arguments_text.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StepEventServing {
    pub serving: Serving,
}

impl Shape for StepEventServing {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "serving")
        })?;
        let serving = shape::required(object, "serving", at, Serving::decode_at)?;
        Ok(Self { serving })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("serving"));
        object.insert("serving", self.serving.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StepEventUsage {
    pub usage: NormalizedUsage,
}

impl Shape for StepEventUsage {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "usage")
        })?;
        let usage = shape::required(object, "usage", at, NormalizedUsage::decode_at)?;
        Ok(Self { usage })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("usage"));
        object.insert("usage", self.usage.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StepEventFinish {
    pub reason: FinishReason,
}

impl Shape for StepEventFinish {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "finish")
        })?;
        let reason = shape::required(object, "reason", at, FinishReason::decode_at)?;
        Ok(Self { reason })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("finish"));
        object.insert("reason", self.reason.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReplyPart {
    Text(TextPart),
    Thought(ThoughtPart),
    Call(ReplyPartCall),
}

impl Shape for ReplyPart {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        match shape::tag(object, "kind", at)? {
            "text" => TextPart::decode_at(value, at).map(Self::Text),
            "thought" => ThoughtPart::decode_at(value, at).map(Self::Thought),
            "call" => ReplyPartCall::decode_at(value, at).map(Self::Call),
            _ => Err(shape::no_branch(at, "kind")),
        }
    }

    fn encode(&self) -> Value {
        match self {
            Self::Text(held) => held.encode(),
            Self::Thought(held) => held.encode(),
            Self::Call(held) => held.encode(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReplyPartCall {
    pub index: f64,
    pub wire_id: String,
    pub name: String,
    pub arguments_text: String,
}

impl Shape for ReplyPartCall {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "call")
        })?;
        let index = shape::required(object, "index", at, shape::number)?;
        let wire_id = shape::required(object, "wireId", at, shape::text)?;
        let name = shape::required(object, "name", at, shape::text)?;
        let arguments_text = shape::required(object, "argumentsText", at, shape::text)?;
        Ok(Self {
            index,
            wire_id,
            name,
            arguments_text,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("call"));
        object.insert("index", Value::Number(self.index));
        object.insert("wireId", Value::from(self.wire_id.as_str()));
        object.insert("name", Value::from(self.name.as_str()));
        object.insert("argumentsText", Value::from(self.arguments_text.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Serving {
    pub host: Option<String>,
    pub generation_id: Option<String>,
}

impl Shape for Serving {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let host = shape::optional(object, "host", at, shape::text)?;
        let generation_id = shape::optional(object, "generationId", at, shape::text)?;
        Ok(Self {
            host,
            generation_id,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        if let Some(held) = &self.host {
            object.insert("host", Value::from(held.as_str()));
        }
        if let Some(held) = &self.generation_id {
            object.insert("generationId", Value::from(held.as_str()));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FinishReason {
    Stop,
    ToolCalls,
    Length,
    ContentFilter,
    Error,
    Empty,
}

impl FinishReason {
    pub const ALL: [Self; 6] = [
        Self::Stop,
        Self::ToolCalls,
        Self::Length,
        Self::ContentFilter,
        Self::Error,
        Self::Empty,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Stop => "stop",
            Self::ToolCalls => "tool_calls",
            Self::Length => "length",
            Self::ContentFilter => "content_filter",
            Self::Error => "error",
            Self::Empty => "",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for FinishReason {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NormalizedUsage {
    pub input: f64,
    pub output: f64,
    pub cached: Option<f64>,
    pub cache_write: Option<f64>,
    pub reasoning: Option<f64>,
    pub cost: Option<f64>,
    pub upstream_cost: Option<f64>,
    pub byok: Option<bool>,
    pub billing_source: Option<String>,
    pub list_cost: Option<f64>,
    pub allowance: Option<f64>,
}

impl Shape for NormalizedUsage {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let input = shape::required(object, "input", at, shape::number)?;
        let output = shape::required(object, "output", at, shape::number)?;
        let cached = shape::optional(object, "cached", at, shape::number)?;
        let cache_write = shape::optional(object, "cacheWrite", at, shape::number)?;
        let reasoning = shape::optional(object, "reasoning", at, shape::number)?;
        let cost = shape::optional(object, "cost", at, shape::number)?;
        let upstream_cost = shape::optional(object, "upstreamCost", at, shape::number)?;
        let byok = shape::optional(object, "byok", at, shape::flag)?;
        let billing_source = shape::optional(object, "billingSource", at, shape::text)?;
        let list_cost = shape::optional(object, "listCost", at, shape::number)?;
        let allowance = shape::optional(object, "allowance", at, shape::number)?;
        Ok(Self {
            input,
            output,
            cached,
            cache_write,
            reasoning,
            cost,
            upstream_cost,
            byok,
            billing_source,
            list_cost,
            allowance,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("input", Value::Number(self.input));
        object.insert("output", Value::Number(self.output));
        if let Some(held) = &self.cached {
            object.insert("cached", Value::Number(*held));
        }
        if let Some(held) = &self.cache_write {
            object.insert("cacheWrite", Value::Number(*held));
        }
        if let Some(held) = &self.reasoning {
            object.insert("reasoning", Value::Number(*held));
        }
        if let Some(held) = &self.cost {
            object.insert("cost", Value::Number(*held));
        }
        if let Some(held) = &self.upstream_cost {
            object.insert("upstreamCost", Value::Number(*held));
        }
        if let Some(held) = &self.byok {
            object.insert("byok", Value::Bool(*held));
        }
        if let Some(held) = &self.billing_source {
            object.insert("billingSource", Value::from(held.as_str()));
        }
        if let Some(held) = &self.list_cost {
            object.insert("listCost", Value::Number(*held));
        }
        if let Some(held) = &self.allowance {
            object.insert("allowance", Value::Number(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StepResult {
    pub parts: Vec<ReplyPart>,
    pub finish: FinishReason,
    pub usage: Option<NormalizedUsage>,
    pub serving: Serving,
}

impl Shape for StepResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let parts = shape::required(object, "parts", at, |value, at| {
            shape::list(value, at, ReplyPart::decode_at)
        })?;
        let finish = shape::required(object, "finish", at, FinishReason::decode_at)?;
        let usage = shape::optional(object, "usage", at, NormalizedUsage::decode_at)?;
        let serving = shape::required(object, "serving", at, Serving::decode_at)?;
        Ok(Self {
            parts,
            finish,
            usage,
            serving,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "parts",
            Value::Array(self.parts.iter().map(|held| held.encode()).collect()),
        );
        object.insert("finish", self.finish.encode());
        if let Some(held) = &self.usage {
            object.insert("usage", held.encode());
        }
        object.insert("serving", self.serving.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FailureCategory {
    KeyMissing,
    KeyRejected,
    OutOfCredit,
    Allowance,
    Refused,
    NotFound,
    BadRequest,
    ReplayRejected,
    RateLimited,
    Overloaded,
    Unavailable,
    Network,
    Stream,
    Empty,
    Truncated,
    Idle,
}

impl FailureCategory {
    pub const ALL: [Self; 16] = [
        Self::KeyMissing,
        Self::KeyRejected,
        Self::OutOfCredit,
        Self::Allowance,
        Self::Refused,
        Self::NotFound,
        Self::BadRequest,
        Self::ReplayRejected,
        Self::RateLimited,
        Self::Overloaded,
        Self::Unavailable,
        Self::Network,
        Self::Stream,
        Self::Empty,
        Self::Truncated,
        Self::Idle,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::KeyMissing => "keyMissing",
            Self::KeyRejected => "keyRejected",
            Self::OutOfCredit => "outOfCredit",
            Self::Allowance => "allowance",
            Self::Refused => "refused",
            Self::NotFound => "notFound",
            Self::BadRequest => "badRequest",
            Self::ReplayRejected => "replayRejected",
            Self::RateLimited => "rateLimited",
            Self::Overloaded => "overloaded",
            Self::Unavailable => "unavailable",
            Self::Network => "network",
            Self::Stream => "stream",
            Self::Empty => "empty",
            Self::Truncated => "truncated",
            Self::Idle => "idle",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for FailureCategory {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Failure {
    pub category: FailureCategory,
    pub status: FailureStatus,
    pub retry_after_ms: Option<i64>,
    pub resets_at: Option<i64>,
    pub message: String,
    pub reasons: Option<Vec<String>>,
    pub billed: Option<FailureBilled>,
}

impl Shape for Failure {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let category = shape::required(object, "category", at, FailureCategory::decode_at)?;
        let status = shape::required(object, "status", at, FailureStatus::decode_at)?;
        let retry_after_ms = shape::optional(object, "retryAfterMs", at, shape::integer)?;
        let resets_at = shape::optional(object, "resetsAt", at, shape::integer)?;
        let message = shape::required(object, "message", at, shape::text)?;
        let reasons = shape::optional(object, "reasons", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let billed = shape::optional(object, "billed", at, FailureBilled::decode_at)?;
        Ok(Self {
            category,
            status,
            retry_after_ms,
            resets_at,
            message,
            reasons,
            billed,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("category", self.category.encode());
        object.insert("status", self.status.encode());
        if let Some(held) = &self.retry_after_ms {
            object.insert("retryAfterMs", Value::Number(*held as f64));
        }
        if let Some(held) = &self.resets_at {
            object.insert("resetsAt", Value::Number(*held as f64));
        }
        object.insert("message", Value::from(self.message.as_str()));
        if let Some(held) = &self.reasons {
            object.insert(
                "reasons",
                Value::Array(held.iter().map(|held| Value::from(held.as_str())).collect()),
            );
        }
        if let Some(held) = &self.billed {
            object.insert("billed", held.encode());
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FailureStatus {
    Number(f64),
    Text(String),
}

impl Shape for FailureStatus {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        if let Ok(held) = shape::number(value, at) {
            return Ok(Self::Number(held));
        }
        if let Ok(held) = shape::text(value, at) {
            return Ok(Self::Text(held));
        }
        Err(shape::refused(at, "no branch matches"))
    }

    fn encode(&self) -> Value {
        match self {
            Self::Number(held) => Value::Number(*held),
            Self::Text(held) => Value::from(held.as_str()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailureBilled {
    pub usage: Option<NormalizedUsage>,
    pub host: Option<String>,
    pub generation_id: Option<String>,
}

impl Shape for FailureBilled {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let usage = shape::optional(object, "usage", at, NormalizedUsage::decode_at)?;
        let host = shape::optional(object, "host", at, shape::text)?;
        let generation_id = shape::optional(object, "generationId", at, shape::text)?;
        Ok(Self {
            usage,
            host,
            generation_id,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        if let Some(held) = &self.usage {
            object.insert("usage", held.encode());
        }
        if let Some(held) = &self.host {
            object.insert("host", Value::from(held.as_str()));
        }
        if let Some(held) = &self.generation_id {
            object.insert("generationId", Value::from(held.as_str()));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ApiFamily {
    ChatCompletions,
    Responses,
    Messages,
    GenerateContent,
}

impl ApiFamily {
    pub const ALL: [Self; 4] = [
        Self::ChatCompletions,
        Self::Responses,
        Self::Messages,
        Self::GenerateContent,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::ChatCompletions => "chat-completions",
            Self::Responses => "responses",
            Self::Messages => "messages",
            Self::GenerateContent => "generate-content",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for ApiFamily {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApiFacts {
    pub family: ApiFamily,
    pub structured: bool,
    pub choice: Vec<String>,
    pub strict: bool,
    pub reasoning: Option<String>,
    pub toggle: Option<bool>,
}

impl Shape for ApiFacts {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let family = shape::required(object, "family", at, ApiFamily::decode_at)?;
        let structured = shape::required(object, "structured", at, shape::flag)?;
        let choice = shape::required(object, "choice", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let strict = shape::required(object, "strict", at, shape::flag)?;
        let reasoning = shape::optional(object, "reasoning", at, shape::text)?;
        let toggle = shape::optional(object, "toggle", at, shape::flag)?;
        Ok(Self {
            family,
            structured,
            choice,
            strict,
            reasoning,
            toggle,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("family", self.family.encode());
        object.insert("structured", Value::Bool(self.structured));
        object.insert(
            "choice",
            Value::Array(
                self.choice
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        object.insert("strict", Value::Bool(self.strict));
        if let Some(held) = &self.reasoning {
            object.insert("reasoning", Value::from(held.as_str()));
        }
        if let Some(held) = &self.toggle {
            object.insert("toggle", Value::Bool(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CodecFacts {
    pub api: ApiFacts,
    pub efforts: Vec<Effort>,
    pub output_limit: Option<i64>,
}

impl Shape for CodecFacts {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let api = shape::required(object, "api", at, ApiFacts::decode_at)?;
        let efforts = shape::required(object, "efforts", at, |value, at| {
            shape::list(value, at, Effort::decode_at)
        })?;
        let output_limit = shape::optional(object, "outputLimit", at, shape::integer)?;
        Ok(Self {
            api,
            efforts,
            output_limit,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("api", self.api.encode());
        object.insert(
            "efforts",
            Value::Array(self.efforts.iter().map(|held| held.encode()).collect()),
        );
        if let Some(held) = &self.output_limit {
            object.insert("outputLimit", Value::Number(*held as f64));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Encoded {
    pub path: String,
    pub body: Value,
    pub headers: Option<Record<String>>,
}

impl Shape for Encoded {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let path = shape::required(object, "path", at, shape::text)?;
        let body = shape::required(object, "body", at, shape::any)?;
        let headers = shape::optional(object, "headers", at, |value, at| {
            shape::record(value, at, shape::text)
        })?;
        Ok(Self {
            path,
            body,
            headers,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("path", Value::from(self.path.as_str()));
        object.insert("body", self.body.clone());
        if let Some(held) = &self.headers {
            object.insert(
                "headers",
                shape::record_value(held, |held| Value::from(held.as_str())),
            );
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SseItem {
    Comment(SseItemComment),
    Data(SseItemData),
    Done(SseItemDone),
}

impl Shape for SseItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        match shape::tag(object, "kind", at)? {
            "comment" => SseItemComment::decode_at(value, at).map(Self::Comment),
            "data" => SseItemData::decode_at(value, at).map(Self::Data),
            "done" => SseItemDone::decode_at(value, at).map(Self::Done),
            _ => Err(shape::no_branch(at, "kind")),
        }
    }

    fn encode(&self) -> Value {
        match self {
            Self::Comment(held) => held.encode(),
            Self::Data(held) => held.encode(),
            Self::Done(held) => held.encode(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SseItemComment {
    pub text: String,
}

impl Shape for SseItemComment {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "comment")
        })?;
        let text = shape::required(object, "text", at, shape::text)?;
        Ok(Self { text })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("comment"));
        object.insert("text", Value::from(self.text.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SseItemData {
    pub data: String,
    pub event: Option<String>,
}

impl Shape for SseItemData {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "data")
        })?;
        let data = shape::required(object, "data", at, shape::text)?;
        let event = shape::optional(object, "event", at, shape::text)?;
        Ok(Self { data, event })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("data"));
        object.insert("data", Value::from(self.data.as_str()));
        if let Some(held) = &self.event {
            object.insert("event", Value::from(held.as_str()));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SseItemDone {}

impl Shape for SseItemDone {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "done")
        })?;
        Ok(Self {})
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("done"));
        Value::Object(object)
    }
}

pub type HeaderList = Vec<(String, String)>;

pub fn decode_header_list(value: &Value, at: &mut Path) -> Result<HeaderList, Refused> {
    shape::list(value, at, |value, at| {
        shape::tuple(value, at, 2).and_then(|items| {
            Ok((
                shape::place(items, 0, at, shape::text)?,
                shape::place(items, 1, at, shape::text)?,
            ))
        })
    })
}

pub fn encode_header_list(held: &HeaderList) -> Value {
    Value::Array(
        held.iter()
            .map(|held| {
                Value::Array(vec![
                    Value::from(held.0.as_str()),
                    Value::from(held.1.as_str()),
                ])
            })
            .collect(),
    )
}

/// Decodes then encodes a named shape.
pub fn normalize(shape: &str, value: &Value) -> Option<Result<Value, Refused>> {
    let at = &mut Path::new();
    Some(match shape {
        "Effort" => Effort::decode_at(value, at).map(|held| held.encode()),
        "ToolSpec" => ToolSpec::decode_at(value, at).map(|held| held.encode()),
        "TextPart" => TextPart::decode_at(value, at).map(|held| held.encode()),
        "ThoughtPart" => ThoughtPart::decode_at(value, at).map(|held| held.encode()),
        "TurnPart" => TurnPart::decode_at(value, at).map(|held| held.encode()),
        "Turn" => Turn::decode_at(value, at).map(|held| held.encode()),
        "CanonicalRequest" => CanonicalRequest::decode_at(value, at).map(|held| held.encode()),
        "ThoughtDelta" => ThoughtDelta::decode_at(value, at).map(|held| held.encode()),
        "StepEvent" => StepEvent::decode_at(value, at).map(|held| held.encode()),
        "ReplyPart" => ReplyPart::decode_at(value, at).map(|held| held.encode()),
        "Serving" => Serving::decode_at(value, at).map(|held| held.encode()),
        "FinishReason" => FinishReason::decode_at(value, at).map(|held| held.encode()),
        "NormalizedUsage" => NormalizedUsage::decode_at(value, at).map(|held| held.encode()),
        "StepResult" => StepResult::decode_at(value, at).map(|held| held.encode()),
        "FailureCategory" => FailureCategory::decode_at(value, at).map(|held| held.encode()),
        "Failure" => Failure::decode_at(value, at).map(|held| held.encode()),
        "ApiFamily" => ApiFamily::decode_at(value, at).map(|held| held.encode()),
        "ApiFacts" => ApiFacts::decode_at(value, at).map(|held| held.encode()),
        "CodecFacts" => CodecFacts::decode_at(value, at).map(|held| held.encode()),
        "Encoded" => Encoded::decode_at(value, at).map(|held| held.encode()),
        "SseItem" => SseItem::decode_at(value, at).map(|held| held.encode()),
        "HeaderList" => decode_header_list(value, at).map(|held| encode_header_list(&held)),
        _ => return None,
    })
}
