//! Generated from schema/frames.output.json by scripts/contract.ts. Do not edit.

#![allow(
    clippy::enum_variant_names,
    clippy::large_enum_variant,
    clippy::ptr_arg
)]

use crate::json::{Object, Value};
use crate::shape::{self, Path, Refused, Shape};

#[derive(Debug, Clone, PartialEq)]
pub enum PathPart {
    Text(String),
    Integer(u64),
}

impl Shape for PathPart {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        if let Ok(held) = shape::text(value, at) {
            return Ok(Self::Text(held));
        }
        if let Ok(held) = shape::natural(value, at) {
            return Ok(Self::Integer(held));
        }
        Err(shape::refused(at, "no branch matches"))
    }

    fn encode(&self) -> Value {
        match self {
            Self::Text(held) => Value::from(held.as_str()),
            Self::Integer(held) => Value::Number(*held as f64),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Fixed {
    pub now_ms: i64,
    pub step_ms: u64,
    pub id_prefix: String,
}

impl Shape for Fixed {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let now_ms = shape::required(object, "nowMs", at, shape::integer)?;
        let step_ms = shape::defaulted(object, "stepMs", at, r#"0"#, shape::natural)?;
        let id_prefix = shape::required(object, "idPrefix", at, shape::text)?;
        Ok(Self {
            now_ms,
            step_ms,
            id_prefix,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("nowMs", Value::Number(self.now_ms as f64));
        object.insert("stepMs", Value::Number(self.step_ms as f64));
        object.insert("idPrefix", Value::from(self.id_prefix.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Hello {
    pub version: String,
}

impl Shape for Hello {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "type", at, |value, at| {
            shape::is_text(value, at, "hello")
        })?;
        let version = shape::required(object, "version", at, shape::text)?;
        Ok(Self { version })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("type", Value::from("hello"));
        object.insert("version", Value::from(self.version.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Start {
    pub run: String,
    pub command: String,
    pub input: Value,
    pub fixed: Option<Fixed>,
}

impl Shape for Start {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "type", at, |value, at| {
            shape::is_text(value, at, "start")
        })?;
        let run = shape::required(object, "run", at, |value, at| {
            shape::text_within(value, at, Some(1), None)
        })?;
        let command = shape::required(object, "command", at, |value, at| {
            shape::text_within(value, at, Some(1), None)
        })?;
        let input = shape::required(object, "input", at, shape::any)?;
        let fixed = shape::optional(object, "fixed", at, Fixed::decode_at)?;
        Ok(Self {
            run,
            command,
            input,
            fixed,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("type", Value::from("start"));
        object.insert("run", Value::from(self.run.as_str()));
        object.insert("command", Value::from(self.command.as_str()));
        object.insert("input", self.input.clone());
        if let Some(held) = &self.fixed {
            object.insert("fixed", held.encode());
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Steer {
    pub run: String,
    pub text: String,
}

impl Shape for Steer {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "type", at, |value, at| {
            shape::is_text(value, at, "steer")
        })?;
        let run = shape::required(object, "run", at, |value, at| {
            shape::text_within(value, at, Some(1), None)
        })?;
        let text = shape::required(object, "text", at, shape::text)?;
        Ok(Self { run, text })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("type", Value::from("steer"));
        object.insert("run", Value::from(self.run.as_str()));
        object.insert("text", Value::from(self.text.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Interrupt {
    pub run: String,
}

impl Shape for Interrupt {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "type", at, |value, at| {
            shape::is_text(value, at, "interrupt")
        })?;
        let run = shape::required(object, "run", at, |value, at| {
            shape::text_within(value, at, Some(1), None)
        })?;
        Ok(Self { run })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("type", Value::from("interrupt"));
        object.insert("run", Value::from(self.run.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Reply {
    pub run: String,
    pub id: String,
    pub value: Value,
}

impl Shape for Reply {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "type", at, |value, at| {
            shape::is_text(value, at, "reply")
        })?;
        let run = shape::required(object, "run", at, |value, at| {
            shape::text_within(value, at, Some(1), None)
        })?;
        let id = shape::required(object, "id", at, |value, at| {
            shape::text_within(value, at, Some(1), None)
        })?;
        let value_field = shape::required(object, "value", at, shape::any)?;
        Ok(Self {
            run,
            id,
            value: value_field,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("type", Value::from("reply"));
        object.insert("run", Value::from(self.run.as_str()));
        object.insert("id", Value::from(self.id.as_str()));
        object.insert("value", self.value.clone());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReplyFailed {
    pub run: String,
    pub id: String,
    pub message: String,
}

impl Shape for ReplyFailed {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "type", at, |value, at| {
            shape::is_text(value, at, "reply-failed")
        })?;
        let run = shape::required(object, "run", at, |value, at| {
            shape::text_within(value, at, Some(1), None)
        })?;
        let id = shape::required(object, "id", at, |value, at| {
            shape::text_within(value, at, Some(1), None)
        })?;
        let message = shape::required(object, "message", at, shape::text)?;
        Ok(Self { run, id, message })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("type", Value::from("reply-failed"));
        object.insert("run", Value::from(self.run.as_str()));
        object.insert("id", Value::from(self.id.as_str()));
        object.insert("message", Value::from(self.message.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum HostFrame {
    Hello(Hello),
    Start(Start),
    Steer(Steer),
    Interrupt(Interrupt),
    Reply(Reply),
    ReplyFailed(ReplyFailed),
}

impl Shape for HostFrame {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        match shape::tag(object, "type", at)? {
            "hello" => Hello::decode_at(value, at).map(Self::Hello),
            "start" => Start::decode_at(value, at).map(Self::Start),
            "steer" => Steer::decode_at(value, at).map(Self::Steer),
            "interrupt" => Interrupt::decode_at(value, at).map(Self::Interrupt),
            "reply" => Reply::decode_at(value, at).map(Self::Reply),
            "reply-failed" => ReplyFailed::decode_at(value, at).map(Self::ReplyFailed),
            _ => Err(shape::no_branch(at, "type")),
        }
    }

    fn encode(&self) -> Value {
        match self {
            Self::Hello(held) => held.encode(),
            Self::Start(held) => held.encode(),
            Self::Steer(held) => held.encode(),
            Self::Interrupt(held) => held.encode(),
            Self::Reply(held) => held.encode(),
            Self::ReplyFailed(held) => held.encode(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Welcome {
    pub version: String,
}

impl Shape for Welcome {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "type", at, |value, at| {
            shape::is_text(value, at, "welcome")
        })?;
        let version = shape::required(object, "version", at, shape::text)?;
        Ok(Self { version })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("type", Value::from("welcome"));
        object.insert("version", Value::from(self.version.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Reject {
    pub version: String,
    pub reason: String,
}

impl Shape for Reject {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "type", at, |value, at| {
            shape::is_text(value, at, "reject")
        })?;
        let version = shape::required(object, "version", at, shape::text)?;
        let reason = shape::required(object, "reason", at, shape::text)?;
        Ok(Self { version, reason })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("type", Value::from("reject"));
        object.insert("version", Value::from(self.version.as_str()));
        object.insert("reason", Value::from(self.reason.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunEvent {
    pub run: String,
    pub seq: u64,
    pub event: Value,
}

impl Shape for RunEvent {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "type", at, |value, at| {
            shape::is_text(value, at, "event")
        })?;
        let run = shape::required(object, "run", at, |value, at| {
            shape::text_within(value, at, Some(1), None)
        })?;
        let seq = shape::required(object, "seq", at, |value, at| {
            shape::natural(value, at).and_then(|held| shape::at_least(held, 1, at))
        })?;
        let event = shape::required(object, "event", at, shape::any)?;
        Ok(Self { run, seq, event })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("type", Value::from("event"));
        object.insert("run", Value::from(self.run.as_str()));
        object.insert("seq", Value::Number(self.seq as f64));
        object.insert("event", self.event.clone());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Request {
    pub run: String,
    pub id: String,
    pub kind: String,
    pub input: Value,
}

impl Shape for Request {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "type", at, |value, at| {
            shape::is_text(value, at, "request")
        })?;
        let run = shape::required(object, "run", at, |value, at| {
            shape::text_within(value, at, Some(1), None)
        })?;
        let id = shape::required(object, "id", at, |value, at| {
            shape::text_within(value, at, Some(1), None)
        })?;
        let kind = shape::required(object, "kind", at, |value, at| {
            shape::text_within(value, at, Some(1), None)
        })?;
        let input = shape::required(object, "input", at, shape::any)?;
        Ok(Self {
            run,
            id,
            kind,
            input,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("type", Value::from("request"));
        object.insert("run", Value::from(self.run.as_str()));
        object.insert("id", Value::from(self.id.as_str()));
        object.insert("kind", Value::from(self.kind.as_str()));
        object.insert("input", self.input.clone());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Done {
    pub run: String,
    pub seq: u64,
    pub result: Value,
}

impl Shape for Done {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "type", at, |value, at| {
            shape::is_text(value, at, "done")
        })?;
        let run = shape::required(object, "run", at, |value, at| {
            shape::text_within(value, at, Some(1), None)
        })?;
        let seq = shape::required(object, "seq", at, |value, at| {
            shape::natural(value, at).and_then(|held| shape::at_least(held, 1, at))
        })?;
        let result = shape::required(object, "result", at, shape::any)?;
        Ok(Self { run, seq, result })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("type", Value::from("done"));
        object.insert("run", Value::from(self.run.as_str()));
        object.insert("seq", Value::Number(self.seq as f64));
        object.insert("result", self.result.clone());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Failed {
    pub run: String,
    pub seq: u64,
    pub code: RunFailure,
    pub path: Option<Vec<PathPart>>,
    pub message: String,
}

impl Shape for Failed {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "type", at, |value, at| {
            shape::is_text(value, at, "failed")
        })?;
        let run = shape::required(object, "run", at, |value, at| {
            shape::text_within(value, at, Some(1), None)
        })?;
        let seq = shape::required(object, "seq", at, |value, at| {
            shape::natural(value, at).and_then(|held| shape::at_least(held, 1, at))
        })?;
        let code = shape::required(object, "code", at, RunFailure::decode_at)?;
        let path = shape::optional(object, "path", at, |value, at| {
            shape::list(value, at, PathPart::decode_at)
        })?;
        let message = shape::required(object, "message", at, shape::text)?;
        Ok(Self {
            run,
            seq,
            code,
            path,
            message,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("type", Value::from("failed"));
        object.insert("run", Value::from(self.run.as_str()));
        object.insert("seq", Value::Number(self.seq as f64));
        object.insert("code", self.code.encode());
        if let Some(held) = &self.path {
            object.insert(
                "path",
                Value::Array(held.iter().map(|held| held.encode()).collect()),
            );
        }
        object.insert("message", Value::from(self.message.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FaultCode {
    BadFrame,
    OversizedFrame,
    NotGreeted,
    UnknownRun,
    DuplicateRun,
    UnknownReply,
}

impl FaultCode {
    pub const ALL: [Self; 6] = [
        Self::BadFrame,
        Self::OversizedFrame,
        Self::NotGreeted,
        Self::UnknownRun,
        Self::DuplicateRun,
        Self::UnknownReply,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::BadFrame => "bad_frame",
            Self::OversizedFrame => "oversized_frame",
            Self::NotGreeted => "not_greeted",
            Self::UnknownRun => "unknown_run",
            Self::DuplicateRun => "duplicate_run",
            Self::UnknownReply => "unknown_reply",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for FaultCode {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RunFailure {
    Interrupted,
    UnknownCommand,
    BadInput,
    HostRequest,
    Error,
}

impl RunFailure {
    pub const ALL: [Self; 5] = [
        Self::Interrupted,
        Self::UnknownCommand,
        Self::BadInput,
        Self::HostRequest,
        Self::Error,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Interrupted => "interrupted",
            Self::UnknownCommand => "unknown_command",
            Self::BadInput => "bad_input",
            Self::HostRequest => "host_request",
            Self::Error => "error",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for RunFailure {
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
pub struct Fault {
    pub code: FaultCode,
    pub run: Option<String>,
    pub path: Option<Vec<PathPart>>,
    pub message: String,
}

impl Shape for Fault {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "type", at, |value, at| {
            shape::is_text(value, at, "fault")
        })?;
        let code = shape::required(object, "code", at, FaultCode::decode_at)?;
        let run = shape::optional(object, "run", at, |value, at| {
            shape::text_within(value, at, Some(1), None)
        })?;
        let path = shape::optional(object, "path", at, |value, at| {
            shape::list(value, at, PathPart::decode_at)
        })?;
        let message = shape::required(object, "message", at, shape::text)?;
        Ok(Self {
            code,
            run,
            path,
            message,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("type", Value::from("fault"));
        object.insert("code", self.code.encode());
        if let Some(held) = &self.run {
            object.insert("run", Value::from(held.as_str()));
        }
        if let Some(held) = &self.path {
            object.insert(
                "path",
                Value::Array(held.iter().map(|held| held.encode()).collect()),
            );
        }
        object.insert("message", Value::from(self.message.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AdapterFrame {
    Welcome(Welcome),
    Reject(Reject),
    Event(RunEvent),
    Request(Request),
    Done(Done),
    Failed(Failed),
    Fault(Fault),
}

impl Shape for AdapterFrame {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        match shape::tag(object, "type", at)? {
            "welcome" => Welcome::decode_at(value, at).map(Self::Welcome),
            "reject" => Reject::decode_at(value, at).map(Self::Reject),
            "event" => RunEvent::decode_at(value, at).map(Self::Event),
            "request" => Request::decode_at(value, at).map(Self::Request),
            "done" => Done::decode_at(value, at).map(Self::Done),
            "failed" => Failed::decode_at(value, at).map(Self::Failed),
            "fault" => Fault::decode_at(value, at).map(Self::Fault),
            _ => Err(shape::no_branch(at, "type")),
        }
    }

    fn encode(&self) -> Value {
        match self {
            Self::Welcome(held) => held.encode(),
            Self::Reject(held) => held.encode(),
            Self::Event(held) => held.encode(),
            Self::Request(held) => held.encode(),
            Self::Done(held) => held.encode(),
            Self::Failed(held) => held.encode(),
            Self::Fault(held) => held.encode(),
        }
    }
}

/// Decodes then encodes a named shape.
pub fn normalize(shape: &str, value: &Value) -> Option<Result<Value, Refused>> {
    let at = &mut Path::new();
    Some(match shape {
        "PathPart" => PathPart::decode_at(value, at).map(|held| held.encode()),
        "Fixed" => Fixed::decode_at(value, at).map(|held| held.encode()),
        "Hello" => Hello::decode_at(value, at).map(|held| held.encode()),
        "Start" => Start::decode_at(value, at).map(|held| held.encode()),
        "Steer" => Steer::decode_at(value, at).map(|held| held.encode()),
        "Interrupt" => Interrupt::decode_at(value, at).map(|held| held.encode()),
        "Reply" => Reply::decode_at(value, at).map(|held| held.encode()),
        "ReplyFailed" => ReplyFailed::decode_at(value, at).map(|held| held.encode()),
        "HostFrame" => HostFrame::decode_at(value, at).map(|held| held.encode()),
        "Welcome" => Welcome::decode_at(value, at).map(|held| held.encode()),
        "Reject" => Reject::decode_at(value, at).map(|held| held.encode()),
        "RunEvent" => RunEvent::decode_at(value, at).map(|held| held.encode()),
        "Request" => Request::decode_at(value, at).map(|held| held.encode()),
        "Done" => Done::decode_at(value, at).map(|held| held.encode()),
        "Failed" => Failed::decode_at(value, at).map(|held| held.encode()),
        "FaultCode" => FaultCode::decode_at(value, at).map(|held| held.encode()),
        "RunFailure" => RunFailure::decode_at(value, at).map(|held| held.encode()),
        "Fault" => Fault::decode_at(value, at).map(|held| held.encode()),
        "AdapterFrame" => AdapterFrame::decode_at(value, at).map(|held| held.encode()),
        _ => return None,
    })
}
