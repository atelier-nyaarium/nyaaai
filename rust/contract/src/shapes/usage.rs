//! Generated from schema/usage.output.json by scripts/contract.ts. Do not edit.

#![allow(
    clippy::enum_variant_names,
    clippy::large_enum_variant,
    clippy::ptr_arg
)]

use crate::json::{Object, Value};
use crate::shape::{self, Path, Refused, Shape};

#[derive(Debug, Clone, PartialEq)]
pub struct Cost {
    pub usd: f64,
    pub estimated: bool,
    pub unpriced: Option<()>,
}

impl Shape for Cost {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let usd = shape::required(object, "usd", at, shape::number)?;
        let estimated = shape::required(object, "estimated", at, shape::flag)?;
        let unpriced = shape::optional(object, "unpriced", at, |value, at| {
            shape::is_flag(value, at, true)
        })?;
        Ok(Self {
            usd,
            estimated,
            unpriced,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("usd", Value::Number(self.usd));
        object.insert("estimated", Value::Bool(self.estimated));
        if self.unpriced.is_some() {
            object.insert("unpriced", Value::Bool(true));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CallTag {
    pub feature: String,
    pub lane: Option<String>,
    pub folder: Option<String>,
}

impl Shape for CallTag {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let feature = shape::required(object, "feature", at, shape::text)?;
        let lane = shape::optional(object, "lane", at, shape::text)?;
        let folder = shape::optional(object, "folder", at, shape::text)?;
        Ok(Self {
            feature,
            lane,
            folder,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("feature", Value::from(self.feature.as_str()));
        if let Some(held) = &self.lane {
            object.insert("lane", Value::from(held.as_str()));
        }
        if let Some(held) = &self.folder {
            object.insert("folder", Value::from(held.as_str()));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeteredCall {
    pub tag: CallTag,
    pub model: String,
    pub entry: Option<String>,
    pub label: Option<String>,
    pub at: f64,
    pub ms: f64,
    pub first_token_ms: Option<f64>,
    pub usage: Option<crate::shapes::step::NormalizedUsage>,
    pub cost: Cost,
    pub provider: Option<String>,
    pub generation_id: Option<String>,
    pub finish_reason: Option<String>,
    pub failure: Option<String>,
    pub cancelled: Option<bool>,
}

impl Shape for MeteredCall {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let tag = shape::required(object, "tag", at, CallTag::decode_at)?;
        let model = shape::required(object, "model", at, shape::text)?;
        let entry = shape::optional(object, "entry", at, shape::text)?;
        let label = shape::optional(object, "label", at, shape::text)?;
        let at_field = shape::required(object, "at", at, shape::number)?;
        let ms = shape::required(object, "ms", at, shape::number)?;
        let first_token_ms = shape::optional(object, "firstTokenMs", at, shape::number)?;
        let usage = shape::optional(
            object,
            "usage",
            at,
            crate::shapes::step::NormalizedUsage::decode_at,
        )?;
        let cost = shape::required(object, "cost", at, Cost::decode_at)?;
        let provider = shape::optional(object, "provider", at, shape::text)?;
        let generation_id = shape::optional(object, "generationId", at, shape::text)?;
        let finish_reason = shape::optional(object, "finishReason", at, shape::text)?;
        let failure = shape::optional(object, "failure", at, shape::text)?;
        let cancelled = shape::optional(object, "cancelled", at, shape::flag)?;
        Ok(Self {
            tag,
            model,
            entry,
            label,
            at: at_field,
            ms,
            first_token_ms,
            usage,
            cost,
            provider,
            generation_id,
            finish_reason,
            failure,
            cancelled,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("tag", self.tag.encode());
        object.insert("model", Value::from(self.model.as_str()));
        if let Some(held) = &self.entry {
            object.insert("entry", Value::from(held.as_str()));
        }
        if let Some(held) = &self.label {
            object.insert("label", Value::from(held.as_str()));
        }
        object.insert("at", Value::Number(self.at));
        object.insert("ms", Value::Number(self.ms));
        if let Some(held) = &self.first_token_ms {
            object.insert("firstTokenMs", Value::Number(*held));
        }
        if let Some(held) = &self.usage {
            object.insert("usage", held.encode());
        }
        object.insert("cost", self.cost.encode());
        if let Some(held) = &self.provider {
            object.insert("provider", Value::from(held.as_str()));
        }
        if let Some(held) = &self.generation_id {
            object.insert("generationId", Value::from(held.as_str()));
        }
        if let Some(held) = &self.finish_reason {
            object.insert("finishReason", Value::from(held.as_str()));
        }
        if let Some(held) = &self.failure {
            object.insert("failure", Value::from(held.as_str()));
        }
        if let Some(held) = &self.cancelled {
            object.insert("cancelled", Value::Bool(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CallRecord {
    pub id: String,
    pub at: f64,
    pub feature: String,
    pub lane: Option<String>,
    pub model: String,
    pub entry: Option<String>,
    pub label: Option<String>,
    pub app: Option<String>,
    pub ms: f64,
    pub first: Option<f64>,
    pub in_: Option<f64>,
    pub out: Option<f64>,
    pub cached: Option<f64>,
    pub cache_write: Option<f64>,
    pub reasoning: Option<f64>,
    pub cost: f64,
    pub account_cost: Option<f64>,
    pub upstream_cost: Option<f64>,
    pub byok: Option<bool>,
    pub billing_source: Option<String>,
    pub estimated: Option<bool>,
    pub unpriced: Option<()>,
    pub provider: Option<String>,
    pub gen_: Option<String>,
    pub finish: Option<String>,
    pub failure: Option<String>,
    pub cancelled: Option<bool>,
}

impl Shape for CallRecord {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let id = shape::required(object, "id", at, shape::text)?;
        let at_field = shape::required(object, "at", at, shape::number)?;
        let feature = shape::required(object, "feature", at, |value, at| {
            shape::text_within(value, at, Some(1), None)
        })?;
        let lane = shape::optional(object, "lane", at, shape::text)?;
        let model = shape::required(object, "model", at, shape::text)?;
        let entry = shape::optional(object, "entry", at, shape::text)?;
        let label = shape::optional(object, "label", at, shape::text)?;
        let app = shape::optional(object, "app", at, shape::text)?;
        let ms = shape::required(object, "ms", at, shape::number)?;
        let first = shape::optional(object, "first", at, shape::number)?;
        let in_ = shape::optional(object, "in", at, shape::number)?;
        let out = shape::optional(object, "out", at, shape::number)?;
        let cached = shape::optional(object, "cached", at, shape::number)?;
        let cache_write = shape::optional(object, "cacheWrite", at, shape::number)?;
        let reasoning = shape::optional(object, "reasoning", at, shape::number)?;
        let cost = shape::required(object, "cost", at, shape::number)?;
        let account_cost = shape::optional(object, "accountCost", at, shape::number)?;
        let upstream_cost = shape::optional(object, "upstreamCost", at, shape::number)?;
        let byok = shape::optional(object, "byok", at, shape::flag)?;
        let billing_source = shape::optional(object, "billingSource", at, shape::text)?;
        let estimated = shape::optional(object, "estimated", at, shape::flag)?;
        let unpriced = shape::optional(object, "unpriced", at, |value, at| {
            shape::is_flag(value, at, true)
        })?;
        let provider = shape::optional(object, "provider", at, shape::text)?;
        let gen_ = shape::optional(object, "gen", at, shape::text)?;
        let finish = shape::optional(object, "finish", at, shape::text)?;
        let failure = shape::optional(object, "failure", at, shape::text)?;
        let cancelled = shape::optional(object, "cancelled", at, shape::flag)?;
        Ok(Self {
            id,
            at: at_field,
            feature,
            lane,
            model,
            entry,
            label,
            app,
            ms,
            first,
            in_,
            out,
            cached,
            cache_write,
            reasoning,
            cost,
            account_cost,
            upstream_cost,
            byok,
            billing_source,
            estimated,
            unpriced,
            provider,
            gen_,
            finish,
            failure,
            cancelled,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("id", Value::from(self.id.as_str()));
        object.insert("at", Value::Number(self.at));
        object.insert("feature", Value::from(self.feature.as_str()));
        if let Some(held) = &self.lane {
            object.insert("lane", Value::from(held.as_str()));
        }
        object.insert("model", Value::from(self.model.as_str()));
        if let Some(held) = &self.entry {
            object.insert("entry", Value::from(held.as_str()));
        }
        if let Some(held) = &self.label {
            object.insert("label", Value::from(held.as_str()));
        }
        if let Some(held) = &self.app {
            object.insert("app", Value::from(held.as_str()));
        }
        object.insert("ms", Value::Number(self.ms));
        if let Some(held) = &self.first {
            object.insert("first", Value::Number(*held));
        }
        if let Some(held) = &self.in_ {
            object.insert("in", Value::Number(*held));
        }
        if let Some(held) = &self.out {
            object.insert("out", Value::Number(*held));
        }
        if let Some(held) = &self.cached {
            object.insert("cached", Value::Number(*held));
        }
        if let Some(held) = &self.cache_write {
            object.insert("cacheWrite", Value::Number(*held));
        }
        if let Some(held) = &self.reasoning {
            object.insert("reasoning", Value::Number(*held));
        }
        object.insert("cost", Value::Number(self.cost));
        if let Some(held) = &self.account_cost {
            object.insert("accountCost", Value::Number(*held));
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
        if let Some(held) = &self.estimated {
            object.insert("estimated", Value::Bool(*held));
        }
        if self.unpriced.is_some() {
            object.insert("unpriced", Value::Bool(true));
        }
        if let Some(held) = &self.provider {
            object.insert("provider", Value::from(held.as_str()));
        }
        if let Some(held) = &self.gen_ {
            object.insert("gen", Value::from(held.as_str()));
        }
        if let Some(held) = &self.finish {
            object.insert("finish", Value::from(held.as_str()));
        }
        if let Some(held) = &self.failure {
            object.insert("failure", Value::from(held.as_str()));
        }
        if let Some(held) = &self.cancelled {
            object.insert("cancelled", Value::Bool(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GenerationStats {
    pub provider: Option<String>,
    pub latency_ms: Option<f64>,
    pub generation_ms: Option<f64>,
    pub cost: Option<f64>,
    pub account_cost: Option<f64>,
    pub upstream_cost: Option<f64>,
    pub byok: Option<bool>,
    pub in_: Option<f64>,
    pub out: Option<f64>,
    pub finish: Option<String>,
}

impl Shape for GenerationStats {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let provider = shape::optional(object, "provider", at, shape::text)?;
        let latency_ms = shape::optional(object, "latencyMs", at, shape::number)?;
        let generation_ms = shape::optional(object, "generationMs", at, shape::number)?;
        let cost = shape::optional(object, "cost", at, shape::number)?;
        let account_cost = shape::optional(object, "accountCost", at, shape::number)?;
        let upstream_cost = shape::optional(object, "upstreamCost", at, shape::number)?;
        let byok = shape::optional(object, "byok", at, shape::flag)?;
        let in_ = shape::optional(object, "in", at, shape::number)?;
        let out = shape::optional(object, "out", at, shape::number)?;
        let finish = shape::optional(object, "finish", at, shape::text)?;
        Ok(Self {
            provider,
            latency_ms,
            generation_ms,
            cost,
            account_cost,
            upstream_cost,
            byok,
            in_,
            out,
            finish,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        if let Some(held) = &self.provider {
            object.insert("provider", Value::from(held.as_str()));
        }
        if let Some(held) = &self.latency_ms {
            object.insert("latencyMs", Value::Number(*held));
        }
        if let Some(held) = &self.generation_ms {
            object.insert("generationMs", Value::Number(*held));
        }
        if let Some(held) = &self.cost {
            object.insert("cost", Value::Number(*held));
        }
        if let Some(held) = &self.account_cost {
            object.insert("accountCost", Value::Number(*held));
        }
        if let Some(held) = &self.upstream_cost {
            object.insert("upstreamCost", Value::Number(*held));
        }
        if let Some(held) = &self.byok {
            object.insert("byok", Value::Bool(*held));
        }
        if let Some(held) = &self.in_ {
            object.insert("in", Value::Number(*held));
        }
        if let Some(held) = &self.out {
            object.insert("out", Value::Number(*held));
        }
        if let Some(held) = &self.finish {
            object.insert("finish", Value::from(held.as_str()));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CallLine {
    pub call: CallRecord,
}

impl Shape for CallLine {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let call = shape::required(object, "call", at, CallRecord::decode_at)?;
        Ok(Self { call })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("call", self.call.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StatsRecord {
    pub id: String,
    pub of: GenerationStats,
}

impl Shape for StatsRecord {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let id = shape::required(object, "id", at, shape::text)?;
        let of = shape::required(object, "of", at, GenerationStats::decode_at)?;
        Ok(Self { id, of })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("id", Value::from(self.id.as_str()));
        object.insert("of", self.of.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StatsLine {
    pub stats: StatsRecord,
}

impl Shape for StatsLine {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let stats = shape::required(object, "stats", at, StatsRecord::decode_at)?;
        Ok(Self { stats })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("stats", self.stats.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum UsageLine {
    CallLine(CallLine),
    StatsLine(StatsLine),
}

impl Shape for UsageLine {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        if let Ok(held) = CallLine::decode_at(value, at) {
            return Ok(Self::CallLine(held));
        }
        if let Ok(held) = StatsLine::decode_at(value, at) {
            return Ok(Self::StatsLine(held));
        }
        Err(shape::refused(at, "no branch matches"))
    }

    fn encode(&self) -> Value {
        match self {
            Self::CallLine(held) => held.encode(),
            Self::StatsLine(held) => held.encode(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnfilledRecord {
    pub id: String,
    pub generation_id: String,
    pub at: f64,
    pub entry: Option<String>,
}

impl Shape for UnfilledRecord {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let id = shape::required(object, "id", at, shape::text)?;
        let generation_id = shape::required(object, "generationId", at, shape::text)?;
        let at_field = shape::required(object, "at", at, shape::number)?;
        let entry = shape::optional(object, "entry", at, shape::text)?;
        Ok(Self {
            id,
            generation_id,
            at: at_field,
            entry,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("id", Value::from(self.id.as_str()));
        object.insert("generationId", Value::from(self.generation_id.as_str()));
        object.insert("at", Value::Number(self.at));
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
        "Cost" => Cost::decode_at(value, at).map(|held| held.encode()),
        "CallTag" => CallTag::decode_at(value, at).map(|held| held.encode()),
        "MeteredCall" => MeteredCall::decode_at(value, at).map(|held| held.encode()),
        "CallRecord" => CallRecord::decode_at(value, at).map(|held| held.encode()),
        "GenerationStats" => GenerationStats::decode_at(value, at).map(|held| held.encode()),
        "CallLine" => CallLine::decode_at(value, at).map(|held| held.encode()),
        "StatsRecord" => StatsRecord::decode_at(value, at).map(|held| held.encode()),
        "StatsLine" => StatsLine::decode_at(value, at).map(|held| held.encode()),
        "UsageLine" => UsageLine::decode_at(value, at).map(|held| held.encode()),
        "UnfilledRecord" => UnfilledRecord::decode_at(value, at).map(|held| held.encode()),
        _ => return None,
    })
}
