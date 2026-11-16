//! Generated from schema/provider.output.json by scripts/contract.ts. Do not edit.

#![allow(
    clippy::enum_variant_names,
    clippy::large_enum_variant,
    clippy::ptr_arg
)]

use crate::json::{Object, Value};
use crate::shape::{self, Path, Refused, Shape};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProviderKindName {
    Openrouter,
    Opencode,
    Ollama,
    Lmstudio,
    Llamacpp,
    Compatible,
}

impl ProviderKindName {
    pub const ALL: [Self; 6] = [
        Self::Openrouter,
        Self::Opencode,
        Self::Ollama,
        Self::Lmstudio,
        Self::Llamacpp,
        Self::Compatible,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Openrouter => "openrouter",
            Self::Opencode => "opencode",
            Self::Ollama => "ollama",
            Self::Lmstudio => "lmstudio",
            Self::Llamacpp => "llamacpp",
            Self::Compatible => "compatible",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for ProviderKindName {
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
pub enum ProviderPlan {
    Go,
    Payg,
}

impl ProviderPlan {
    pub const ALL: [Self; 2] = [Self::Go, Self::Payg];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Go => "go",
            Self::Payg => "payg",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for ProviderPlan {
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
pub struct ProviderEntry {
    pub id: String,
    pub kind: ProviderKindName,
    pub label: String,
    pub plan: Option<ProviderPlan>,
    pub url: Option<String>,
    pub extra_usage: Option<bool>,
}

impl Shape for ProviderEntry {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let id = shape::required(object, "id", at, shape::text)?;
        let kind = shape::required(object, "kind", at, ProviderKindName::decode_at)?;
        let label = shape::required(object, "label", at, shape::text)?;
        let plan = shape::optional(object, "plan", at, ProviderPlan::decode_at)?;
        let url = shape::optional(object, "url", at, shape::text)?;
        let extra_usage = shape::optional(object, "extraUsage", at, shape::flag)?;
        Ok(Self {
            id,
            kind,
            label,
            plan,
            url,
            extra_usage,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("id", Value::from(self.id.as_str()));
        object.insert("kind", self.kind.encode());
        object.insert("label", Value::from(self.label.as_str()));
        if let Some(held) = &self.plan {
            object.insert("plan", held.encode());
        }
        if let Some(held) = &self.url {
            object.insert("url", Value::from(held.as_str()));
        }
        if let Some(held) = &self.extra_usage {
            object.insert("extraUsage", Value::Bool(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EntryFields {
    pub kind: ProviderKindName,
    pub label: String,
    pub plan: Option<ProviderPlan>,
    pub url: Option<String>,
    pub extra_usage: Option<bool>,
}

impl Shape for EntryFields {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let kind = shape::required(object, "kind", at, ProviderKindName::decode_at)?;
        let label = shape::required(object, "label", at, shape::text)?;
        let plan = shape::optional(object, "plan", at, ProviderPlan::decode_at)?;
        let url = shape::optional(object, "url", at, shape::text)?;
        let extra_usage = shape::optional(object, "extraUsage", at, shape::flag)?;
        Ok(Self {
            kind,
            label,
            plan,
            url,
            extra_usage,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", self.kind.encode());
        object.insert("label", Value::from(self.label.as_str()));
        if let Some(held) = &self.plan {
            object.insert("plan", held.encode());
        }
        if let Some(held) = &self.url {
            object.insert("url", Value::from(held.as_str()));
        }
        if let Some(held) = &self.extra_usage {
            object.insert("extraUsage", Value::Bool(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum KeyState {
    None(KeyStateNone),
    Checking(KeyStateChecking),
    Ok(KeyStateOk),
    Rejected(KeyStateRejected),
    Unreachable(KeyStateUnreachable),
    Invalid(KeyStateInvalid),
}

impl Shape for KeyState {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        match shape::tag(object, "kind", at)? {
            "none" => KeyStateNone::decode_at(value, at).map(Self::None),
            "checking" => KeyStateChecking::decode_at(value, at).map(Self::Checking),
            "ok" => KeyStateOk::decode_at(value, at).map(Self::Ok),
            "rejected" => KeyStateRejected::decode_at(value, at).map(Self::Rejected),
            "unreachable" => KeyStateUnreachable::decode_at(value, at).map(Self::Unreachable),
            "invalid" => KeyStateInvalid::decode_at(value, at).map(Self::Invalid),
            _ => Err(shape::no_branch(at, "kind")),
        }
    }

    fn encode(&self) -> Value {
        match self {
            Self::None(held) => held.encode(),
            Self::Checking(held) => held.encode(),
            Self::Ok(held) => held.encode(),
            Self::Rejected(held) => held.encode(),
            Self::Unreachable(held) => held.encode(),
            Self::Invalid(held) => held.encode(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KeyStateNone {}

impl Shape for KeyStateNone {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "none")
        })?;
        Ok(Self {})
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("none"));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KeyStateChecking {}

impl Shape for KeyStateChecking {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "checking")
        })?;
        Ok(Self {})
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("checking"));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KeyStateOk {
    pub usage: Option<f64>,
    pub limit: Option<f64>,
    pub models: Option<f64>,
}

impl Shape for KeyStateOk {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "ok")
        })?;
        let usage = shape::optional(object, "usage", at, shape::number)?;
        let limit = shape::optional(object, "limit", at, shape::number)?;
        let models = shape::optional(object, "models", at, shape::number)?;
        Ok(Self {
            usage,
            limit,
            models,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("ok"));
        if let Some(held) = &self.usage {
            object.insert("usage", Value::Number(*held));
        }
        if let Some(held) = &self.limit {
            object.insert("limit", Value::Number(*held));
        }
        if let Some(held) = &self.models {
            object.insert("models", Value::Number(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KeyStateRejected {}

impl Shape for KeyStateRejected {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "rejected")
        })?;
        Ok(Self {})
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("rejected"));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KeyStateUnreachable {}

impl Shape for KeyStateUnreachable {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "unreachable")
        })?;
        Ok(Self {})
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("unreachable"));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KeyStateInvalid {}

impl Shape for KeyStateInvalid {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "invalid")
        })?;
        Ok(Self {})
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("invalid"));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KindLine {
    pub kind: ProviderKindName,
    pub plan: Option<ProviderPlan>,
    pub label: String,
    pub field: KindLineField,
    pub url: Option<String>,
    pub optional_key: Option<()>,
}

impl Shape for KindLine {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let kind = shape::required(object, "kind", at, ProviderKindName::decode_at)?;
        let plan = shape::optional(object, "plan", at, ProviderPlan::decode_at)?;
        let label = shape::required(object, "label", at, shape::text)?;
        let field = shape::required(object, "field", at, KindLineField::decode_at)?;
        let url = shape::optional(object, "url", at, shape::text)?;
        let optional_key = shape::optional(object, "optionalKey", at, |value, at| {
            shape::is_flag(value, at, true)
        })?;
        Ok(Self {
            kind,
            plan,
            label,
            field,
            url,
            optional_key,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", self.kind.encode());
        if let Some(held) = &self.plan {
            object.insert("plan", held.encode());
        }
        object.insert("label", Value::from(self.label.as_str()));
        object.insert("field", self.field.encode());
        if let Some(held) = &self.url {
            object.insert("url", Value::from(held.as_str()));
        }
        if self.optional_key.is_some() {
            object.insert("optionalKey", Value::Bool(true));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KindLineField {
    Key,
    Url,
}

impl KindLineField {
    pub const ALL: [Self; 2] = [Self::Key, Self::Url];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Key => "key",
            Self::Url => "url",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for KindLineField {
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
pub struct ProviderView {
    pub id: String,
    pub kind: ProviderKindName,
    pub plan: Option<ProviderPlan>,
    pub label: String,
    pub url: Option<String>,
    pub status: KeyState,
    pub facts: Option<String>,
    pub balance: Option<()>,
    pub extra_usage: Option<bool>,
}

impl Shape for ProviderView {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let id = shape::required(object, "id", at, shape::text)?;
        let kind = shape::required(object, "kind", at, ProviderKindName::decode_at)?;
        let plan = shape::optional(object, "plan", at, ProviderPlan::decode_at)?;
        let label = shape::required(object, "label", at, shape::text)?;
        let url = shape::optional(object, "url", at, shape::text)?;
        let status = shape::required(object, "status", at, KeyState::decode_at)?;
        let facts = shape::optional(object, "facts", at, shape::text)?;
        let balance = shape::optional(object, "balance", at, |value, at| {
            shape::is_flag(value, at, true)
        })?;
        let extra_usage = shape::optional(object, "extraUsage", at, shape::flag)?;
        Ok(Self {
            id,
            kind,
            plan,
            label,
            url,
            status,
            facts,
            balance,
            extra_usage,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("id", Value::from(self.id.as_str()));
        object.insert("kind", self.kind.encode());
        if let Some(held) = &self.plan {
            object.insert("plan", held.encode());
        }
        object.insert("label", Value::from(self.label.as_str()));
        if let Some(held) = &self.url {
            object.insert("url", Value::from(held.as_str()));
        }
        object.insert("status", self.status.encode());
        if let Some(held) = &self.facts {
            object.insert("facts", Value::from(held.as_str()));
        }
        if self.balance.is_some() {
            object.insert("balance", Value::Bool(true));
        }
        if let Some(held) = &self.extra_usage {
            object.insert("extraUsage", Value::Bool(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyReach {
    Ok,
    KeyMissing,
    KeyRejected,
    Unreachable,
}

impl KeyReach {
    pub const ALL: [Self; 4] = [
        Self::Ok,
        Self::KeyMissing,
        Self::KeyRejected,
        Self::Unreachable,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::KeyMissing => "keyMissing",
            Self::KeyRejected => "keyRejected",
            Self::Unreachable => "unreachable",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for KeyReach {
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
pub struct Limits {
    pub limit: Option<f64>,
    pub reset: Option<String>,
}

impl Shape for Limits {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let limit = shape::optional(object, "limit", at, shape::number)?;
        let reset = shape::optional(object, "reset", at, shape::text)?;
        Ok(Self { limit, reset })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        if let Some(held) = &self.limit {
            object.insert("limit", Value::Number(*held));
        }
        if let Some(held) = &self.reset {
            object.insert("reset", Value::from(held.as_str()));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Billed {
    pub billing: String,
    pub allowance: Option<f64>,
    pub cost: Option<f64>,
}

impl Shape for Billed {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let billing = shape::required(object, "billing", at, shape::text)?;
        let allowance = shape::optional(object, "allowance", at, shape::number)?;
        let cost = shape::optional(object, "cost", at, shape::number)?;
        Ok(Self {
            billing,
            allowance,
            cost,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("billing", Value::from(self.billing.as_str()));
        if let Some(held) = &self.allowance {
            object.insert("allowance", Value::Number(*held));
        }
        if let Some(held) = &self.cost {
            object.insert("cost", Value::Number(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SkipReason {
    KeyMissing,
    KeyRejected,
    Credit,
    Allowance,
    Model,
    Overloaded,
    Unavailable,
    Network,
    Paused,
}

impl SkipReason {
    pub const ALL: [Self; 9] = [
        Self::KeyMissing,
        Self::KeyRejected,
        Self::Credit,
        Self::Allowance,
        Self::Model,
        Self::Overloaded,
        Self::Unavailable,
        Self::Network,
        Self::Paused,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::KeyMissing => "keyMissing",
            Self::KeyRejected => "keyRejected",
            Self::Credit => "credit",
            Self::Allowance => "allowance",
            Self::Model => "model",
            Self::Overloaded => "overloaded",
            Self::Unavailable => "unavailable",
            Self::Network => "network",
            Self::Paused => "paused",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for SkipReason {
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
pub struct Skip {
    pub entry: String,
    pub label: String,
    pub reason: SkipReason,
}

impl Shape for Skip {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let entry = shape::required(object, "entry", at, shape::text)?;
        let label = shape::required(object, "label", at, shape::text)?;
        let reason = shape::required(object, "reason", at, SkipReason::decode_at)?;
        Ok(Self {
            entry,
            label,
            reason,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("entry", Value::from(self.entry.as_str()));
        object.insert("label", Value::from(self.label.as_str()));
        object.insert("reason", self.reason.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Passed {
    pub entry: String,
    pub label: String,
    pub reason: SkipReason,
    pub before: f64,
}

impl Shape for Passed {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let entry = shape::required(object, "entry", at, shape::text)?;
        let label = shape::required(object, "label", at, shape::text)?;
        let reason = shape::required(object, "reason", at, SkipReason::decode_at)?;
        let before = shape::required(object, "before", at, shape::number)?;
        Ok(Self {
            entry,
            label,
            reason,
            before,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("entry", Value::from(self.entry.as_str()));
        object.insert("label", Value::from(self.label.as_str()));
        object.insert("reason", self.reason.encode());
        object.insert("before", Value::Number(self.before));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NoticeKind {
    KeyMissing,
    KeyRejected,
    SpendLimit,
    Refused,
    NoZdrHost,
    ModelRetired,
    ModelUnavailable,
    Unreachable,
    BadRequest,
    RetriesExhausted,
    ProviderOverloaded,
    StreamFailed,
    OutputCut,
    NoModel,
    HistoryDropped,
}

impl NoticeKind {
    pub const ALL: [Self; 15] = [
        Self::KeyMissing,
        Self::KeyRejected,
        Self::SpendLimit,
        Self::Refused,
        Self::NoZdrHost,
        Self::ModelRetired,
        Self::ModelUnavailable,
        Self::Unreachable,
        Self::BadRequest,
        Self::RetriesExhausted,
        Self::ProviderOverloaded,
        Self::StreamFailed,
        Self::OutputCut,
        Self::NoModel,
        Self::HistoryDropped,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::KeyMissing => "keyMissing",
            Self::KeyRejected => "keyRejected",
            Self::SpendLimit => "spendLimit",
            Self::Refused => "refused",
            Self::NoZdrHost => "noZdrHost",
            Self::ModelRetired => "modelRetired",
            Self::ModelUnavailable => "modelUnavailable",
            Self::Unreachable => "unreachable",
            Self::BadRequest => "badRequest",
            Self::RetriesExhausted => "retriesExhausted",
            Self::ProviderOverloaded => "providerOverloaded",
            Self::StreamFailed => "streamFailed",
            Self::OutputCut => "outputCut",
            Self::NoModel => "noModel",
            Self::HistoryDropped => "historyDropped",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for NoticeKind {
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
pub enum Disposition {
    Retry(DispositionRetry),
    FallThrough(DispositionFallThrough),
    Stop(DispositionStop),
}

impl Shape for Disposition {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        match shape::tag(object, "act", at)? {
            "retry" => DispositionRetry::decode_at(value, at).map(Self::Retry),
            "fallThrough" => DispositionFallThrough::decode_at(value, at).map(Self::FallThrough),
            "stop" => DispositionStop::decode_at(value, at).map(Self::Stop),
            _ => Err(shape::no_branch(at, "act")),
        }
    }

    fn encode(&self) -> Value {
        match self {
            Self::Retry(held) => held.encode(),
            Self::FallThrough(held) => held.encode(),
            Self::Stop(held) => held.encode(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DispositionRetry {}

impl Shape for DispositionRetry {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "act", at, |value, at| {
            shape::is_text(value, at, "retry")
        })?;
        Ok(Self {})
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("act", Value::from("retry"));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DispositionFallThrough {
    pub notice: NoticeKind,
}

impl Shape for DispositionFallThrough {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "act", at, |value, at| {
            shape::is_text(value, at, "fallThrough")
        })?;
        let notice = shape::required(object, "notice", at, NoticeKind::decode_at)?;
        Ok(Self { notice })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("act", Value::from("fallThrough"));
        object.insert("notice", self.notice.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DispositionStop {
    pub notice: NoticeKind,
}

impl Shape for DispositionStop {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "act", at, |value, at| {
            shape::is_text(value, at, "stop")
        })?;
        let notice = shape::required(object, "notice", at, NoticeKind::decode_at)?;
        Ok(Self { notice })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("act", Value::from("stop"));
        object.insert("notice", self.notice.encode());
        Value::Object(object)
    }
}

/// Decodes then encodes a named shape.
pub fn normalize(shape: &str, value: &Value) -> Option<Result<Value, Refused>> {
    let at = &mut Path::new();
    Some(match shape {
        "ProviderKindName" => ProviderKindName::decode_at(value, at).map(|held| held.encode()),
        "ProviderPlan" => ProviderPlan::decode_at(value, at).map(|held| held.encode()),
        "ProviderEntry" => ProviderEntry::decode_at(value, at).map(|held| held.encode()),
        "EntryFields" => EntryFields::decode_at(value, at).map(|held| held.encode()),
        "KeyState" => KeyState::decode_at(value, at).map(|held| held.encode()),
        "KindLine" => KindLine::decode_at(value, at).map(|held| held.encode()),
        "ProviderView" => ProviderView::decode_at(value, at).map(|held| held.encode()),
        "KeyReach" => KeyReach::decode_at(value, at).map(|held| held.encode()),
        "Limits" => Limits::decode_at(value, at).map(|held| held.encode()),
        "Billed" => Billed::decode_at(value, at).map(|held| held.encode()),
        "SkipReason" => SkipReason::decode_at(value, at).map(|held| held.encode()),
        "Skip" => Skip::decode_at(value, at).map(|held| held.encode()),
        "Passed" => Passed::decode_at(value, at).map(|held| held.encode()),
        "NoticeKind" => NoticeKind::decode_at(value, at).map(|held| held.encode()),
        "Disposition" => Disposition::decode_at(value, at).map(|held| held.encode()),
        _ => return None,
    })
}
