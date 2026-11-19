//! Generated from schema/lifecycle.output.json by scripts/contract.ts. Do not edit.

#![allow(
    clippy::enum_variant_names,
    clippy::large_enum_variant,
    clippy::ptr_arg
)]

use crate::json::{Object, Value};
use crate::shape::{self, Path, Refused, Shape};

#[derive(Debug, Clone, PartialEq)]
pub enum TraceEntry {
    Skip(TraceEntrySkip),
    Held(TraceEntryHeld),
    Released(TraceEntryReleased),
    Admitted(TraceEntryAdmitted),
    Kept(TraceEntryKept),
    Attempt(TraceEntryAttempt),
    Metered(TraceEntryMetered),
    Billing(TraceEntryBilling),
    Failure(TraceEntryFailure),
    Settled(TraceEntrySettled),
    GateFailed(TraceEntryGateFailed),
    Retry(TraceEntryRetry),
    Limits(TraceEntryLimits),
    Threw(TraceEntryThrew),
    Served(TraceEntryServed),
    Stopped(TraceEntryStopped),
    Notice(TraceEntryNotice),
}

impl Shape for TraceEntry {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        match shape::tag(object, "kind", at)? {
            "skip" => TraceEntrySkip::decode_at(value, at).map(Self::Skip),
            "held" => TraceEntryHeld::decode_at(value, at).map(Self::Held),
            "released" => TraceEntryReleased::decode_at(value, at).map(Self::Released),
            "admitted" => TraceEntryAdmitted::decode_at(value, at).map(Self::Admitted),
            "kept" => TraceEntryKept::decode_at(value, at).map(Self::Kept),
            "attempt" => TraceEntryAttempt::decode_at(value, at).map(Self::Attempt),
            "metered" => TraceEntryMetered::decode_at(value, at).map(Self::Metered),
            "billing" => TraceEntryBilling::decode_at(value, at).map(Self::Billing),
            "failure" => TraceEntryFailure::decode_at(value, at).map(Self::Failure),
            "settled" => TraceEntrySettled::decode_at(value, at).map(Self::Settled),
            "gateFailed" => TraceEntryGateFailed::decode_at(value, at).map(Self::GateFailed),
            "retry" => TraceEntryRetry::decode_at(value, at).map(Self::Retry),
            "limits" => TraceEntryLimits::decode_at(value, at).map(Self::Limits),
            "threw" => TraceEntryThrew::decode_at(value, at).map(Self::Threw),
            "served" => TraceEntryServed::decode_at(value, at).map(Self::Served),
            "stopped" => TraceEntryStopped::decode_at(value, at).map(Self::Stopped),
            "notice" => TraceEntryNotice::decode_at(value, at).map(Self::Notice),
            _ => Err(shape::no_branch(at, "kind")),
        }
    }

    fn encode(&self) -> Value {
        match self {
            Self::Skip(held) => held.encode(),
            Self::Held(held) => held.encode(),
            Self::Released(held) => held.encode(),
            Self::Admitted(held) => held.encode(),
            Self::Kept(held) => held.encode(),
            Self::Attempt(held) => held.encode(),
            Self::Metered(held) => held.encode(),
            Self::Billing(held) => held.encode(),
            Self::Failure(held) => held.encode(),
            Self::Settled(held) => held.encode(),
            Self::GateFailed(held) => held.encode(),
            Self::Retry(held) => held.encode(),
            Self::Limits(held) => held.encode(),
            Self::Threw(held) => held.encode(),
            Self::Served(held) => held.encode(),
            Self::Stopped(held) => held.encode(),
            Self::Notice(held) => held.encode(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraceEntrySkip {
    pub cause: TraceEntrySkipCause,
    pub entry: String,
    pub label: String,
    pub reason: crate::shapes::provider::SkipReason,
}

impl Shape for TraceEntrySkip {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "skip")
        })?;
        let cause = shape::required(object, "cause", at, TraceEntrySkipCause::decode_at)?;
        let entry = shape::required(object, "entry", at, shape::text)?;
        let label = shape::required(object, "label", at, shape::text)?;
        let reason = shape::required(
            object,
            "reason",
            at,
            crate::shapes::provider::SkipReason::decode_at,
        )?;
        Ok(Self {
            cause,
            entry,
            label,
            reason,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("skip"));
        object.insert("cause", self.cause.encode());
        object.insert("entry", Value::from(self.entry.as_str()));
        object.insert("label", Value::from(self.label.as_str()));
        object.insert("reason", self.reason.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraceEntryHeld {
    pub entry: String,
    pub waiting: crate::shapes::health::GateWaiting,
}

impl Shape for TraceEntryHeld {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "held")
        })?;
        let entry = shape::required(object, "entry", at, shape::text)?;
        let waiting = shape::required(
            object,
            "waiting",
            at,
            crate::shapes::health::GateWaiting::decode_at,
        )?;
        Ok(Self { entry, waiting })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("held"));
        object.insert("entry", Value::from(self.entry.as_str()));
        object.insert("waiting", self.waiting.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraceEntryReleased {
    pub entry: String,
}

impl Shape for TraceEntryReleased {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "released")
        })?;
        let entry = shape::required(object, "entry", at, shape::text)?;
        Ok(Self { entry })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("released"));
        object.insert("entry", Value::from(self.entry.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraceEntryAdmitted {
    pub entry: String,
    pub admission: crate::shapes::health::Admission,
}

impl Shape for TraceEntryAdmitted {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "admitted")
        })?;
        let entry = shape::required(object, "entry", at, shape::text)?;
        let admission = shape::required(
            object,
            "admission",
            at,
            crate::shapes::health::Admission::decode_at,
        )?;
        Ok(Self { entry, admission })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("admitted"));
        object.insert("entry", Value::from(self.entry.as_str()));
        object.insert("admission", self.admission.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraceEntryKept {
    pub entry: String,
    pub model: String,
}

impl Shape for TraceEntryKept {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "kept")
        })?;
        let entry = shape::required(object, "entry", at, shape::text)?;
        let model = shape::required(object, "model", at, shape::text)?;
        Ok(Self { entry, model })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("kept"));
        object.insert("entry", Value::from(self.entry.as_str()));
        object.insert("model", Value::from(self.model.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraceEntryAttempt {
    pub entry: String,
    pub model: String,
    pub attempt: u64,
    pub live: bool,
    pub again: bool,
}

impl Shape for TraceEntryAttempt {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "attempt")
        })?;
        let entry = shape::required(object, "entry", at, shape::text)?;
        let model = shape::required(object, "model", at, shape::text)?;
        let attempt = shape::required(object, "attempt", at, |value, at| {
            shape::natural(value, at).and_then(|held| shape::at_least(held, 1, at))
        })?;
        let live = shape::required(object, "live", at, shape::flag)?;
        let again = shape::required(object, "again", at, shape::flag)?;
        Ok(Self {
            entry,
            model,
            attempt,
            live,
            again,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("attempt"));
        object.insert("entry", Value::from(self.entry.as_str()));
        object.insert("model", Value::from(self.model.as_str()));
        object.insert("attempt", Value::Number(self.attempt as f64));
        object.insert("live", Value::Bool(self.live));
        object.insert("again", Value::Bool(self.again));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraceEntryMetered {
    pub call: crate::shapes::usage::MeteredCall,
}

impl Shape for TraceEntryMetered {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "metered")
        })?;
        let call = shape::required(
            object,
            "call",
            at,
            crate::shapes::usage::MeteredCall::decode_at,
        )?;
        Ok(Self { call })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("metered"));
        object.insert("call", self.call.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraceEntryBilling {
    pub entry: String,
    pub reply: TraceEntryBillingReply,
    pub usage: Option<crate::shapes::step::NormalizedUsage>,
    pub dropped: Option<bool>,
}

impl Shape for TraceEntryBilling {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "billing")
        })?;
        let entry = shape::required(object, "entry", at, shape::text)?;
        let reply = shape::required(object, "reply", at, TraceEntryBillingReply::decode_at)?;
        let usage = shape::optional(
            object,
            "usage",
            at,
            crate::shapes::step::NormalizedUsage::decode_at,
        )?;
        let dropped = shape::optional(object, "dropped", at, shape::flag)?;
        Ok(Self {
            entry,
            reply,
            usage,
            dropped,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("billing"));
        object.insert("entry", Value::from(self.entry.as_str()));
        object.insert("reply", self.reply.encode());
        if let Some(held) = &self.usage {
            object.insert("usage", held.encode());
        }
        if let Some(held) = &self.dropped {
            object.insert("dropped", Value::Bool(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraceEntryFailure {
    pub entry: String,
    pub model: String,
    pub failure: crate::shapes::step::Failure,
    pub disposition: Option<crate::shapes::provider::Disposition>,
    pub fresh: bool,
    pub onward: bool,
}

impl Shape for TraceEntryFailure {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "failure")
        })?;
        let entry = shape::required(object, "entry", at, shape::text)?;
        let model = shape::required(object, "model", at, shape::text)?;
        let failure = shape::required(
            object,
            "failure",
            at,
            crate::shapes::step::Failure::decode_at,
        )?;
        let disposition = shape::optional(
            object,
            "disposition",
            at,
            crate::shapes::provider::Disposition::decode_at,
        )?;
        let fresh = shape::required(object, "fresh", at, shape::flag)?;
        let onward = shape::required(object, "onward", at, shape::flag)?;
        Ok(Self {
            entry,
            model,
            failure,
            disposition,
            fresh,
            onward,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("failure"));
        object.insert("entry", Value::from(self.entry.as_str()));
        object.insert("model", Value::from(self.model.as_str()));
        object.insert("failure", self.failure.encode());
        if let Some(held) = &self.disposition {
            object.insert("disposition", held.encode());
        }
        object.insert("fresh", Value::Bool(self.fresh));
        object.insert("onward", Value::Bool(self.onward));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraceEntrySettled {
    pub entry: String,
    pub outcome: crate::shapes::health::GateOutcome,
    pub retry_after_ms: Option<i64>,
    pub answer: crate::shapes::health::GateSettled,
}

impl Shape for TraceEntrySettled {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "settled")
        })?;
        let entry = shape::required(object, "entry", at, shape::text)?;
        let outcome = shape::required(
            object,
            "outcome",
            at,
            crate::shapes::health::GateOutcome::decode_at,
        )?;
        let retry_after_ms = shape::optional(object, "retryAfterMs", at, shape::integer)?;
        let answer = shape::required(
            object,
            "answer",
            at,
            crate::shapes::health::GateSettled::decode_at,
        )?;
        Ok(Self {
            entry,
            outcome,
            retry_after_ms,
            answer,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("settled"));
        object.insert("entry", Value::from(self.entry.as_str()));
        object.insert("outcome", self.outcome.encode());
        if let Some(held) = &self.retry_after_ms {
            object.insert("retryAfterMs", Value::Number(*held as f64));
        }
        object.insert("answer", self.answer.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraceEntryGateFailed {
    pub entry: String,
    pub failure: crate::shapes::step::Failure,
    pub onward: bool,
    pub dropped: Option<bool>,
}

impl Shape for TraceEntryGateFailed {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "gateFailed")
        })?;
        let entry = shape::required(object, "entry", at, shape::text)?;
        let failure = shape::required(
            object,
            "failure",
            at,
            crate::shapes::step::Failure::decode_at,
        )?;
        let onward = shape::required(object, "onward", at, shape::flag)?;
        let dropped = shape::optional(object, "dropped", at, shape::flag)?;
        Ok(Self {
            entry,
            failure,
            onward,
            dropped,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("gateFailed"));
        object.insert("entry", Value::from(self.entry.as_str()));
        object.insert("failure", self.failure.encode());
        object.insert("onward", Value::Bool(self.onward));
        if let Some(held) = &self.dropped {
            object.insert("dropped", Value::Bool(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraceEntryRetry {
    pub entry: String,
    pub attempt: u64,
    pub wait_ms: u64,
    pub why: TraceEntryRetryWhy,
    pub status: crate::shapes::health::ErrorStatus,
}

impl Shape for TraceEntryRetry {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "retry")
        })?;
        let entry = shape::required(object, "entry", at, shape::text)?;
        let attempt = shape::required(object, "attempt", at, |value, at| {
            shape::natural(value, at).and_then(|held| shape::at_least(held, 1, at))
        })?;
        let wait_ms = shape::required(object, "waitMs", at, shape::natural)?;
        let why = shape::required(object, "why", at, TraceEntryRetryWhy::decode_at)?;
        let status = shape::required(
            object,
            "status",
            at,
            crate::shapes::health::ErrorStatus::decode_at,
        )?;
        Ok(Self {
            entry,
            attempt,
            wait_ms,
            why,
            status,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("retry"));
        object.insert("entry", Value::from(self.entry.as_str()));
        object.insert("attempt", Value::Number(self.attempt as f64));
        object.insert("waitMs", Value::Number(self.wait_ms as f64));
        object.insert("why", self.why.encode());
        object.insert("status", self.status.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraceEntryLimits {
    pub entry: String,
    pub read: bool,
    pub limit: Option<f64>,
    pub reset: Option<String>,
}

impl Shape for TraceEntryLimits {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "limits")
        })?;
        let entry = shape::required(object, "entry", at, shape::text)?;
        let read = shape::required(object, "read", at, shape::flag)?;
        let limit = shape::optional(object, "limit", at, shape::number)?;
        let reset = shape::optional(object, "reset", at, shape::text)?;
        Ok(Self {
            entry,
            read,
            limit,
            reset,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("limits"));
        object.insert("entry", Value::from(self.entry.as_str()));
        object.insert("read", Value::Bool(self.read));
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
pub struct TraceEntryThrew {
    pub entry: String,
    pub message: String,
}

impl Shape for TraceEntryThrew {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "threw")
        })?;
        let entry = shape::required(object, "entry", at, shape::text)?;
        let message = shape::required(object, "message", at, shape::text)?;
        Ok(Self { entry, message })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("threw"));
        object.insert("entry", Value::from(self.entry.as_str()));
        object.insert("message", Value::from(self.message.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraceEntryServed {
    pub entry: String,
    pub label: String,
    pub model: String,
}

impl Shape for TraceEntryServed {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "served")
        })?;
        let entry = shape::required(object, "entry", at, shape::text)?;
        let label = shape::required(object, "label", at, shape::text)?;
        let model = shape::required(object, "model", at, shape::text)?;
        Ok(Self {
            entry,
            label,
            model,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("served"));
        object.insert("entry", Value::from(self.entry.as_str()));
        object.insert("label", Value::from(self.label.as_str()));
        object.insert("model", Value::from(self.model.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraceEntryStopped {}

impl Shape for TraceEntryStopped {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "stopped")
        })?;
        Ok(Self {})
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("stopped"));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraceEntryNotice {
    pub notice: crate::shapes::health::Notice,
}

impl Shape for TraceEntryNotice {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "notice")
        })?;
        let notice = shape::required(
            object,
            "notice",
            at,
            crate::shapes::health::Notice::decode_at,
        )?;
        Ok(Self { notice })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("notice"));
        object.insert("notice", self.notice.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TraceEntrySkipCause {
    Passed,
    Deferred,
    FellThrough,
}

impl TraceEntrySkipCause {
    pub const ALL: [Self; 3] = [Self::Passed, Self::Deferred, Self::FellThrough];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Deferred => "deferred",
            Self::FellThrough => "fellThrough",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for TraceEntrySkipCause {
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
pub enum TraceEntryBillingReply {
    Ok,
    Failed,
}

impl TraceEntryBillingReply {
    pub const ALL: [Self; 2] = [Self::Ok, Self::Failed];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Failed => "failed",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for TraceEntryBillingReply {
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
pub enum TraceEntryRetryWhy {
    Backoff,
    RetryAfter,
}

impl TraceEntryRetryWhy {
    pub const ALL: [Self; 2] = [Self::Backoff, Self::RetryAfter];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Backoff => "backoff",
            Self::RetryAfter => "retryAfter",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for TraceEntryRetryWhy {
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

/// Decodes then encodes a named shape.
pub fn normalize(shape: &str, value: &Value) -> Option<Result<Value, Refused>> {
    let at = &mut Path::new();
    Some(match shape {
        "TraceEntry" => TraceEntry::decode_at(value, at).map(|held| held.encode()),
        _ => return None,
    })
}
