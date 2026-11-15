//! Generated from schema/sessionCut.output.json by scripts/contract.ts. Do not edit.

#![allow(
    clippy::enum_variant_names,
    clippy::large_enum_variant,
    clippy::ptr_arg
)]

use crate::json::{Object, Record, Value};
use crate::shape::{self, Path, Refused, Shape};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CutMode {
    Trim,
    Clear,
    Compact,
}

impl CutMode {
    pub const ALL: [Self; 3] = [Self::Trim, Self::Clear, Self::Compact];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Trim => "trim",
            Self::Clear => "clear",
            Self::Compact => "compact",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for CutMode {
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
pub enum CutRefusal {
    NoRows,
    CliVersion,
    BoundaryShape,
    UnreadableRows,
    NothingToTrim,
    PreservedUnresolved,
    TargetMissing,
    TargetCompacted,
    TargetNotTurnStart,
    TargetMidTool,
    TailMidTool,
    EndMissing,
    LateMessage,
    NothingAbove,
    SummaryNeeded,
}

impl CutRefusal {
    pub const ALL: [Self; 15] = [
        Self::NoRows,
        Self::CliVersion,
        Self::BoundaryShape,
        Self::UnreadableRows,
        Self::NothingToTrim,
        Self::PreservedUnresolved,
        Self::TargetMissing,
        Self::TargetCompacted,
        Self::TargetNotTurnStart,
        Self::TargetMidTool,
        Self::TailMidTool,
        Self::EndMissing,
        Self::LateMessage,
        Self::NothingAbove,
        Self::SummaryNeeded,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::NoRows => "no_rows",
            Self::CliVersion => "cli_version",
            Self::BoundaryShape => "boundary_shape",
            Self::UnreadableRows => "unreadable_rows",
            Self::NothingToTrim => "nothing_to_trim",
            Self::PreservedUnresolved => "preserved_unresolved",
            Self::TargetMissing => "target_missing",
            Self::TargetCompacted => "target_compacted",
            Self::TargetNotTurnStart => "target_not_turn_start",
            Self::TargetMidTool => "target_mid_tool",
            Self::TailMidTool => "tail_mid_tool",
            Self::EndMissing => "end_missing",
            Self::LateMessage => "late_message",
            Self::NothingAbove => "nothing_above",
            Self::SummaryNeeded => "summary_needed",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for CutRefusal {
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
pub struct CutRefused {
    pub reason: CutRefusal,
    pub detail: String,
}

impl Shape for CutRefused {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "ok", at, |value, at| {
            shape::is_flag(value, at, false)
        })?;
        let reason = shape::required(object, "reason", at, CutRefusal::decode_at)?;
        let detail = shape::required(object, "detail", at, shape::text)?;
        Ok(Self { reason, detail })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("ok", Value::Bool(false));
        object.insert("reason", self.reason.encode());
        object.insert("detail", Value::from(self.detail.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RowMeta {
    pub line: u64,
    pub bytes: u64,
    pub type_: String,
    pub uuid: Option<String>,
    pub at: Option<String>,
    pub open_tools: u64,
    pub delivered: bool,
    pub text: Option<String>,
    pub points: Vec<String>,
}

impl Shape for RowMeta {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let line = shape::required(object, "line", at, shape::natural)?;
        let bytes = shape::required(object, "bytes", at, shape::natural)?;
        let type_ = shape::required(object, "type", at, shape::text)?;
        let uuid = shape::optional(object, "uuid", at, shape::text)?;
        let at_field = shape::optional(object, "at", at, shape::text)?;
        let open_tools = shape::required(object, "openTools", at, shape::natural)?;
        let delivered = shape::required(object, "delivered", at, shape::flag)?;
        let text = shape::optional(object, "text", at, shape::text)?;
        let points = shape::required(object, "points", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self {
            line,
            bytes,
            type_,
            uuid,
            at: at_field,
            open_tools,
            delivered,
            text,
            points,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("line", Value::Number(self.line as f64));
        object.insert("bytes", Value::Number(self.bytes as f64));
        object.insert("type", Value::from(self.type_.as_str()));
        if let Some(held) = &self.uuid {
            object.insert("uuid", Value::from(held.as_str()));
        }
        if let Some(held) = &self.at {
            object.insert("at", Value::from(held.as_str()));
        }
        object.insert("openTools", Value::Number(self.open_tools as f64));
        object.insert("delivered", Value::Bool(self.delivered));
        if let Some(held) = &self.text {
            object.insert("text", Value::from(held.as_str()));
        }
        object.insert(
            "points",
            Value::Array(
                self.points
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BoundaryMeta {
    pub line: u64,
    pub uuid: String,
    pub preserved_lines: Vec<u64>,
    pub preserved_sound: bool,
    pub preserved: Vec<String>,
    pub summary_line: Option<u64>,
    pub cumulative_dropped_tokens: f64,
    pub post_tokens: f64,
    pub known: bool,
}

impl Shape for BoundaryMeta {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let line = shape::required(object, "line", at, shape::natural)?;
        let uuid = shape::required(object, "uuid", at, shape::text)?;
        let preserved_lines = shape::required(object, "preservedLines", at, |value, at| {
            shape::list(value, at, shape::natural)
        })?;
        let preserved_sound = shape::required(object, "preservedSound", at, shape::flag)?;
        let preserved = shape::required(object, "preserved", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let summary_line = shape::required(object, "summaryLine", at, |value, at| {
            shape::nullable(value, at, shape::natural)
        })?;
        let cumulative_dropped_tokens =
            shape::required(object, "cumulativeDroppedTokens", at, shape::number)?;
        let post_tokens = shape::required(object, "postTokens", at, shape::number)?;
        let known = shape::required(object, "known", at, shape::flag)?;
        Ok(Self {
            line,
            uuid,
            preserved_lines,
            preserved_sound,
            preserved,
            summary_line,
            cumulative_dropped_tokens,
            post_tokens,
            known,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("line", Value::Number(self.line as f64));
        object.insert("uuid", Value::from(self.uuid.as_str()));
        object.insert(
            "preservedLines",
            Value::Array(
                self.preserved_lines
                    .iter()
                    .map(|held| Value::Number(*held as f64))
                    .collect(),
            ),
        );
        object.insert("preservedSound", Value::Bool(self.preserved_sound));
        object.insert(
            "preserved",
            Value::Array(
                self.preserved
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        object.insert(
            "summaryLine",
            self.summary_line
                .as_ref()
                .map_or(Value::Null, |held| Value::Number(*held as f64)),
        );
        object.insert(
            "cumulativeDroppedTokens",
            Value::Number(self.cumulative_dropped_tokens),
        );
        object.insert("postTokens", Value::Number(self.post_tokens));
        object.insert("known", Value::Bool(self.known));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SessionIndex {
    pub lines: u64,
    pub unparsed: u64,
    pub unreadable: Vec<u64>,
    pub version: Option<String>,
    pub header: Record<u64>,
    pub boundary: Option<BoundaryMeta>,
    pub boundaries: u64,
    pub standing: Option<String>,
    pub tail: Vec<RowMeta>,
}

impl Shape for SessionIndex {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let lines = shape::required(object, "lines", at, shape::natural)?;
        let unparsed = shape::required(object, "unparsed", at, shape::natural)?;
        let unreadable = shape::required(object, "unreadable", at, |value, at| {
            shape::list(value, at, shape::natural)
        })?;
        let version = shape::required(object, "version", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        let header = shape::required(object, "header", at, |value, at| {
            shape::record(value, at, shape::natural)
        })?;
        let boundary = shape::required(object, "boundary", at, |value, at| {
            shape::nullable(value, at, BoundaryMeta::decode_at)
        })?;
        let boundaries = shape::required(object, "boundaries", at, shape::natural)?;
        let standing = shape::required(object, "standing", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        let tail = shape::required(object, "tail", at, |value, at| {
            shape::list(value, at, RowMeta::decode_at)
        })?;
        Ok(Self {
            lines,
            unparsed,
            unreadable,
            version,
            header,
            boundary,
            boundaries,
            standing,
            tail,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("lines", Value::Number(self.lines as f64));
        object.insert("unparsed", Value::Number(self.unparsed as f64));
        object.insert(
            "unreadable",
            Value::Array(
                self.unreadable
                    .iter()
                    .map(|held| Value::Number(*held as f64))
                    .collect(),
            ),
        );
        object.insert(
            "version",
            self.version
                .as_ref()
                .map_or(Value::Null, |held| Value::from(held.as_str())),
        );
        object.insert(
            "header",
            shape::record_value(&self.header, |held| Value::Number(*held as f64)),
        );
        object.insert(
            "boundary",
            self.boundary
                .as_ref()
                .map_or(Value::Null, |held| held.encode()),
        );
        object.insert("boundaries", Value::Number(self.boundaries as f64));
        object.insert(
            "standing",
            self.standing
                .as_ref()
                .map_or(Value::Null, |held| Value::from(held.as_str())),
        );
        object.insert(
            "tail",
            Value::Array(self.tail.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CutRequest {
    pub mode: CutMode,
    pub target_uuid: Option<String>,
    pub through_uuid: Option<String>,
    pub end_before: Option<String>,
    pub summary_text: Option<String>,
    pub live_tail: Option<bool>,
}

impl Shape for CutRequest {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let mode = shape::required(object, "mode", at, CutMode::decode_at)?;
        let target_uuid = shape::optional(object, "targetUuid", at, shape::text)?;
        let through_uuid = shape::optional(object, "throughUuid", at, shape::text)?;
        let end_before = shape::optional(object, "endBefore", at, shape::text)?;
        let summary_text = shape::optional(object, "summaryText", at, shape::text)?;
        let live_tail = shape::optional(object, "liveTail", at, shape::flag)?;
        Ok(Self {
            mode,
            target_uuid,
            through_uuid,
            end_before,
            summary_text,
            live_tail,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("mode", self.mode.encode());
        if let Some(held) = &self.target_uuid {
            object.insert("targetUuid", Value::from(held.as_str()));
        }
        if let Some(held) = &self.through_uuid {
            object.insert("throughUuid", Value::from(held.as_str()));
        }
        if let Some(held) = &self.end_before {
            object.insert("endBefore", Value::from(held.as_str()));
        }
        if let Some(held) = &self.summary_text {
            object.insert("summaryText", Value::from(held.as_str()));
        }
        if let Some(held) = &self.live_tail {
            object.insert("liveTail", Value::Bool(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TurnPick {
    pub at: i64,
    pub text: Option<String>,
}

impl Shape for TurnPick {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let at_field = shape::required(object, "at", at, shape::integer)?;
        let text = shape::optional(object, "text", at, shape::text)?;
        Ok(Self { at: at_field, text })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("at", Value::Number(self.at as f64));
        if let Some(held) = &self.text {
            object.insert("text", Value::from(held.as_str()));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PickedTurn {
    pub uuid: String,
    pub exact: bool,
}

impl Shape for PickedTurn {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let uuid = shape::required(object, "uuid", at, shape::text)?;
        let exact = shape::required(object, "exact", at, shape::flag)?;
        Ok(Self { uuid, exact })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("uuid", Value::from(self.uuid.as_str()));
        object.insert("exact", Value::Bool(self.exact));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Preserved {
    pub head: String,
    pub tail: String,
    pub uuids: Vec<String>,
}

impl Shape for Preserved {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let head = shape::required(object, "head", at, shape::text)?;
        let tail = shape::required(object, "tail", at, shape::text)?;
        let uuids = shape::required(object, "uuids", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self { head, tail, uuids })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("head", Value::from(self.head.as_str()));
        object.insert("tail", Value::from(self.tail.as_str()));
        object.insert(
            "uuids",
            Value::Array(
                self.uuids
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KeptFrom {
    pub at: Option<String>,
    pub text: String,
}

impl Shape for KeptFrom {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let at_field = shape::required(object, "at", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        let text = shape::required(object, "text", at, shape::text)?;
        Ok(Self { at: at_field, text })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "at",
            self.at
                .as_ref()
                .map_or(Value::Null, |held| Value::from(held.as_str())),
        );
        object.insert("text", Value::from(self.text.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CutPlan {
    pub mode: CutMode,
    pub header_lines: Vec<u64>,
    pub kept_lines: Vec<u64>,
    pub preserved: Option<Preserved>,
    pub fold_lines: Vec<u64>,
    pub kept_bytes: u64,
    pub dropped_bytes: u64,
    pub dropped_rows: u64,
    pub from: Option<KeptFrom>,
}

impl Shape for CutPlan {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "ok", at, |value, at| {
            shape::is_flag(value, at, true)
        })?;
        let mode = shape::required(object, "mode", at, CutMode::decode_at)?;
        let header_lines = shape::required(object, "headerLines", at, |value, at| {
            shape::list(value, at, shape::natural)
        })?;
        let kept_lines = shape::required(object, "keptLines", at, |value, at| {
            shape::list(value, at, shape::natural)
        })?;
        let preserved = shape::required(object, "preserved", at, |value, at| {
            shape::nullable(value, at, Preserved::decode_at)
        })?;
        let fold_lines = shape::required(object, "foldLines", at, |value, at| {
            shape::list(value, at, shape::natural)
        })?;
        let kept_bytes = shape::required(object, "keptBytes", at, shape::natural)?;
        let dropped_bytes = shape::required(object, "droppedBytes", at, shape::natural)?;
        let dropped_rows = shape::required(object, "droppedRows", at, shape::natural)?;
        let from = shape::required(object, "from", at, |value, at| {
            shape::nullable(value, at, KeptFrom::decode_at)
        })?;
        Ok(Self {
            mode,
            header_lines,
            kept_lines,
            preserved,
            fold_lines,
            kept_bytes,
            dropped_bytes,
            dropped_rows,
            from,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("ok", Value::Bool(true));
        object.insert("mode", self.mode.encode());
        object.insert(
            "headerLines",
            Value::Array(
                self.header_lines
                    .iter()
                    .map(|held| Value::Number(*held as f64))
                    .collect(),
            ),
        );
        object.insert(
            "keptLines",
            Value::Array(
                self.kept_lines
                    .iter()
                    .map(|held| Value::Number(*held as f64))
                    .collect(),
            ),
        );
        object.insert(
            "preserved",
            self.preserved
                .as_ref()
                .map_or(Value::Null, |held| held.encode()),
        );
        object.insert(
            "foldLines",
            Value::Array(
                self.fold_lines
                    .iter()
                    .map(|held| Value::Number(*held as f64))
                    .collect(),
            ),
        );
        object.insert("keptBytes", Value::Number(self.kept_bytes as f64));
        object.insert("droppedBytes", Value::Number(self.dropped_bytes as f64));
        object.insert("droppedRows", Value::Number(self.dropped_rows as f64));
        object.insert(
            "from",
            self.from.as_ref().map_or(Value::Null, |held| held.encode()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CutDone {
    pub session_id: String,
    pub plan: CutPlan,
}

impl Shape for CutDone {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "ok", at, |value, at| {
            shape::is_flag(value, at, true)
        })?;
        let session_id = shape::required(object, "sessionId", at, shape::text)?;
        let plan = shape::required(object, "plan", at, CutPlan::decode_at)?;
        Ok(Self { session_id, plan })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("ok", Value::Bool(true));
        object.insert("sessionId", Value::from(self.session_id.as_str()));
        object.insert("plan", self.plan.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanCommand {
    pub source: String,
    pub request: CutRequest,
    pub cli_version: String,
}

impl Shape for PlanCommand {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let source = shape::required(object, "source", at, shape::text)?;
        let request = shape::required(object, "request", at, CutRequest::decode_at)?;
        let cli_version = shape::required(object, "cliVersion", at, shape::text)?;
        Ok(Self {
            source,
            request,
            cli_version,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("source", Value::from(self.source.as_str()));
        object.insert("request", self.request.encode());
        object.insert("cliVersion", Value::from(self.cli_version.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PlanOutcome {
    CutPlan(CutPlan),
    CutRefused(CutRefused),
}

impl Shape for PlanOutcome {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        if let Ok(held) = CutPlan::decode_at(value, at) {
            return Ok(Self::CutPlan(held));
        }
        if let Ok(held) = CutRefused::decode_at(value, at) {
            return Ok(Self::CutRefused(held));
        }
        Err(shape::refused(at, "no branch matches"))
    }

    fn encode(&self) -> Value {
        match self {
            Self::CutPlan(held) => held.encode(),
            Self::CutRefused(held) => held.encode(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApplyCommand {
    pub source: String,
    pub out: String,
    pub keep_out_of: Option<String>,
    pub request: CutRequest,
    pub cli_version: String,
}

impl Shape for ApplyCommand {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let source = shape::required(object, "source", at, shape::text)?;
        let out = shape::required(object, "out", at, shape::text)?;
        let keep_out_of = shape::optional(object, "keepOutOf", at, shape::text)?;
        let request = shape::required(object, "request", at, CutRequest::decode_at)?;
        let cli_version = shape::required(object, "cliVersion", at, shape::text)?;
        Ok(Self {
            source,
            out,
            keep_out_of,
            request,
            cli_version,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("source", Value::from(self.source.as_str()));
        object.insert("out", Value::from(self.out.as_str()));
        if let Some(held) = &self.keep_out_of {
            object.insert("keepOutOf", Value::from(held.as_str()));
        }
        object.insert("request", self.request.encode());
        object.insert("cliVersion", Value::from(self.cli_version.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CutOutcome {
    CutDone(CutDone),
    CutRefused(CutRefused),
}

impl Shape for CutOutcome {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        if let Ok(held) = CutDone::decode_at(value, at) {
            return Ok(Self::CutDone(held));
        }
        if let Ok(held) = CutRefused::decode_at(value, at) {
            return Ok(Self::CutRefused(held));
        }
        Err(shape::refused(at, "no branch matches"))
    }

    fn encode(&self) -> Value {
        match self {
            Self::CutDone(held) => held.encode(),
            Self::CutRefused(held) => held.encode(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SummarizeRequest {
    pub standing: Option<String>,
    pub rows: Vec<String>,
}

impl Shape for SummarizeRequest {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let standing = shape::required(object, "standing", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        let rows = shape::required(object, "rows", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self { standing, rows })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "standing",
            self.standing
                .as_ref()
                .map_or(Value::Null, |held| Value::from(held.as_str())),
        );
        object.insert(
            "rows",
            Value::Array(
                self.rows
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

/// Decodes then encodes a named shape.
pub fn normalize(shape: &str, value: &Value) -> Option<Result<Value, Refused>> {
    let at = &mut Path::new();
    Some(match shape {
        "CutMode" => CutMode::decode_at(value, at).map(|held| held.encode()),
        "CutRefusal" => CutRefusal::decode_at(value, at).map(|held| held.encode()),
        "CutRefused" => CutRefused::decode_at(value, at).map(|held| held.encode()),
        "RowMeta" => RowMeta::decode_at(value, at).map(|held| held.encode()),
        "BoundaryMeta" => BoundaryMeta::decode_at(value, at).map(|held| held.encode()),
        "SessionIndex" => SessionIndex::decode_at(value, at).map(|held| held.encode()),
        "CutRequest" => CutRequest::decode_at(value, at).map(|held| held.encode()),
        "TurnPick" => TurnPick::decode_at(value, at).map(|held| held.encode()),
        "PickedTurn" => PickedTurn::decode_at(value, at).map(|held| held.encode()),
        "Preserved" => Preserved::decode_at(value, at).map(|held| held.encode()),
        "KeptFrom" => KeptFrom::decode_at(value, at).map(|held| held.encode()),
        "CutPlan" => CutPlan::decode_at(value, at).map(|held| held.encode()),
        "CutDone" => CutDone::decode_at(value, at).map(|held| held.encode()),
        "PlanCommand" => PlanCommand::decode_at(value, at).map(|held| held.encode()),
        "PlanOutcome" => PlanOutcome::decode_at(value, at).map(|held| held.encode()),
        "ApplyCommand" => ApplyCommand::decode_at(value, at).map(|held| held.encode()),
        "CutOutcome" => CutOutcome::decode_at(value, at).map(|held| held.encode()),
        "SummarizeRequest" => SummarizeRequest::decode_at(value, at).map(|held| held.encode()),
        _ => return None,
    })
}
