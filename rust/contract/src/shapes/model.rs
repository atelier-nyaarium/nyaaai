//! Generated from schema/model.output.json by scripts/contract.ts. Do not edit.

#![allow(
    clippy::enum_variant_names,
    clippy::large_enum_variant,
    clippy::ptr_arg
)]

use std::sync::LazyLock;

use crate::json::{Object, Record, Value};
use crate::pattern::Pattern;
use crate::shape::{self, Path, Refused, Shape};

use super::model_hooks as hooks;

static PATTERN_1: LazyLock<Option<Pattern>> =
    LazyLock::new(|| Pattern::compile("^[^\\u{0}-\\u{1f}\\u{7f}-\\u{9f}]+$", "").ok());
static PATTERN_2: LazyLock<Option<Pattern>> =
    LazyLock::new(|| Pattern::compile("^\\d{4}-\\d{2}-\\d{2}$", "").ok());

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ability {
    Schema,
}

impl Ability {
    pub const ALL: [Self; 1] = [Self::Schema];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Schema => "schema",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for Ability {
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
pub struct ModelUse {
    pub name: String,
    pub requires: Vec<Ability>,
    pub hides: bool,
}

impl Shape for ModelUse {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let name = shape::required(object, "name", at, |value, at| {
            shape::text_within(value, at, Some(1), Some(64))
                .and_then(|held| shape::matches(PATTERN_1.as_ref(), held, at, None))
        })?;
        let requires = shape::required(object, "requires", at, |value, at| {
            shape::list(value, at, Ability::decode_at)
                .and_then(|held| shape::holds(hooks::distinct_abilities(&held), held, at, None))
        })?;
        let hides = shape::required(object, "hides", at, shape::flag)?;
        Ok(Self {
            name,
            requires,
            hides,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("name", Value::from(self.name.as_str()));
        object.insert(
            "requires",
            Value::Array(self.requires.iter().map(|held| held.encode()).collect()),
        );
        object.insert("hides", Value::Bool(self.hides));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Binding {
    pub id: String,
}

impl Shape for Binding {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let id = shape::required(object, "id", at, shape::text)?;
        Ok(Self { id })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("id", Value::from(self.id.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RowIdentity {
    Curated(RowIdentityCurated),
    Hosted(RowIdentityHosted),
    Local(RowIdentityLocal),
}

impl Shape for RowIdentity {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        match shape::tag(object, "kind", at)? {
            "curated" => RowIdentityCurated::decode_at(value, at).map(Self::Curated),
            "hosted" => RowIdentityHosted::decode_at(value, at).map(Self::Hosted),
            "local" => RowIdentityLocal::decode_at(value, at).map(Self::Local),
            _ => Err(shape::no_branch(at, "kind")),
        }
    }

    fn encode(&self) -> Value {
        match self {
            Self::Curated(held) => held.encode(),
            Self::Hosted(held) => held.encode(),
            Self::Local(held) => held.encode(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RowIdentityCurated {
    pub key: String,
}

impl Shape for RowIdentityCurated {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "curated")
        })?;
        let key = shape::required(object, "key", at, shape::text)?;
        Ok(Self { key })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("curated"));
        object.insert("key", Value::from(self.key.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RowIdentityHosted {
    pub provider: crate::shapes::provider::ProviderKindName,
    pub id: String,
}

impl Shape for RowIdentityHosted {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "hosted")
        })?;
        let provider = shape::required(
            object,
            "provider",
            at,
            crate::shapes::provider::ProviderKindName::decode_at,
        )?;
        let id = shape::required(object, "id", at, shape::text)?;
        Ok(Self { provider, id })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("hosted"));
        object.insert("provider", self.provider.encode());
        object.insert("id", Value::from(self.id.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RowIdentityLocal {
    pub entry: String,
    pub tag: String,
}

impl Shape for RowIdentityLocal {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "local")
        })?;
        let entry = shape::required(object, "entry", at, shape::text)?;
        let tag = shape::required(object, "tag", at, shape::text)?;
        Ok(Self { entry, tag })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("local"));
        object.insert("entry", Value::from(self.entry.as_str()));
        object.insert("tag", Value::from(self.tag.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModelRow {
    pub key: String,
    pub bindings: Record<Binding>,
    pub family: String,
    pub label: String,
    pub reasoning: bool,
    pub schema: bool,
    pub cache: ModelRowCache,
    pub verified: String,
    pub verified_price: ModelRowVerifiedPrice,
    pub identity: RowIdentity,
}

impl Shape for ModelRow {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let key = shape::required(object, "key", at, shape::text)?;
        let bindings = shape::required(object, "bindings", at, |value, at| {
            shape::record(value, at, Binding::decode_at)
        })?;
        let family = shape::required(object, "family", at, shape::text)?;
        let label = shape::required(object, "label", at, shape::text)?;
        let reasoning = shape::required(object, "reasoning", at, shape::flag)?;
        let schema = shape::required(object, "schema", at, shape::flag)?;
        let cache = shape::required(object, "cache", at, ModelRowCache::decode_at)?;
        let verified = shape::required(object, "verified", at, shape::text)?;
        let verified_price = shape::required(
            object,
            "verifiedPrice",
            at,
            ModelRowVerifiedPrice::decode_at,
        )?;
        let identity = shape::required(object, "identity", at, RowIdentity::decode_at)?;
        Ok(Self {
            key,
            bindings,
            family,
            label,
            reasoning,
            schema,
            cache,
            verified,
            verified_price,
            identity,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("key", Value::from(self.key.as_str()));
        object.insert(
            "bindings",
            shape::record_value(&self.bindings, |held| held.encode()),
        );
        object.insert("family", Value::from(self.family.as_str()));
        object.insert("label", Value::from(self.label.as_str()));
        object.insert("reasoning", Value::Bool(self.reasoning));
        object.insert("schema", Value::Bool(self.schema));
        object.insert("cache", self.cache.encode());
        object.insert("verified", Value::from(self.verified.as_str()));
        object.insert("verifiedPrice", self.verified_price.encode());
        object.insert("identity", self.identity.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModelRowCache {
    Implicit,
    Explicit,
}

impl ModelRowCache {
    pub const ALL: [Self; 2] = [Self::Implicit, Self::Explicit];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Implicit => "implicit",
            Self::Explicit => "explicit",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for ModelRowCache {
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
pub struct ModelRowVerifiedPrice {
    pub prompt: f64,
    pub completion: f64,
}

impl Shape for ModelRowVerifiedPrice {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let prompt = shape::required(object, "prompt", at, shape::number)?;
        let completion = shape::required(object, "completion", at, shape::number)?;
        Ok(Self { prompt, completion })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("prompt", Value::Number(self.prompt));
        object.insert("completion", Value::Number(self.completion));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PriceTier {
    pub min_prompt_tokens: f64,
    pub prompt: f64,
    pub completion: f64,
    pub cache_read: Option<f64>,
    pub cache_write: Option<f64>,
    pub cache_write1h: Option<f64>,
}

impl Shape for PriceTier {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let min_prompt_tokens = shape::required(object, "minPromptTokens", at, shape::number)?;
        let prompt = shape::required(object, "prompt", at, shape::number)?;
        let completion = shape::required(object, "completion", at, shape::number)?;
        let cache_read = shape::optional(object, "cacheRead", at, shape::number)?;
        let cache_write = shape::optional(object, "cacheWrite", at, shape::number)?;
        let cache_write1h = shape::optional(object, "cacheWrite1h", at, shape::number)?;
        Ok(Self {
            min_prompt_tokens,
            prompt,
            completion,
            cache_read,
            cache_write,
            cache_write1h,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("minPromptTokens", Value::Number(self.min_prompt_tokens));
        object.insert("prompt", Value::Number(self.prompt));
        object.insert("completion", Value::Number(self.completion));
        if let Some(held) = &self.cache_read {
            object.insert("cacheRead", Value::Number(*held));
        }
        if let Some(held) = &self.cache_write {
            object.insert("cacheWrite", Value::Number(*held));
        }
        if let Some(held) = &self.cache_write1h {
            object.insert("cacheWrite1h", Value::Number(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Price {
    pub prompt: (f64, f64),
    pub completion: (f64, f64),
    pub tiers: Vec<PriceTier>,
}

impl Shape for Price {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let prompt = shape::required(object, "prompt", at, |value, at| {
            shape::tuple(value, at, 2).and_then(|items| {
                Ok((
                    shape::place(items, 0, at, shape::number)?,
                    shape::place(items, 1, at, shape::number)?,
                ))
            })
        })?;
        let completion = shape::required(object, "completion", at, |value, at| {
            shape::tuple(value, at, 2).and_then(|items| {
                Ok((
                    shape::place(items, 0, at, shape::number)?,
                    shape::place(items, 1, at, shape::number)?,
                ))
            })
        })?;
        let tiers = shape::required(object, "tiers", at, |value, at| {
            shape::list(value, at, PriceTier::decode_at)
        })?;
        Ok(Self {
            prompt,
            completion,
            tiers,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "prompt",
            Value::Array(vec![
                Value::Number(self.prompt.0),
                Value::Number(self.prompt.1),
            ]),
        );
        object.insert(
            "completion",
            Value::Array(vec![
                Value::Number(self.completion.0),
                Value::Number(self.completion.1),
            ]),
        );
        object.insert(
            "tiers",
            Value::Array(self.tiers.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModelView {
    pub row: ModelRow,
    pub status: ModelViewStatus,
    pub retired_on: Option<String>,
    pub efforts: Vec<crate::shapes::step::Effort>,
    pub prompt_limit: Option<f64>,
    pub output_limit: Option<f64>,
    pub price: Option<ModelViewPrice>,
    pub cache_write1h: Option<()>,
    pub hosts: Vec<String>,
    pub data_kept: Option<()>,
    pub api: Option<crate::shapes::step::ApiFacts>,
    pub cache: Option<ModelViewCache>,
}

impl Shape for ModelView {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let row = shape::required(object, "row", at, ModelRow::decode_at)?;
        let status = shape::required(object, "status", at, ModelViewStatus::decode_at)?;
        let retired_on = shape::optional(object, "retiredOn", at, shape::text)?;
        let efforts = shape::required(object, "efforts", at, |value, at| {
            shape::list(value, at, crate::shapes::step::Effort::decode_at)
        })?;
        let prompt_limit = shape::optional(object, "promptLimit", at, shape::number)?;
        let output_limit = shape::optional(object, "outputLimit", at, shape::number)?;
        let price = shape::optional(object, "price", at, ModelViewPrice::decode_at)?;
        let cache_write1h = shape::optional(object, "cacheWrite1h", at, |value, at| {
            shape::is_flag(value, at, true)
        })?;
        let hosts = shape::required(object, "hosts", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let data_kept = shape::optional(object, "dataKept", at, |value, at| {
            shape::is_flag(value, at, true)
        })?;
        let api = shape::optional(object, "api", at, crate::shapes::step::ApiFacts::decode_at)?;
        let cache = shape::optional(object, "cache", at, ModelViewCache::decode_at)?;
        Ok(Self {
            row,
            status,
            retired_on,
            efforts,
            prompt_limit,
            output_limit,
            price,
            cache_write1h,
            hosts,
            data_kept,
            api,
            cache,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("row", self.row.encode());
        object.insert("status", self.status.encode());
        if let Some(held) = &self.retired_on {
            object.insert("retiredOn", Value::from(held.as_str()));
        }
        object.insert(
            "efforts",
            Value::Array(self.efforts.iter().map(|held| held.encode()).collect()),
        );
        if let Some(held) = &self.prompt_limit {
            object.insert("promptLimit", Value::Number(*held));
        }
        if let Some(held) = &self.output_limit {
            object.insert("outputLimit", Value::Number(*held));
        }
        if let Some(held) = &self.price {
            object.insert("price", held.encode());
        }
        if self.cache_write1h.is_some() {
            object.insert("cacheWrite1h", Value::Bool(true));
        }
        object.insert(
            "hosts",
            Value::Array(
                self.hosts
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        if self.data_kept.is_some() {
            object.insert("dataKept", Value::Bool(true));
        }
        if let Some(held) = &self.api {
            object.insert("api", held.encode());
        }
        if let Some(held) = &self.cache {
            object.insert("cache", held.encode());
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModelViewStatus {
    Ok,
    Retired,
    NoZdrHost,
    NoTools,
    Unknown,
}

impl ModelViewStatus {
    pub const ALL: [Self; 5] = [
        Self::Ok,
        Self::Retired,
        Self::NoZdrHost,
        Self::NoTools,
        Self::Unknown,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Retired => "retired",
            Self::NoZdrHost => "noZdrHost",
            Self::NoTools => "noTools",
            Self::Unknown => "unknown",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for ModelViewStatus {
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
pub enum ModelViewPrice {
    Price(Price),
    Free,
    Unknown,
}

impl Shape for ModelViewPrice {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        if let Ok(held) = Price::decode_at(value, at) {
            return Ok(Self::Price(held));
        }
        if shape::is_text(value, at, "free").is_ok() {
            return Ok(Self::Free);
        }
        if shape::is_text(value, at, "unknown").is_ok() {
            return Ok(Self::Unknown);
        }
        Err(shape::refused(at, "no branch matches"))
    }

    fn encode(&self) -> Value {
        match self {
            Self::Price(held) => held.encode(),
            Self::Free => Value::from("free"),
            Self::Unknown => Value::from("unknown"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModelViewCache {
    Implicit,
    Explicit,
}

impl ModelViewCache {
    pub const ALL: [Self; 2] = [Self::Implicit, Self::Explicit];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Implicit => "implicit",
            Self::Explicit => "explicit",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for ModelViewCache {
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
pub struct TestRecord {
    pub label: String,
    pub date: String,
    pub cost: f64,
    pub estimated: Option<bool>,
    pub checks: TestRecordChecks,
    pub reasoning: bool,
    pub cache: TestRecordCache,
    pub verified_price: TestRecordVerifiedPrice,
}

impl Shape for TestRecord {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let label = shape::required(object, "label", at, |value, at| {
            let value = &hooks::trimmed(value);
            shape::text_within(value, at, Some(1), Some(200))
        })?;
        let date = shape::required(object, "date", at, |value, at| {
            shape::text(value, at)
                .and_then(|held| shape::matches(PATTERN_2.as_ref(), held, at, None))
        })?;
        let cost = shape::required(object, "cost", at, |value, at| {
            shape::number(value, at).and_then(|held| shape::at_least(held, 0.0, at))
        })?;
        let estimated = shape::optional(object, "estimated", at, shape::flag)?;
        let checks = shape::required(object, "checks", at, TestRecordChecks::decode_at)?;
        let reasoning = shape::required(object, "reasoning", at, shape::flag)?;
        let cache = shape::required(object, "cache", at, TestRecordCache::decode_at)?;
        let verified_price = shape::required(
            object,
            "verifiedPrice",
            at,
            TestRecordVerifiedPrice::decode_at,
        )?;
        Ok(Self {
            label,
            date,
            cost,
            estimated,
            checks,
            reasoning,
            cache,
            verified_price,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("label", Value::from(self.label.as_str()));
        object.insert("date", Value::from(self.date.as_str()));
        object.insert("cost", Value::Number(self.cost));
        if let Some(held) = &self.estimated {
            object.insert("estimated", Value::Bool(*held));
        }
        object.insert("checks", self.checks.encode());
        object.insert("reasoning", Value::Bool(self.reasoning));
        object.insert("cache", self.cache.encode());
        object.insert("verifiedPrice", self.verified_price.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestRecordChecks {
    pub tool_plus: TestRecordChecksToolPlus,
    pub recovery: TestRecordChecksRecovery,
    pub reasoning: TestRecordChecksReasoning,
    pub schema: TestRecordChecksSchema,
}

impl Shape for TestRecordChecks {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let tool_plus =
            shape::required(object, "toolPlus", at, TestRecordChecksToolPlus::decode_at)?;
        let recovery =
            shape::required(object, "recovery", at, TestRecordChecksRecovery::decode_at)?;
        let reasoning = shape::required(
            object,
            "reasoning",
            at,
            TestRecordChecksReasoning::decode_at,
        )?;
        let schema = shape::required(object, "schema", at, TestRecordChecksSchema::decode_at)?;
        Ok(Self {
            tool_plus,
            recovery,
            reasoning,
            schema,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("toolPlus", self.tool_plus.encode());
        object.insert("recovery", self.recovery.encode());
        object.insert("reasoning", self.reasoning.encode());
        object.insert("schema", self.schema.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TestRecordCache {
    Implicit,
    Explicit,
    None,
}

impl TestRecordCache {
    pub const ALL: [Self; 3] = [Self::Implicit, Self::Explicit, Self::None];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Implicit => "implicit",
            Self::Explicit => "explicit",
            Self::None => "none",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for TestRecordCache {
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
pub struct TestRecordVerifiedPrice {
    pub prompt: f64,
    pub completion: f64,
}

impl Shape for TestRecordVerifiedPrice {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let prompt = shape::required(object, "prompt", at, |value, at| {
            shape::number(value, at).and_then(|held| shape::at_least(held, 0.0, at))
        })?;
        let completion = shape::required(object, "completion", at, |value, at| {
            shape::number(value, at).and_then(|held| shape::at_least(held, 0.0, at))
        })?;
        Ok(Self { prompt, completion })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("prompt", Value::Number(self.prompt));
        object.insert("completion", Value::Number(self.completion));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TestRecordChecksToolPlus {
    Pass,
    Fail,
    NA,
}

impl TestRecordChecksToolPlus {
    pub const ALL: [Self; 3] = [Self::Pass, Self::Fail, Self::NA];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Fail => "fail",
            Self::NA => "n/a",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for TestRecordChecksToolPlus {
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
pub enum TestRecordChecksRecovery {
    Pass,
    Fail,
    NA,
}

impl TestRecordChecksRecovery {
    pub const ALL: [Self; 3] = [Self::Pass, Self::Fail, Self::NA];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Fail => "fail",
            Self::NA => "n/a",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for TestRecordChecksRecovery {
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
pub enum TestRecordChecksReasoning {
    Pass,
    Fail,
    NA,
}

impl TestRecordChecksReasoning {
    pub const ALL: [Self; 3] = [Self::Pass, Self::Fail, Self::NA];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Fail => "fail",
            Self::NA => "n/a",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for TestRecordChecksReasoning {
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
pub enum TestRecordChecksSchema {
    Pass,
    Fail,
    NA,
}

impl TestRecordChecksSchema {
    pub const ALL: [Self; 3] = [Self::Pass, Self::Fail, Self::NA];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Fail => "fail",
            Self::NA => "n/a",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for TestRecordChecksSchema {
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
pub struct StoredRegistry {
    pub tested: Record<Record<TestRecord>>,
    pub added: Record<Vec<String>>,
    pub hidden: Vec<String>,
}

impl Shape for StoredRegistry {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let tested = shape::required(object, "tested", at, |value, at| {
            shape::record(value, at, |value, at| {
                shape::record(value, at, TestRecord::decode_at)
            })
        })?;
        let added = shape::required(object, "added", at, |value, at| {
            shape::record(value, at, |value, at| shape::list(value, at, shape::text))
        })?;
        let hidden = shape::required(object, "hidden", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self {
            tested,
            added,
            hidden,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "tested",
            shape::record_value(&self.tested, |held| {
                shape::record_value(held, |held| held.encode())
            }),
        );
        object.insert(
            "added",
            shape::record_value(&self.added, |held| {
                Value::Array(held.iter().map(|held| Value::from(held.as_str())).collect())
            }),
        );
        object.insert(
            "hidden",
            Value::Array(
                self.hidden
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProviderRow {
    pub name: String,
    pub tag: String,
    pub context: Option<f64>,
    pub max_output: Option<f64>,
    pub quantization: Option<String>,
    pub prompt: Option<f64>,
    pub completion: Option<f64>,
    pub cache_read: Option<f64>,
    pub uptime: Option<f64>,
    pub zdr: bool,
}

impl Shape for ProviderRow {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let name = shape::required(object, "name", at, shape::text)?;
        let tag = shape::required(object, "tag", at, shape::text)?;
        let context = shape::optional(object, "context", at, shape::number)?;
        let max_output = shape::optional(object, "maxOutput", at, shape::number)?;
        let quantization = shape::optional(object, "quantization", at, shape::text)?;
        let prompt = shape::optional(object, "prompt", at, shape::number)?;
        let completion = shape::optional(object, "completion", at, shape::number)?;
        let cache_read = shape::optional(object, "cacheRead", at, shape::number)?;
        let uptime = shape::optional(object, "uptime", at, shape::number)?;
        let zdr = shape::required(object, "zdr", at, shape::flag)?;
        Ok(Self {
            name,
            tag,
            context,
            max_output,
            quantization,
            prompt,
            completion,
            cache_read,
            uptime,
            zdr,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("name", Value::from(self.name.as_str()));
        object.insert("tag", Value::from(self.tag.as_str()));
        if let Some(held) = &self.context {
            object.insert("context", Value::Number(*held));
        }
        if let Some(held) = &self.max_output {
            object.insert("maxOutput", Value::Number(*held));
        }
        if let Some(held) = &self.quantization {
            object.insert("quantization", Value::from(held.as_str()));
        }
        if let Some(held) = &self.prompt {
            object.insert("prompt", Value::Number(*held));
        }
        if let Some(held) = &self.completion {
            object.insert("completion", Value::Number(*held));
        }
        if let Some(held) = &self.cache_read {
            object.insert("cacheRead", Value::Number(*held));
        }
        if let Some(held) = &self.uptime {
            object.insert("uptime", Value::Number(*held));
        }
        object.insert("zdr", Value::Bool(self.zdr));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BrowseModel {
    pub id: String,
    pub display: String,
    pub label: String,
    pub curated: bool,
    pub description: String,
    pub context: Option<f64>,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub parameters: Vec<String>,
    pub price: BrowseModelPrice,
    pub tools: BrowseModelTools,
    pub retestable: Option<()>,
    pub family: Option<crate::shapes::step::ApiFamily>,
}

impl Shape for BrowseModel {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let id = shape::required(object, "id", at, shape::text)?;
        let display = shape::required(object, "display", at, shape::text)?;
        let label = shape::required(object, "label", at, shape::text)?;
        let curated = shape::required(object, "curated", at, shape::flag)?;
        let description = shape::required(object, "description", at, shape::text)?;
        let context = shape::optional(object, "context", at, shape::number)?;
        let inputs = shape::required(object, "inputs", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let outputs = shape::required(object, "outputs", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let parameters = shape::required(object, "parameters", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let price = shape::required(object, "price", at, BrowseModelPrice::decode_at)?;
        let tools = shape::required(object, "tools", at, BrowseModelTools::decode_at)?;
        let retestable = shape::optional(object, "retestable", at, |value, at| {
            shape::is_flag(value, at, true)
        })?;
        let family = shape::optional(
            object,
            "family",
            at,
            crate::shapes::step::ApiFamily::decode_at,
        )?;
        Ok(Self {
            id,
            display,
            label,
            curated,
            description,
            context,
            inputs,
            outputs,
            parameters,
            price,
            tools,
            retestable,
            family,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("id", Value::from(self.id.as_str()));
        object.insert("display", Value::from(self.display.as_str()));
        object.insert("label", Value::from(self.label.as_str()));
        object.insert("curated", Value::Bool(self.curated));
        object.insert("description", Value::from(self.description.as_str()));
        if let Some(held) = &self.context {
            object.insert("context", Value::Number(*held));
        }
        object.insert(
            "inputs",
            Value::Array(
                self.inputs
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        object.insert(
            "outputs",
            Value::Array(
                self.outputs
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        object.insert(
            "parameters",
            Value::Array(
                self.parameters
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        object.insert("price", self.price.encode());
        object.insert("tools", self.tools.encode());
        if self.retestable.is_some() {
            object.insert("retestable", Value::Bool(true));
        }
        if let Some(held) = &self.family {
            object.insert("family", held.encode());
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BrowseModelPrice {
    Object(BrowseModelPriceObject),
    Free,
    Unknown,
}

impl Shape for BrowseModelPrice {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        if let Ok(held) = BrowseModelPriceObject::decode_at(value, at) {
            return Ok(Self::Object(held));
        }
        if shape::is_text(value, at, "free").is_ok() {
            return Ok(Self::Free);
        }
        if shape::is_text(value, at, "unknown").is_ok() {
            return Ok(Self::Unknown);
        }
        Err(shape::refused(at, "no branch matches"))
    }

    fn encode(&self) -> Value {
        match self {
            Self::Object(held) => held.encode(),
            Self::Free => Value::from("free"),
            Self::Unknown => Value::from("unknown"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BrowseModelTools {
    Ok,
    None,
    NoZdrHost,
}

impl BrowseModelTools {
    pub const ALL: [Self; 3] = [Self::Ok, Self::None, Self::NoZdrHost];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::None => "none",
            Self::NoZdrHost => "noZdrHost",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for BrowseModelTools {
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
pub struct BrowseModelPriceObject {
    pub prompt: (f64, f64),
    pub completion: (f64, f64),
}

impl Shape for BrowseModelPriceObject {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let prompt = shape::required(object, "prompt", at, |value, at| {
            shape::tuple(value, at, 2).and_then(|items| {
                Ok((
                    shape::place(items, 0, at, shape::number)?,
                    shape::place(items, 1, at, shape::number)?,
                ))
            })
        })?;
        let completion = shape::required(object, "completion", at, |value, at| {
            shape::tuple(value, at, 2).and_then(|items| {
                Ok((
                    shape::place(items, 0, at, shape::number)?,
                    shape::place(items, 1, at, shape::number)?,
                ))
            })
        })?;
        Ok(Self { prompt, completion })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "prompt",
            Value::Array(vec![
                Value::Number(self.prompt.0),
                Value::Number(self.prompt.1),
            ]),
        );
        object.insert(
            "completion",
            Value::Array(vec![
                Value::Number(self.completion.0),
                Value::Number(self.completion.1),
            ]),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BrowseDetail {
    pub hosts: Option<Vec<ProviderRow>>,
    pub max_output: Option<f64>,
    pub efforts: Option<Vec<String>>,
    pub choice: Option<Vec<String>>,
    pub cache: Option<BrowseDetailCache>,
    pub retention: Option<BrowseDetailRetention>,
}

impl Shape for BrowseDetail {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let hosts = shape::optional(object, "hosts", at, |value, at| {
            shape::list(value, at, ProviderRow::decode_at)
        })?;
        let max_output = shape::optional(object, "maxOutput", at, shape::number)?;
        let efforts = shape::optional(object, "efforts", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let choice = shape::optional(object, "choice", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let cache = shape::optional(object, "cache", at, BrowseDetailCache::decode_at)?;
        let retention = shape::optional(object, "retention", at, BrowseDetailRetention::decode_at)?;
        Ok(Self {
            hosts,
            max_output,
            efforts,
            choice,
            cache,
            retention,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        if let Some(held) = &self.hosts {
            object.insert(
                "hosts",
                Value::Array(held.iter().map(|held| held.encode()).collect()),
            );
        }
        if let Some(held) = &self.max_output {
            object.insert("maxOutput", Value::Number(*held));
        }
        if let Some(held) = &self.efforts {
            object.insert(
                "efforts",
                Value::Array(held.iter().map(|held| Value::from(held.as_str())).collect()),
            );
        }
        if let Some(held) = &self.choice {
            object.insert(
                "choice",
                Value::Array(held.iter().map(|held| Value::from(held.as_str())).collect()),
            );
        }
        if let Some(held) = &self.cache {
            object.insert("cache", held.encode());
        }
        if let Some(held) = &self.retention {
            object.insert("retention", held.encode());
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BrowseDetailCache {
    Implicit,
    Explicit,
}

impl BrowseDetailCache {
    pub const ALL: [Self; 2] = [Self::Implicit, Self::Explicit];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Implicit => "implicit",
            Self::Explicit => "explicit",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for BrowseDetailCache {
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
pub struct BrowseDetailRetention {
    pub zdr: bool,
    pub until: Option<String>,
}

impl Shape for BrowseDetailRetention {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let zdr = shape::required(object, "zdr", at, shape::flag)?;
        let until = shape::optional(object, "until", at, shape::text)?;
        Ok(Self { zdr, until })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("zdr", Value::Bool(self.zdr));
        if let Some(held) = &self.until {
            object.insert("until", Value::from(held.as_str()));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Via {
    pub entry: String,
    pub label: String,
}

impl Shape for Via {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let entry = shape::required(object, "entry", at, shape::text)?;
        let label = shape::required(object, "label", at, shape::text)?;
        Ok(Self { entry, label })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("entry", Value::from(self.entry.as_str()));
        object.insert("label", Value::from(self.label.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Refusal {
    KeyMissing,
    KeyRejected,
    Unreachable,
    NoZdrHost,
    ModelRetired,
    NoModel,
}

impl Refusal {
    pub const ALL: [Self; 6] = [
        Self::KeyMissing,
        Self::KeyRejected,
        Self::Unreachable,
        Self::NoZdrHost,
        Self::ModelRetired,
        Self::NoModel,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::KeyMissing => "keyMissing",
            Self::KeyRejected => "keyRejected",
            Self::Unreachable => "unreachable",
            Self::NoZdrHost => "noZdrHost",
            Self::ModelRetired => "modelRetired",
            Self::NoModel => "noModel",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for Refusal {
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
pub struct Unresolved {
    pub key: String,
    pub refusal: Refusal,
    pub entry: Option<String>,
}

impl Shape for Unresolved {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "ok", at, |value, at| {
            shape::is_flag(value, at, false)
        })?;
        let key = shape::required(object, "key", at, shape::text)?;
        let refusal = shape::required(object, "refusal", at, Refusal::decode_at)?;
        let entry = shape::optional(object, "entry", at, shape::text)?;
        Ok(Self {
            key,
            refusal,
            entry,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("ok", Value::Bool(false));
        object.insert("key", Value::from(self.key.as_str()));
        object.insert("refusal", self.refusal.encode());
        if let Some(held) = &self.entry {
            object.insert("entry", Value::from(held.as_str()));
        }
        Value::Object(object)
    }
}

/// Decodes then encodes a named shape.
pub fn normalize(shape: &str, value: &Value) -> Option<Result<Value, Refused>> {
    let at = &mut Path::new();
    Some(match shape {
        "Ability" => Ability::decode_at(value, at).map(|held| held.encode()),
        "ModelUse" => ModelUse::decode_at(value, at).map(|held| held.encode()),
        "Binding" => Binding::decode_at(value, at).map(|held| held.encode()),
        "RowIdentity" => RowIdentity::decode_at(value, at).map(|held| held.encode()),
        "ModelRow" => ModelRow::decode_at(value, at).map(|held| held.encode()),
        "PriceTier" => PriceTier::decode_at(value, at).map(|held| held.encode()),
        "Price" => Price::decode_at(value, at).map(|held| held.encode()),
        "ModelView" => ModelView::decode_at(value, at).map(|held| held.encode()),
        "TestRecord" => TestRecord::decode_at(value, at).map(|held| held.encode()),
        "StoredRegistry" => StoredRegistry::decode_at(value, at).map(|held| held.encode()),
        "ProviderRow" => ProviderRow::decode_at(value, at).map(|held| held.encode()),
        "BrowseModel" => BrowseModel::decode_at(value, at).map(|held| held.encode()),
        "BrowseDetail" => BrowseDetail::decode_at(value, at).map(|held| held.encode()),
        "Via" => Via::decode_at(value, at).map(|held| held.encode()),
        "Refusal" => Refusal::decode_at(value, at).map(|held| held.encode()),
        "Unresolved" => Unresolved::decode_at(value, at).map(|held| held.encode()),
        _ => return None,
    })
}
