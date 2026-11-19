//! Generated from schema/health.output.json by scripts/contract.ts. Do not edit.

#![allow(
    clippy::enum_variant_names,
    clippy::large_enum_variant,
    clippy::ptr_arg
)]

use crate::json::{Object, Value};
use crate::shape::{self, Path, Refused, Shape};

#[derive(Debug, Clone, PartialEq)]
pub struct Ticket {
    pub gate: String,
    pub model: String,
    pub era: u64,
    pub generation: u64,
    pub probe: bool,
    pub serial: u64,
}

impl Shape for Ticket {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let gate = shape::required(object, "gate", at, shape::text)?;
        let model = shape::required(object, "model", at, shape::text)?;
        let era = shape::required(object, "era", at, shape::natural)?;
        let generation = shape::required(object, "generation", at, shape::natural)?;
        let probe = shape::required(object, "probe", at, shape::flag)?;
        let serial = shape::required(object, "serial", at, shape::natural)?;
        Ok(Self {
            gate,
            model,
            era,
            generation,
            probe,
            serial,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("gate", Value::from(self.gate.as_str()));
        object.insert("model", Value::from(self.model.as_str()));
        object.insert("era", Value::Number(self.era as f64));
        object.insert("generation", Value::Number(self.generation as f64));
        object.insert("probe", Value::Bool(self.probe));
        object.insert("serial", Value::Number(self.serial as f64));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Admission {
    Go(AdmissionGo),
    GaveUp(AdmissionGaveUp),
    Stopped(AdmissionStopped),
    Retired(AdmissionRetired),
}

impl Shape for Admission {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        match shape::tag(object, "kind", at)? {
            "go" => AdmissionGo::decode_at(value, at).map(Self::Go),
            "gaveUp" => AdmissionGaveUp::decode_at(value, at).map(Self::GaveUp),
            "stopped" => AdmissionStopped::decode_at(value, at).map(Self::Stopped),
            "retired" => AdmissionRetired::decode_at(value, at).map(Self::Retired),
            _ => Err(shape::no_branch(at, "kind")),
        }
    }

    fn encode(&self) -> Value {
        match self {
            Self::Go(held) => held.encode(),
            Self::GaveUp(held) => held.encode(),
            Self::Stopped(held) => held.encode(),
            Self::Retired(held) => held.encode(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdmissionGo {
    pub ticket: Ticket,
}

impl Shape for AdmissionGo {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "go")
        })?;
        let ticket = shape::required(object, "ticket", at, Ticket::decode_at)?;
        Ok(Self { ticket })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("go"));
        object.insert("ticket", self.ticket.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdmissionGaveUp {}

impl Shape for AdmissionGaveUp {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "gaveUp")
        })?;
        Ok(Self {})
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("gaveUp"));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdmissionStopped {}

impl Shape for AdmissionStopped {
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
pub struct AdmissionRetired {}

impl Shape for AdmissionRetired {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "retired")
        })?;
        Ok(Self {})
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("retired"));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GateWaiting {
    pub model: String,
    pub resume_at: Option<f64>,
    pub pause: u64,
    pub of: u64,
}

impl Shape for GateWaiting {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let model = shape::required(object, "model", at, shape::text)?;
        let resume_at = shape::required(object, "resumeAt", at, |value, at| {
            shape::nullable(value, at, shape::number)
        })?;
        let pause = shape::required(object, "pause", at, shape::natural)?;
        let of = shape::required(object, "of", at, shape::natural)?;
        Ok(Self {
            model,
            resume_at,
            pause,
            of,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("model", Value::from(self.model.as_str()));
        object.insert(
            "resumeAt",
            self.resume_at
                .as_ref()
                .map_or(Value::Null, |held| Value::Number(*held)),
        );
        object.insert("pause", Value::Number(self.pause as f64));
        object.insert("of", Value::Number(self.of as f64));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GateSettled {
    Done,
    Again,
    GaveUp,
}

impl GateSettled {
    pub const ALL: [Self; 3] = [Self::Done, Self::Again, Self::GaveUp];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Done => "done",
            Self::Again => "again",
            Self::GaveUp => "gaveUp",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for GateSettled {
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
pub enum GateOutcome {
    Ok,
    Overloaded,
    Failed,
    Cancelled,
}

impl GateOutcome {
    pub const ALL: [Self; 4] = [Self::Ok, Self::Overloaded, Self::Failed, Self::Cancelled];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Overloaded => "overloaded",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for GateOutcome {
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
pub enum PoolState {
    Clear(PoolStateClear),
    Paused(PoolStatePaused),
    Reset(PoolStateReset),
    Balance(PoolStateBalance),
}

impl Shape for PoolState {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        match shape::tag(object, "kind", at)? {
            "clear" => PoolStateClear::decode_at(value, at).map(Self::Clear),
            "paused" => PoolStatePaused::decode_at(value, at).map(Self::Paused),
            "reset" => PoolStateReset::decode_at(value, at).map(Self::Reset),
            "balance" => PoolStateBalance::decode_at(value, at).map(Self::Balance),
            _ => Err(shape::no_branch(at, "kind")),
        }
    }

    fn encode(&self) -> Value {
        match self {
            Self::Clear(held) => held.encode(),
            Self::Paused(held) => held.encode(),
            Self::Reset(held) => held.encode(),
            Self::Balance(held) => held.encode(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PoolStateClear {}

impl Shape for PoolStateClear {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "clear")
        })?;
        Ok(Self {})
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("clear"));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PoolStatePaused {
    pub reason: PoolStatePausedReason,
    pub until: f64,
}

impl Shape for PoolStatePaused {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "paused")
        })?;
        let reason = shape::required(object, "reason", at, PoolStatePausedReason::decode_at)?;
        let until = shape::required(object, "until", at, shape::number)?;
        Ok(Self { reason, until })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("paused"));
        object.insert("reason", self.reason.encode());
        object.insert("until", Value::Number(self.until));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PoolStateReset {
    pub at: f64,
}

impl Shape for PoolStateReset {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "reset")
        })?;
        let at_field = shape::required(object, "at", at, shape::number)?;
        Ok(Self { at: at_field })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("reset"));
        object.insert("at", Value::Number(self.at));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PoolStateBalance {
    pub eta: f64,
    pub next: f64,
    pub probe: Option<PoolStateBalanceProbe>,
}

impl Shape for PoolStateBalance {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "balance")
        })?;
        let eta = shape::required(object, "eta", at, shape::number)?;
        let next = shape::required(object, "next", at, shape::number)?;
        let probe = shape::required(object, "probe", at, |value, at| {
            shape::nullable(value, at, PoolStateBalanceProbe::decode_at)
        })?;
        Ok(Self { eta, next, probe })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("balance"));
        object.insert("eta", Value::Number(self.eta));
        object.insert("next", Value::Number(self.next));
        object.insert(
            "probe",
            self.probe
                .as_ref()
                .map_or(Value::Null, |held| held.encode()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PoolStatePausedReason {
    Allowance,
    Credit,
}

impl PoolStatePausedReason {
    pub const ALL: [Self; 2] = [Self::Allowance, Self::Credit];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Allowance => "allowance",
            Self::Credit => "credit",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for PoolStatePausedReason {
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
pub struct PoolStateBalanceProbe {
    pub ticket: Ticket,
    pub at: f64,
}

impl Shape for PoolStateBalanceProbe {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let ticket = shape::required(object, "ticket", at, Ticket::decode_at)?;
        let at_field = shape::required(object, "at", at, shape::number)?;
        Ok(Self {
            ticket,
            at: at_field,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("ticket", self.ticket.encode());
        object.insert("at", Value::Number(self.at));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PoolEvent {
    Billed(PoolEventBilled),
    Refused(PoolEventRefused),
    ExtraUsage(PoolEventExtraUsage),
    Admitted(PoolEventAdmitted),
    Settled(PoolEventSettled),
}

impl Shape for PoolEvent {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        match shape::tag(object, "kind", at)? {
            "billed" => PoolEventBilled::decode_at(value, at).map(Self::Billed),
            "refused" => PoolEventRefused::decode_at(value, at).map(Self::Refused),
            "extraUsage" => PoolEventExtraUsage::decode_at(value, at).map(Self::ExtraUsage),
            "admitted" => PoolEventAdmitted::decode_at(value, at).map(Self::Admitted),
            "settled" => PoolEventSettled::decode_at(value, at).map(Self::Settled),
            _ => Err(shape::no_branch(at, "kind")),
        }
    }

    fn encode(&self) -> Value {
        match self {
            Self::Billed(held) => held.encode(),
            Self::Refused(held) => held.encode(),
            Self::ExtraUsage(held) => held.encode(),
            Self::Admitted(held) => held.encode(),
            Self::Settled(held) => held.encode(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PoolEventBilled {
    pub source: String,
    pub plan: Option<String>,
    pub extra_usage: bool,
}

impl Shape for PoolEventBilled {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "billed")
        })?;
        let source = shape::required(object, "source", at, shape::text)?;
        let plan = shape::optional(object, "plan", at, shape::text)?;
        let extra_usage = shape::required(object, "extraUsage", at, shape::flag)?;
        Ok(Self {
            source,
            plan,
            extra_usage,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("billed"));
        object.insert("source", Value::from(self.source.as_str()));
        if let Some(held) = &self.plan {
            object.insert("plan", Value::from(held.as_str()));
        }
        object.insert("extraUsage", Value::Bool(self.extra_usage));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PoolEventRefused {
    pub reason: PoolEventRefusedReason,
    pub resets_at: Option<f64>,
    pub retry_after_ms: Option<f64>,
}

impl Shape for PoolEventRefused {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "refused")
        })?;
        let reason = shape::required(object, "reason", at, PoolEventRefusedReason::decode_at)?;
        let resets_at = shape::optional(object, "resetsAt", at, shape::number)?;
        let retry_after_ms = shape::optional(object, "retryAfterMs", at, shape::number)?;
        Ok(Self {
            reason,
            resets_at,
            retry_after_ms,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("refused"));
        object.insert("reason", self.reason.encode());
        if let Some(held) = &self.resets_at {
            object.insert("resetsAt", Value::Number(*held));
        }
        if let Some(held) = &self.retry_after_ms {
            object.insert("retryAfterMs", Value::Number(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PoolEventExtraUsage {
    pub on: bool,
    pub last_bill: Option<String>,
}

impl Shape for PoolEventExtraUsage {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "extraUsage")
        })?;
        let on = shape::required(object, "on", at, shape::flag)?;
        let last_bill = shape::optional(object, "lastBill", at, shape::text)?;
        Ok(Self { on, last_bill })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("extraUsage"));
        object.insert("on", Value::Bool(self.on));
        if let Some(held) = &self.last_bill {
            object.insert("lastBill", Value::from(held.as_str()));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PoolEventAdmitted {
    pub ticket: Ticket,
}

impl Shape for PoolEventAdmitted {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "admitted")
        })?;
        let ticket = shape::required(object, "ticket", at, Ticket::decode_at)?;
        Ok(Self { ticket })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("admitted"));
        object.insert("ticket", self.ticket.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PoolEventSettled {
    pub ticket: Ticket,
}

impl Shape for PoolEventSettled {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "settled")
        })?;
        let ticket = shape::required(object, "ticket", at, Ticket::decode_at)?;
        Ok(Self { ticket })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("settled"));
        object.insert("ticket", self.ticket.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PoolEventRefusedReason {
    Allowance,
    Credit,
}

impl PoolEventRefusedReason {
    pub const ALL: [Self; 2] = [Self::Allowance, Self::Credit];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Allowance => "allowance",
            Self::Credit => "credit",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for PoolEventRefusedReason {
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
pub struct PoolVerdict {
    pub skip: crate::shapes::provider::SkipReason,
    pub blocked: Option<Blocked>,
}

impl Shape for PoolVerdict {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let skip = shape::required(
            object,
            "skip",
            at,
            crate::shapes::provider::SkipReason::decode_at,
        )?;
        let blocked = shape::optional(object, "blocked", at, Blocked::decode_at)?;
        Ok(Self { skip, blocked })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("skip", self.skip.encode());
        if let Some(held) = &self.blocked {
            object.insert("blocked", held.encode());
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Blocked {
    KeyRejected(BlockedKeyRejected),
    SpendLimit(BlockedSpendLimit),
}

impl Shape for Blocked {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        match shape::tag(object, "kind", at)? {
            "keyRejected" => BlockedKeyRejected::decode_at(value, at).map(Self::KeyRejected),
            "spendLimit" => BlockedSpendLimit::decode_at(value, at).map(Self::SpendLimit),
            _ => Err(shape::no_branch(at, "kind")),
        }
    }

    fn encode(&self) -> Value {
        match self {
            Self::KeyRejected(held) => held.encode(),
            Self::SpendLimit(held) => held.encode(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockedKeyRejected {}

impl Shape for BlockedKeyRejected {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "keyRejected")
        })?;
        Ok(Self {})
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("keyRejected"));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockedSpendLimit {
    pub resets_at: Option<f64>,
    pub next_try_at: Option<f64>,
}

impl Shape for BlockedSpendLimit {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "spendLimit")
        })?;
        let resets_at = shape::optional(object, "resetsAt", at, shape::number)?;
        let next_try_at = shape::optional(object, "nextTryAt", at, shape::number)?;
        Ok(Self {
            resets_at,
            next_try_at,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("spendLimit"));
        if let Some(held) = &self.resets_at {
            object.insert("resetsAt", Value::Number(*held));
        }
        if let Some(held) = &self.next_try_at {
            object.insert("nextTryAt", Value::Number(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthChange {
    pub entry: String,
    pub changed: Vec<HealthChangeChangedItem>,
}

impl Shape for HealthChange {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let entry = shape::required(object, "entry", at, shape::text)?;
        let changed = shape::required(object, "changed", at, |value, at| {
            shape::list_within(value, at, Some(1), None, HealthChangeChangedItem::decode_at)
        })?;
        Ok(Self { entry, changed })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("entry", Value::from(self.entry.as_str()));
        object.insert(
            "changed",
            Value::Array(self.changed.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HealthChangeChangedItem {
    Key,
    Billed,
    Pool,
    Waiting,
}

impl HealthChangeChangedItem {
    pub const ALL: [Self; 4] = [Self::Key, Self::Billed, Self::Pool, Self::Waiting];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Key => "key",
            Self::Billed => "billed",
            Self::Pool => "pool",
            Self::Waiting => "waiting",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for HealthChangeChangedItem {
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
pub struct EntryHealth {
    pub entry: String,
    pub key: crate::shapes::provider::KeyState,
    pub billed: Option<crate::shapes::provider::Billed>,
    pub pool: PoolState,
    pub waiting: Vec<String>,
}

impl Shape for EntryHealth {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let entry = shape::required(object, "entry", at, shape::text)?;
        let key = shape::required(
            object,
            "key",
            at,
            crate::shapes::provider::KeyState::decode_at,
        )?;
        let billed = shape::optional(
            object,
            "billed",
            at,
            crate::shapes::provider::Billed::decode_at,
        )?;
        let pool = shape::required(object, "pool", at, PoolState::decode_at)?;
        let waiting = shape::required(object, "waiting", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self {
            entry,
            key,
            billed,
            pool,
            waiting,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("entry", Value::from(self.entry.as_str()));
        object.insert("key", self.key.encode());
        if let Some(held) = &self.billed {
            object.insert("billed", held.encode());
        }
        object.insert("pool", self.pool.encode());
        object.insert(
            "waiting",
            Value::Array(
                self.waiting
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Notice {
    pub kind: crate::shapes::provider::NoticeKind,
    pub status: Option<ErrorStatus>,
    pub message: Option<String>,
    pub reasons: Option<Vec<String>>,
    pub limit: Option<f64>,
    pub reset: Option<String>,
    pub resets_at: Option<f64>,
    pub next_try_at: Option<f64>,
    pub model: Option<String>,
    pub entry: Option<String>,
}

impl Shape for Notice {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let kind = shape::required(
            object,
            "kind",
            at,
            crate::shapes::provider::NoticeKind::decode_at,
        )?;
        let status = shape::optional(object, "status", at, ErrorStatus::decode_at)?;
        let message = shape::optional(object, "message", at, shape::text)?;
        let reasons = shape::optional(object, "reasons", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let limit = shape::optional(object, "limit", at, shape::number)?;
        let reset = shape::optional(object, "reset", at, shape::text)?;
        let resets_at = shape::optional(object, "resetsAt", at, shape::number)?;
        let next_try_at = shape::optional(object, "nextTryAt", at, shape::number)?;
        let model = shape::optional(object, "model", at, shape::text)?;
        let entry = shape::optional(object, "entry", at, shape::text)?;
        Ok(Self {
            kind,
            status,
            message,
            reasons,
            limit,
            reset,
            resets_at,
            next_try_at,
            model,
            entry,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", self.kind.encode());
        if let Some(held) = &self.status {
            object.insert("status", held.encode());
        }
        if let Some(held) = &self.message {
            object.insert("message", Value::from(held.as_str()));
        }
        if let Some(held) = &self.reasons {
            object.insert(
                "reasons",
                Value::Array(held.iter().map(|held| Value::from(held.as_str())).collect()),
            );
        }
        if let Some(held) = &self.limit {
            object.insert("limit", Value::Number(*held));
        }
        if let Some(held) = &self.reset {
            object.insert("reset", Value::from(held.as_str()));
        }
        if let Some(held) = &self.resets_at {
            object.insert("resetsAt", Value::Number(*held));
        }
        if let Some(held) = &self.next_try_at {
            object.insert("nextTryAt", Value::Number(*held));
        }
        if let Some(held) = &self.model {
            object.insert("model", Value::from(held.as_str()));
        }
        if let Some(held) = &self.entry {
            object.insert("entry", Value::from(held.as_str()));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorStatus {
    Number(f64),
    Text(String),
}

impl Shape for ErrorStatus {
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
pub struct RetryOptions {
    pub tries: Option<u64>,
    pub budget_ms: Option<f64>,
    pub cap_ms: Option<f64>,
    pub base_ms: Option<f64>,
}

impl Shape for RetryOptions {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let tries = shape::optional(object, "tries", at, shape::natural)?;
        let budget_ms = shape::optional(object, "budgetMs", at, |value, at| {
            shape::number(value, at).and_then(|held| shape::at_least(held, 0.0, at))
        })?;
        let cap_ms = shape::optional(object, "capMs", at, |value, at| {
            shape::number(value, at).and_then(|held| shape::at_least(held, 0.0, at))
        })?;
        let base_ms = shape::optional(object, "baseMs", at, |value, at| {
            shape::number(value, at).and_then(|held| shape::at_least(held, 0.0, at))
        })?;
        Ok(Self {
            tries,
            budget_ms,
            cap_ms,
            base_ms,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        if let Some(held) = &self.tries {
            object.insert("tries", Value::Number(*held as f64));
        }
        if let Some(held) = &self.budget_ms {
            object.insert("budgetMs", Value::Number(*held));
        }
        if let Some(held) = &self.cap_ms {
            object.insert("capMs", Value::Number(*held));
        }
        if let Some(held) = &self.base_ms {
            object.insert("baseMs", Value::Number(*held));
        }
        Value::Object(object)
    }
}

/// Decodes then encodes a named shape.
pub fn normalize(shape: &str, value: &Value) -> Option<Result<Value, Refused>> {
    let at = &mut Path::new();
    Some(match shape {
        "Ticket" => Ticket::decode_at(value, at).map(|held| held.encode()),
        "Admission" => Admission::decode_at(value, at).map(|held| held.encode()),
        "GateWaiting" => GateWaiting::decode_at(value, at).map(|held| held.encode()),
        "GateSettled" => GateSettled::decode_at(value, at).map(|held| held.encode()),
        "GateOutcome" => GateOutcome::decode_at(value, at).map(|held| held.encode()),
        "PoolState" => PoolState::decode_at(value, at).map(|held| held.encode()),
        "PoolEvent" => PoolEvent::decode_at(value, at).map(|held| held.encode()),
        "PoolVerdict" => PoolVerdict::decode_at(value, at).map(|held| held.encode()),
        "Blocked" => Blocked::decode_at(value, at).map(|held| held.encode()),
        "HealthChange" => HealthChange::decode_at(value, at).map(|held| held.encode()),
        "EntryHealth" => EntryHealth::decode_at(value, at).map(|held| held.encode()),
        "Notice" => Notice::decode_at(value, at).map(|held| held.encode()),
        "ErrorStatus" => ErrorStatus::decode_at(value, at).map(|held| held.encode()),
        "RetryOptions" => RetryOptions::decode_at(value, at).map(|held| held.encode()),
        _ => return None,
    })
}
