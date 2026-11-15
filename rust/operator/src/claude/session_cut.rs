//! Cuts of a Claude Code session file: trim to the live segment, or clear or compact above a turn.
//!
//! The twin of `sessionCut.ts`. Rows are text lines; a kept row is copied as it stands.

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::io;

use nyaaai_contract::json::{Object, Value, parse_json, quote_json, stringify_json, to_js_string};
use nyaaai_contract::time::{iso_time, parse_iso_time};

/// CLI versions whose session file shape a built cut is known to resume under.
pub const CUT_CLI_VERSIONS: &[&str] = &["2.1.323"];

/// Trim keeps the live segment; clear and compact cut above a chosen turn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CutMode {
    Trim,
    Clear,
    Compact,
}

impl CutMode {
    pub fn as_str(self) -> &'static str {
        match self {
            CutMode::Trim => "trim",
            CutMode::Clear => "clear",
            CutMode::Compact => "compact",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        [CutMode::Trim, CutMode::Clear, CutMode::Compact]
            .into_iter()
            .find(|mode| mode.as_str() == text)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    pub fn as_str(self) -> &'static str {
        match self {
            CutRefusal::NoRows => "no_rows",
            CutRefusal::CliVersion => "cli_version",
            CutRefusal::BoundaryShape => "boundary_shape",
            CutRefusal::UnreadableRows => "unreadable_rows",
            CutRefusal::NothingToTrim => "nothing_to_trim",
            CutRefusal::PreservedUnresolved => "preserved_unresolved",
            CutRefusal::TargetMissing => "target_missing",
            CutRefusal::TargetCompacted => "target_compacted",
            CutRefusal::TargetNotTurnStart => "target_not_turn_start",
            CutRefusal::TargetMidTool => "target_mid_tool",
            CutRefusal::TailMidTool => "tail_mid_tool",
            CutRefusal::EndMissing => "end_missing",
            CutRefusal::LateMessage => "late_message",
            CutRefusal::NothingAbove => "nothing_above",
            CutRefusal::SummaryNeeded => "summary_needed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CutRefused {
    pub reason: CutRefusal,
    pub detail: String,
}

/// One row after the live boundary, without its text.
#[derive(Debug, Clone)]
pub struct RowMeta {
    pub line: usize,
    pub bytes: usize,
    /// The row's `type`.
    pub kind: String,
    pub uuid: Option<String>,
    pub at: Option<String>,
    /// Tool calls still unanswered before this row.
    pub open_tools: usize,
    /// A message reached the session here: a turn start, or one taken in mid-turn. A cut point.
    pub delivered: bool,
    /// A delivered message's own text.
    pub text: Option<String>,
    /// Rows this one names: a prompt's leaf, a file snapshot's message.
    pub points: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct BoundaryMeta {
    pub line: usize,
    pub uuid: String,
    /// Its preserved rows, which sit before it in the file.
    pub preserved_lines: Vec<usize>,
    /// Every preserved id is a row, and they are the chain from tail back to head.
    pub preserved_sound: bool,
    pub preserved: Vec<String>,
    pub summary_line: Option<usize>,
    pub cumulative_dropped_tokens: f64,
    pub post_tokens: f64,
    /// Its keys and value types are ones a cut knows.
    pub known: bool,
}

#[derive(Debug, Clone)]
pub struct SessionIndex {
    pub lines: usize,
    pub unparsed: usize,
    /// Lines after the live boundary that do not parse. A cut that keeps past one would lose it.
    pub unreadable: Vec<usize>,
    pub version: Option<String>,
    /// The newest of each header row, in first-seen order.
    pub header: Vec<(String, usize)>,
    pub boundary: Option<BoundaryMeta>,
    /// Compaction rows in the whole file.
    pub boundaries: usize,
    /// The live boundary's summary text.
    pub standing: Option<String>,
    /// Rows after the live boundary, or every row when there is none. The last entry is the file's end.
    pub tail: Vec<RowMeta>,
}

#[derive(Debug, Clone)]
pub struct CutRequest {
    pub mode: CutMode,
    /// The turn start to keep from. Unused by trim.
    pub target_uuid: Option<String>,
    /// The last row to keep; the file's end when absent.
    pub through_uuid: Option<String>,
    /// Ends the kept rows before the first later turn start whose text holds this.
    pub end_before: Option<String>,
    /// A compact's summary, when the caller already holds it.
    pub summary_text: Option<String>,
    /// A plan while the session still writes: where the kept rows end is checked at the build.
    pub live_tail: bool,
}

impl CutRequest {
    pub fn new(mode: CutMode) -> Self {
        Self {
            mode,
            target_uuid: None,
            through_uuid: None,
            end_before: None,
            summary_text: None,
            live_tail: false,
        }
    }
}

/// A chat row to find a turn by: its time, and its text when it is a message the session was sent.
#[derive(Debug, Clone)]
pub struct TurnPick {
    pub at: i64,
    pub text: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PickedTurn {
    pub uuid: String,
    /// The row's own text matched; otherwise the turn at or before its time.
    pub exact: bool,
}

/// The segment a built boundary preserves.
#[derive(Debug, Clone)]
pub struct Preserved {
    pub head: String,
    pub tail: String,
    pub uuids: Vec<String>,
}

/// The kept turn, for a confirmation.
#[derive(Debug, Clone)]
pub struct KeptFrom {
    pub at: Option<String>,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct CutPlan {
    pub mode: CutMode,
    /// The newest header rows, written first.
    pub header_lines: Vec<usize>,
    /// Lines kept verbatim, ascending.
    pub kept_lines: Vec<usize>,
    /// Clear and compact: the segment a built boundary preserves. Trim builds none.
    pub preserved: Option<Preserved>,
    /// Rows a summarizer folds: after the standing summary, above the target.
    pub fold_lines: Vec<usize>,
    pub kept_bytes: usize,
    pub dropped_bytes: usize,
    pub dropped_rows: usize,
    pub from: Option<KeptFrom>,
}

pub struct BuildInputs<'a> {
    pub session_id: &'a str,
    /// Clear and compact: the built summary's text.
    pub summary_text: Option<&'a str>,
    pub new_id: &'a dyn Fn() -> String,
    pub now_ms: &'a dyn Fn() -> i64,
}

/// Rows of text, each fallible to read.
pub type Lines<'a> = Box<dyn Iterator<Item = io::Result<String>> + 'a>;

pub trait CutPorts {
    /// The source's rows, split on LF alone, readable more than once. Never opened for writing.
    fn read(&self) -> io::Result<Lines<'_>>;
    /// Where the source is, for the summary to name.
    fn source_path(&self) -> String;
    /// Exclusive create of the new session's file.
    fn create(&self, session_id: &str, lines: &[String]) -> io::Result<()>;
    /// The built file's rows, for the read-back.
    fn read_built(&self, session_id: &str) -> io::Result<Lines<'_>>;
    /// Removes a built file that failed its read-back.
    fn discard(&self, session_id: &str) -> io::Result<()>;
    fn new_id(&self) -> String;
    fn now_ms(&self) -> i64;
    fn cli_version(&self) -> io::Result<String>;
    /// Whether `summarize` answers.
    fn can_summarize(&self) -> bool {
        false
    }
    /// Folds what stood and the rows above the cut into a summary.
    fn summarize(&self, _standing: Option<&str>, _rows: &[String]) -> io::Result<String> {
        Err(io::Error::other("no summarizer"))
    }
}

#[derive(Debug, Clone)]
pub struct CutDone {
    pub session_id: String,
    pub plan: CutPlan,
}

/// A finished cut, or why none was made.
pub type CutOutcome = Result<CutDone, CutRefused>;

#[derive(Debug)]
pub enum CutError {
    Io(io::Error),
    /// What would stop the built file resuming.
    ReadBack(Vec<String>),
}

impl fmt::Display for CutError {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CutError::Io(error) => write!(out, "{error}"),
            CutError::ReadBack(problems) => {
                let shown: Vec<&str> = problems.iter().take(4).map(String::as_str).collect();
                write!(
                    out,
                    "the built session file failed its read-back: {}",
                    shown.join("; ")
                )
            }
        }
    }
}

impl std::error::Error for CutError {}

impl From<io::Error> for CutError {
    fn from(error: io::Error) -> Self {
        CutError::Io(error)
    }
}

const HEADER_TYPES: &[&str] = &["mode", "permission-mode", "atis-latch", "session-owner"];
/// The CLI's request log; a resume reads none of it.
const SKIPPED_TYPES: &[&str] = &["api-request", "api-request-blob", "api-request-shape"];
const SESSION_KEYS: &[&str] = &["sessionId", "session_id"];
/// Keys under which a row names another row.
const POINTER_KEYS: &[&str] = &["leafUuid", "messageId", "snapshotMessageId"];

/// The boundary a cut writes, in the CLI's own key order.
const BOUNDARY_KEYS: &[&str] = &[
    "parentUuid",
    "logicalParentUuid",
    "isSidechain",
    "type",
    "subtype",
    "content",
    "level",
    "compactMetadata",
    "uuid",
    "timestamp",
    "userType",
    "entrypoint",
    "cwd",
    "sessionId",
    "version",
    "gitBranch",
    "slug",
];
/// An automatic compaction's boundary also carries this.
const BOUNDARY_EXTRA_KEYS: &[&str] = &["isMeta"];
const METADATA_KEYS: &[&str] = &[
    "trigger",
    "preTokens",
    "postTokens",
    "cumulativeDroppedTokens",
    "durationMs",
    "preservedSegment",
    "preservedMessages",
];
const METADATA_EXTRA_KEYS: &[&str] = &["preCompactArtifactReadVersions"];
/// A turn starts this long after its message was sent, at most.
const PICK_SLACK_MS: i64 = 2000;

fn refused(reason: CutRefusal, detail: &str) -> CutRefused {
    CutRefused {
        reason,
        detail: detail.to_owned(),
    }
}

fn parse(text: &str) -> Option<Object> {
    match parse_json(text)? {
        Value::Object(row) => Some(row),
        _ => None,
    }
}

fn text_of_key<'a>(row: &'a Object, key: &str) -> Option<&'a str> {
    row.get(key).and_then(Value::as_str)
}

fn is_true(row: &Object, key: &str) -> bool {
    row.get(key).and_then(Value::as_bool) == Some(true)
}

fn is_boundary(row: &Object) -> bool {
    text_of_key(row, "type") == Some("system")
        && text_of_key(row, "subtype") == Some("compact_boundary")
}

/// A message's content, when the message is an object.
fn content_of(row: &Object) -> Option<&Value> {
    row.get("message")
        .and_then(|message| message.get("content"))
}

/// Content parts that are objects or lists.
fn parts_of(row: &Object) -> Vec<&Value> {
    match content_of(row) {
        Some(Value::Array(parts)) => parts
            .iter()
            .filter(|part| matches!(part, Value::Object(_) | Value::Array(_)))
            .collect(),
        _ => Vec::new(),
    }
}

fn part_ids<'a>(row: &'a Object, kind: &str, key: &str) -> Vec<&'a str> {
    parts_of(row)
        .into_iter()
        .filter(|part| part.get("type").and_then(Value::as_str) == Some(kind))
        .filter_map(|part| part.get(key).and_then(Value::as_str))
        .collect()
}

/// Tool call ids a conversation row opens.
fn opened_by(row: &Object) -> Vec<&str> {
    if text_of_key(row, "type") == Some("assistant") {
        part_ids(row, "tool_use", "id")
    } else {
        Vec::new()
    }
}

/// Tool call ids a conversation row answers.
fn answered_by(row: &Object) -> Vec<&str> {
    if text_of_key(row, "type") == Some("user") {
        part_ids(row, "tool_result", "tool_use_id")
    } else {
        Vec::new()
    }
}

fn text_of(row: &Object) -> Option<String> {
    if let Some(Value::String(content)) = content_of(row) {
        return Some(content.clone());
    }
    let parts = parts_of(row);
    let answers = |part: &&Value| part.get("type").and_then(Value::as_str) == Some("tool_result");
    if parts.is_empty() || parts.iter().any(answers) {
        return None;
    }
    let texts: Vec<&str> = parts
        .iter()
        .map(|part| part.get("text").and_then(Value::as_str).unwrap_or(""))
        .collect();
    Some(texts.join("\n"))
}

/// A message the CLI took in while a turn ran: its text.
fn absorbed_text(row: &Object) -> Option<String> {
    if text_of_key(row, "type") != Some("attachment") {
        return None;
    }
    let held = row.get("attachment")?;
    if held.get("type").and_then(Value::as_str) != Some("queued_command") {
        return None;
    }
    held.get("prompt")
        .and_then(Value::as_str)
        .map(str::to_owned)
}

fn points_of(row: &Object) -> Vec<String> {
    POINTER_KEYS
        .iter()
        .filter_map(|key| text_of_key(row, key))
        .map(str::to_owned)
        .collect()
}

/// Every required key, and nothing outside the required and the allowed.
fn keys_within(object: Option<&Object>, required: &[&str], allowed: &[&str]) -> bool {
    let Some(object) = object else {
        return false;
    };
    required.iter().all(|key| object.get(key).is_some())
        && object
            .keys()
            .all(|key| required.contains(&key) || allowed.contains(&key))
}

fn counted(value: Option<&Value>) -> Option<f64> {
    value
        .and_then(Value::as_f64)
        .filter(|number| number.is_finite())
}

/// A rough count: the CLI reports these numbers and sums them.
fn tokens_of(bytes: usize) -> f64 {
    // Rounds a tie up, as `Math.round` does.
    ((bytes + 2) / 4) as f64
}

/// Whether a resume reads the row.
fn writes(row: &RowMeta) -> bool {
    !SKIPPED_TYPES.contains(&row.kind.as_str())
}

/// Written rows whose pointers all land on a held row.
fn resolving<'a>(range: &'a [RowMeta], held: &HashSet<&str>) -> Vec<&'a RowMeta> {
    range
        .iter()
        .filter(|row| writes(row) && row.points.iter().all(|id| held.contains(id.as_str())))
        .collect()
}

fn size_of<'a>(range: impl IntoIterator<Item = &'a RowMeta>) -> usize {
    range.into_iter().map(|row| row.bytes).sum()
}

/// The end of the JSON string that opens at `from`.
fn string_end(bytes: &[u8], from: usize) -> usize {
    let mut at = from + 1;
    while at < bytes.len() && bytes[at] != b'"' {
        at += if bytes[at] == b'\\' { 2 } else { 1 };
    }
    at
}

/// A row's text with its own session ids changed and nothing else: only top-level keys, found by depth,
/// so an id inside a tool input or a path is left alone.
pub fn rehomed(text: &str, session_id: &str) -> String {
    let bytes = text.as_bytes();
    let within = |at: usize| at.min(bytes.len());
    let mut spans: Vec<(usize, usize)> = Vec::new();
    let mut depth: i64 = 0;
    let mut at = 0;
    while at < bytes.len() {
        match bytes[at] {
            b'{' | b'[' => depth += 1,
            b'}' | b']' => depth -= 1,
            b'"' => {
                let end = string_end(bytes, at);
                let key = &bytes[within(at + 1)..within(end)];
                at = end;
                let names_session = SESSION_KEYS.iter().any(|held| held.as_bytes() == key);
                if depth == 1
                    && names_session
                    && bytes.get(end + 1) == Some(&b':')
                    && bytes.get(end + 2) == Some(&b'"')
                {
                    let close = string_end(bytes, end + 2);
                    spans.push((end + 2, within(close + 1)));
                    at = close;
                }
            }
            _ => {}
        }
        at += 1;
    }
    let quoted = quote_json(session_id);
    let mut next = String::with_capacity(text.len() + spans.len() * quoted.len());
    let mut from = 0;
    for (start, end) in spans {
        next.push_str(&text[from..start]);
        next.push_str(&quoted);
        from = end;
    }
    next.push_str(&text[from..]);
    next
}

/// One pass over the file: the live boundary, the newest header rows, and the rows after it.
pub fn index_session<I>(lines: I) -> io::Result<SessionIndex>
where
    I: IntoIterator<Item = io::Result<String>>,
{
    let mut header: Vec<(String, usize)> = Vec::new();
    // Each row's line, and its parent when that is an id.
    let mut segment: HashMap<String, (usize, Option<String>)> = HashMap::new();
    let mut tail: Vec<RowMeta> = Vec::new();
    let mut boundary: Option<BoundaryMeta> = None;
    let mut boundaries = 0;
    let mut standing: Option<String> = None;
    let mut open: HashSet<String> = HashSet::new();
    let mut version: Option<String> = None;
    let mut unparsed = 0;
    let mut unreadable: Vec<usize> = Vec::new();
    let mut line = 0;

    for text in lines {
        let text = text?;
        line += 1;
        if text.is_empty() {
            continue;
        }
        let Some(row) = parse(&text) else {
            unparsed += 1;
            unreadable.push(line);
            continue;
        };
        let kind = to_js_string(row.get("type"));
        if HEADER_TYPES.contains(&kind.as_str()) {
            match header.iter_mut().find(|(held, _)| *held == kind) {
                Some(entry) => entry.1 = line,
                None => header.push((kind.clone(), line)),
            }
        }
        if let Some(held) = text_of_key(&row, "version") {
            version = Some(held.to_owned());
        }
        let uuid = text_of_key(&row, "uuid");

        if is_boundary(&row) {
            let meta = row.get("compactMetadata").filter(|held| !held.is_null());
            let kept = meta
                .and_then(|held| held.get("preservedSegment"))
                .filter(|held| !held.is_null());
            let uuids = meta
                .and_then(|held| held.get("preservedMessages"))
                .and_then(|held| held.get("uuids"))
                .filter(|held| !held.is_null());
            let listed: Option<Vec<String>> = match uuids {
                None => Some(Vec::new()),
                Some(Value::Array(ids)) => ids
                    .iter()
                    .map(|id| id.as_str().map(str::to_owned))
                    .collect(),
                Some(_) => None,
            };
            let preserved = listed.clone().unwrap_or_default();
            let dropped = counted(meta.and_then(|held| held.get("cumulativeDroppedTokens")));
            let standing_tokens = counted(meta.and_then(|held| held.get("postTokens")));
            let head = kept
                .and_then(|held| held.get("headUuid"))
                .and_then(Value::as_str);

            let mut walk: HashSet<&str> = HashSet::new();
            let mut id = kept
                .and_then(|held| held.get("tailUuid"))
                .and_then(Value::as_str);
            while let Some(held) = id {
                if !walk.insert(held) || Some(held) == head {
                    break;
                }
                id = segment.get(held).and_then(|(_, parent)| parent.as_deref());
            }
            boundary = Some(BoundaryMeta {
                line,
                uuid: uuid.unwrap_or("").to_owned(),
                preserved_lines: preserved
                    .iter()
                    .filter_map(|id| segment.get(id).map(|(at, _)| *at))
                    .collect(),
                preserved_sound: preserved
                    .iter()
                    .all(|id| segment.contains_key(id) && walk.contains(id.as_str()))
                    && walk.len() == preserved.len()
                    && preserved.iter().collect::<HashSet<_>>().len() == preserved.len(),
                preserved,
                summary_line: None,
                cumulative_dropped_tokens: dropped.unwrap_or(0.0),
                post_tokens: standing_tokens.unwrap_or(0.0),
                known: keys_within(Some(&row), BOUNDARY_KEYS, BOUNDARY_EXTRA_KEYS)
                    && keys_within(
                        meta.and_then(Value::as_object),
                        METADATA_KEYS,
                        METADATA_EXTRA_KEYS,
                    )
                    && listed.is_some()
                    && dropped.is_some()
                    && standing_tokens.is_some(),
            });
            boundaries += 1;
            segment = HashMap::new();
            tail = Vec::new();
            open = HashSet::new();
            standing = None;
            unreadable = Vec::new();
        }
        if let Some(held) = uuid.filter(|held| !held.is_empty()) {
            let parent = text_of_key(&row, "parentUuid").map(str::to_owned);
            segment.insert(held.to_owned(), (line, parent));
        }
        if let Some(live) = boundary.as_mut()
            && live.summary_line.is_none()
            && is_true(&row, "isCompactSummary")
        {
            live.summary_line = Some(line);
            standing = text_of(&row);
        }

        let open_tools = open.len();
        for id in opened_by(&row) {
            open.insert(id.to_owned());
        }
        for id in answered_by(&row) {
            open.remove(id);
        }
        let opens = if kind == "user"
            && !is_true(&row, "isCompactSummary")
            && !is_true(&row, "isSidechain")
        {
            text_of(&row)
        } else {
            None
        };
        let message = opens.or_else(|| absorbed_text(&row));
        tail.push(RowMeta {
            line,
            bytes: text.len() + 1,
            kind,
            uuid: uuid.filter(|held| !held.is_empty()).map(str::to_owned),
            at: text_of_key(&row, "timestamp").map(str::to_owned),
            open_tools,
            delivered: message.is_some(),
            text: message,
            points: points_of(&row),
        });
    }
    tail.push(RowMeta {
        line: line + 1,
        bytes: 0,
        kind: "end".to_owned(),
        uuid: None,
        at: None,
        open_tools: open.len(),
        delivered: false,
        text: None,
        points: Vec::new(),
    });
    Ok(SessionIndex {
        lines: line,
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

/// What a cut keeps and drops, or why it is refused. Writes nothing.
pub fn plan_cut(
    index: &SessionIndex,
    request: &CutRequest,
    cli_version: &str,
) -> Result<CutPlan, CutRefused> {
    let Some((end, rows)) = index.tail.split_last().filter(|(_, rows)| !rows.is_empty()) else {
        return Err(refused(CutRefusal::NoRows, "The session file is empty."));
    };
    let proven = |version: &str| CUT_CLI_VERSIONS.contains(&version);
    if !proven(cli_version) {
        return Err(refused(
            CutRefusal::CliVersion,
            &format!(
                "Claude Code {cli_version} is installed, which compacting does not support yet."
            ),
        ));
    }
    if let Some(version) = index.version.as_deref().filter(|version| !proven(version)) {
        return Err(refused(
            CutRefusal::CliVersion,
            &format!(
                "The session file was written by Claude Code {version}, which compacting does not support yet."
            ),
        ));
    }
    let live = index.boundary.as_ref();
    if live.is_some_and(|live| !live.known) {
        return Err(refused(
            CutRefusal::BoundaryShape,
            "The session's last compaction is in a format compacting does not support.",
        ));
    }
    let unreadable = |from: usize, to: usize| -> Result<(), CutRefused> {
        if index
            .unreadable
            .iter()
            .any(|line| *line >= from && *line <= to)
        {
            return Err(refused(
                CutRefusal::UnreadableRows,
                "The session file has a damaged row after this message.",
            ));
        }
        Ok(())
    };
    let mut header_lines: Vec<usize> = index.header.iter().map(|(_, line)| *line).collect();
    header_lines.sort_unstable();

    if request.mode == CutMode::Trim {
        let Some(live) = live else {
            return Err(refused(
                CutRefusal::NothingToTrim,
                "This session has never compacted.",
            ));
        };
        if !live.preserved_sound {
            return Err(refused(
                CutRefusal::PreservedUnresolved,
                "Messages the last compaction kept are missing from the session file.",
            ));
        }
        unreadable(live.line, end.line)?;
        let held: HashSet<&str> = live
            .preserved
            .iter()
            .map(String::as_str)
            .chain(rows.iter().filter_map(|row| row.uuid.as_deref()))
            .collect();
        let kept = resolving(rows, &held);
        let mut kept_lines: Vec<usize> = live
            .preserved_lines
            .iter()
            .copied()
            .chain(kept.iter().map(|row| row.line))
            .collect();
        kept_lines.sort_unstable();
        return Ok(CutPlan {
            mode: CutMode::Trim,
            header_lines,
            kept_lines,
            preserved: None,
            fold_lines: Vec::new(),
            kept_bytes: size_of(kept),
            dropped_bytes: 0,
            dropped_rows: (live.line - 1).saturating_sub(live.preserved_lines.len()),
            from: None,
        });
    }

    let target = request.target_uuid.as_deref();
    let Some(at) = rows
        .iter()
        .position(|row| row.uuid.is_some() && row.uuid.as_deref() == target)
    else {
        return Err(match live {
            Some(_) => refused(
                CutRefusal::TargetCompacted,
                "This message is from before the session's last compaction.",
            ),
            None => refused(
                CutRefusal::TargetMissing,
                "This message is not in the session file.",
            ),
        });
    };
    let target = &rows[at];
    if !target.delivered {
        return Err(refused(
            CutRefusal::TargetNotTurnStart,
            "This row is not a message the session received.",
        ));
    }
    if target.open_tools > 0 {
        return Err(refused(
            CutRefusal::TargetMidTool,
            "A tool call above this message is still open, so the session cannot be cut here.",
        ));
    }
    let mut through = Some(rows.len() - 1);
    if let Some(marker) = request.end_before.as_deref() {
        let asks = |row: &RowMeta| {
            row.delivered
                && row
                    .text
                    .as_deref()
                    .is_some_and(|text| text.contains(marker))
        };
        let Some(stop) = rows
            .iter()
            .enumerate()
            .position(|(i, row)| i > at && asks(row))
        else {
            return Err(refused(
                CutRefusal::EndMissing,
                "The compact request is not in the session file.",
            ));
        };
        if rows[stop + 1..].iter().any(|row| row.delivered) {
            return Err(refused(
                CutRefusal::LateMessage,
                "A message reached the session while it wrote its summary, and it would be lost.",
            ));
        }
        through = stop.checked_sub(1);
    } else if let Some(wanted) = request.through_uuid.as_deref() {
        through = rows
            .iter()
            .position(|row| row.uuid.as_deref() == Some(wanted));
    }
    let Some(through) = through.filter(|through| *through >= at) else {
        return Err(refused(
            CutRefusal::TargetMissing,
            "The kept part would end above where it starts.",
        ));
    };
    unreadable(target.line, rows[through].line)?;
    if !request.live_tail && rows.get(through + 1).unwrap_or(end).open_tools > 0 {
        return Err(refused(
            CutRefusal::TailMidTool,
            "The kept part would end inside an open tool call.",
        ));
    }
    let after_summary = live.and_then(|live| live.summary_line).unwrap_or(0);
    let above: Vec<&RowMeta> = rows[..at]
        .iter()
        .filter(|row| row.line > after_summary && row.uuid.is_some())
        .collect();
    if above.is_empty() {
        return Err(refused(
            CutRefusal::NothingAbove,
            "There is nothing above this message to compact.",
        ));
    }

    let range = &rows[at..=through];
    let chain: Vec<&str> = range.iter().filter_map(|row| row.uuid.as_deref()).collect();
    let kept = resolving(range, &chain.iter().copied().collect());
    let kept_bytes = size_of(kept.iter().copied());
    let preserved_lines = live
        .map(|live| live.preserved_lines.as_slice())
        .unwrap_or(&[]);
    Ok(CutPlan {
        mode: request.mode,
        header_lines,
        preserved: Some(Preserved {
            head: chain.first().copied().unwrap_or("").to_owned(),
            tail: chain.last().copied().unwrap_or("").to_owned(),
            uuids: chain.iter().map(|id| (*id).to_owned()).collect(),
        }),
        fold_lines: preserved_lines
            .iter()
            .copied()
            .chain(above.iter().filter(|row| writes(row)).map(|row| row.line))
            .collect(),
        kept_bytes,
        dropped_bytes: size_of(rows) - kept_bytes,
        dropped_rows: above.len() + preserved_lines.len(),
        from: Some(KeptFrom {
            at: target.at.clone(),
            text: target.text.clone().unwrap_or_default(),
        }),
        kept_lines: kept.iter().map(|row| row.line).collect(),
    })
}

/// The new file's rows: the header, the kept rows, then for a clear or compact a built boundary and summary.
pub fn build_cut<I>(
    plan: &CutPlan,
    index: &SessionIndex,
    lines: I,
    inputs: &BuildInputs<'_>,
) -> io::Result<Vec<String>>
where
    I: IntoIterator<Item = io::Result<String>>,
{
    let header: HashSet<usize> = plan.header_lines.iter().copied().collect();
    let kept: HashSet<usize> = plan.kept_lines.iter().copied().collect();
    let mut headers: Vec<String> = Vec::new();
    let mut body: Vec<String> = Vec::new();
    let mut envelope = Object::new();
    let mut position: Option<Value> = None;
    for (at, text) in lines.into_iter().enumerate() {
        let text = text?;
        let line = at + 1;
        if header.contains(&line) {
            headers.push(rehomed(&text, inputs.session_id));
        }
        if !kept.contains(&line) {
            continue;
        }
        body.push(rehomed(&text, inputs.session_id));
        if let Some(row) = parse(&text) {
            if let Some(held) = row.get("turnPosition") {
                position = Some(held.clone());
            }
            if text_of_key(&row, "uuid").is_some() {
                envelope = row;
            }
        }
    }
    let mut built = headers;
    built.append(&mut body);
    let Some(preserved) = &plan.preserved else {
        return Ok(built);
    };

    let boundary_uuid = (inputs.new_id)();
    let summary_uuid = (inputs.new_id)();
    let at = iso_time((inputs.now_ms)())
        .ok_or_else(|| io::Error::other("the clock is outside the time range"))?;
    let summary_text = inputs.summary_text.unwrap_or("");
    let live = index.boundary.as_ref();
    let before =
        tokens_of(plan.kept_bytes + plan.dropped_bytes) + live.map_or(0.0, |live| live.post_tokens);
    let after = before.min(tokens_of(plan.kept_bytes + summary_text.len()));
    let uuids = || Value::from(preserved.uuids.clone());
    let carry = |row: &mut Object, key: &str| row.insert_some(key, envelope.get(key).cloned());

    let mut segment = Object::new();
    segment.insert("headUuid", Value::from(preserved.head.as_str()));
    segment.insert("anchorUuid", Value::from(summary_uuid.as_str()));
    segment.insert("tailUuid", Value::from(preserved.tail.as_str()));
    let mut messages = Object::new();
    messages.insert("anchorUuid", Value::from(summary_uuid.as_str()));
    messages.insert("uuids", uuids());
    messages.insert("allUuids", uuids());
    let mut meta = Object::new();
    meta.insert("trigger", Value::from("manual"));
    meta.insert("preTokens", Value::Number(before));
    meta.insert("postTokens", Value::Number(after));
    meta.insert(
        "cumulativeDroppedTokens",
        Value::Number(live.map_or(0.0, |live| live.cumulative_dropped_tokens) + before - after),
    );
    meta.insert("durationMs", Value::Number(1.0));
    meta.insert("preservedSegment", Value::Object(segment));
    meta.insert("preservedMessages", Value::Object(messages));

    let mut boundary = Object::new();
    boundary.insert("parentUuid", Value::Null);
    boundary.insert("logicalParentUuid", Value::from(preserved.tail.as_str()));
    boundary.insert("isSidechain", Value::Bool(false));
    boundary.insert("type", Value::from("system"));
    boundary.insert("subtype", Value::from("compact_boundary"));
    boundary.insert("content", Value::from("Conversation compacted"));
    boundary.insert("level", Value::from("info"));
    boundary.insert("compactMetadata", Value::Object(meta));
    boundary.insert("uuid", Value::from(boundary_uuid.as_str()));
    boundary.insert("timestamp", Value::from(at.as_str()));
    carry(&mut boundary, "userType");
    carry(&mut boundary, "entrypoint");
    carry(&mut boundary, "cwd");
    boundary.insert("sessionId", Value::from(inputs.session_id));
    carry(&mut boundary, "version");
    carry(&mut boundary, "gitBranch");
    carry(&mut boundary, "slug");
    built.push(stringify_json(&Value::Object(boundary)));

    let mut message = Object::new();
    message.insert("role", Value::from("user"));
    message.insert("content", Value::from(summary_text));
    let mut summary = Object::new();
    summary.insert("parentUuid", Value::from(boundary_uuid.as_str()));
    summary.insert("isSidechain", Value::Bool(false));
    summary.insert("promptId", Value::from((inputs.new_id)()));
    summary.insert("type", Value::from("user"));
    summary.insert("message", Value::Object(message));
    summary.insert("isVisibleInTranscriptOnly", Value::Bool(true));
    summary.insert("isCompactSummary", Value::Bool(true));
    summary.insert("uuid", Value::from(summary_uuid.as_str()));
    summary.insert("timestamp", Value::from(at.as_str()));
    summary.insert_some("turnPosition", position);
    summary.insert("session_id", Value::from(inputs.session_id));
    carry(&mut summary, "userType");
    carry(&mut summary, "entrypoint");
    carry(&mut summary, "cwd");
    summary.insert("sessionId", Value::from(inputs.session_id));
    carry(&mut summary, "version");
    carry(&mut summary, "gitBranch");
    carry(&mut summary, "slug");
    built.push(stringify_json(&Value::Object(summary)));
    Ok(built)
}

/// `===` on two values that may be absent. Objects and lists are never equal.
fn strict_eq(left: Option<&Value>, right: Option<&Value>) -> bool {
    match (left, right) {
        (None, None) => true,
        (Some(Value::Null), Some(Value::Null)) => true,
        (Some(Value::Bool(a)), Some(Value::Bool(b))) => a == b,
        (Some(Value::Number(a)), Some(Value::Number(b))) => a == b,
        (Some(Value::String(a)), Some(Value::String(b))) => a == b,
        _ => false,
    }
}

/// The read-back: what would stop the built file resuming as planned. Empty when sound.
/// `paired` holds a built segment to its own tool calls; a trim keeps the CLI's own segment as it stood.
pub fn verify_cut<I>(lines: I, session_id: &str, paired: bool) -> io::Result<Vec<String>>
where
    I: IntoIterator<Item = io::Result<String>>,
{
    let mut problems: Vec<String> = Vec::new();
    let mut seen: HashMap<String, usize> = HashMap::new();
    let mut pointers: Vec<(usize, String)> = Vec::new();
    let mut boundary: Option<(usize, Object)> = None;
    let mut after_boundary: Option<Object> = None;
    let mut calls: HashSet<String> = HashSet::new();
    for (at, text) in lines.into_iter().enumerate() {
        let text = text?;
        let line = at + 1;
        let Some(row) = parse(&text) else {
            problems.push(format!("line {line} does not parse"));
            continue;
        };
        if SKIPPED_TYPES.contains(&to_js_string(row.get("type")).as_str()) {
            problems.push(format!("line {line} is a request log row"));
        }
        for key in SESSION_KEYS {
            if text_of_key(&row, key).is_some_and(|held| held != session_id) {
                problems.push(format!("line {line} names another session"));
            }
        }
        pointers.extend(points_of(&row).into_iter().map(|id| (line, id)));
        let follows = boundary.as_ref().is_some_and(|(held, _)| line == held + 1);
        let bounds = is_boundary(&row);
        if bounds && boundary.is_some() {
            problems.push(format!("line {line} is a second compaction row"));
        }
        if let Some(uuid) = text_of_key(&row, "uuid") {
            seen.insert(uuid.to_owned(), line);
        }
        for id in opened_by(&row) {
            calls.insert(id.to_owned());
        }
        if paired && answered_by(&row).iter().any(|id| !calls.contains(*id)) {
            problems.push(format!(
                "line {line} answers a call that is not in the file"
            ));
        }
        if bounds {
            boundary = Some((line, row));
            after_boundary = None;
        } else if follows {
            after_boundary = Some(row);
        }
    }
    for (line, id) in &pointers {
        if !seen.contains_key(id) {
            problems.push(format!("line {line} names a row that is not in the file"));
        }
    }
    let Some((boundary_line, boundary)) = boundary else {
        problems.push("no compaction row".to_owned());
        return Ok(problems);
    };

    let meta = boundary
        .get("compactMetadata")
        .filter(|held| !held.is_null());
    let segment = meta
        .and_then(|held| held.get("preservedSegment"))
        .filter(|held| !held.is_null());
    let messages = meta
        .and_then(|held| held.get("preservedMessages"))
        .filter(|held| !held.is_null());
    let uuids: &[Value] = match messages
        .and_then(|held| held.get("uuids"))
        .filter(|held| !held.is_null())
    {
        None => &[],
        Some(Value::Array(ids)) => ids,
        Some(_) => {
            problems.push("the preserved rows are not a list".to_owned());
            &[]
        }
    };
    let places: Vec<Option<usize>> = uuids
        .iter()
        .map(|id| id.as_str().and_then(|id| seen.get(id).copied()))
        .collect();
    if places
        .iter()
        .any(|at| at.is_none_or(|at| at > boundary_line))
    {
        problems.push("a preserved row is missing or after the compaction row".to_owned());
    }
    if places
        .windows(2)
        .any(|pair| pair[1].unwrap_or(0) < pair[0].unwrap_or(0))
    {
        problems.push("preserved rows are not in conversation order".to_owned());
    }
    let end_of = |key: &str| segment.and_then(|held| held.get(key));
    if !uuids.is_empty()
        && (!strict_eq(end_of("headUuid"), uuids.first())
            || !strict_eq(end_of("tailUuid"), uuids.last()))
    {
        problems.push("the preserved segment's ends are not its rows' ends".to_owned());
    }
    if !boundary.get("parentUuid").is_some_and(Value::is_null) {
        problems.push("the compaction row has a parent".to_owned());
    }
    let summary = after_boundary.as_ref().filter(|row| {
        is_true(row, "isCompactSummary") && strict_eq(row.get("parentUuid"), boundary.get("uuid"))
    });
    match summary {
        None => problems.push("no summary follows the compaction row".to_owned()),
        Some(summary) => {
            let uuid = summary.get("uuid");
            let anchor = messages.and_then(|held| held.get("anchorUuid"));
            if !strict_eq(uuid, end_of("anchorUuid")) || !strict_eq(uuid, anchor) {
                problems.push("the summary is not the compaction row's anchor".to_owned());
            }
        }
    }
    Ok(problems)
}

/// What `\s` matches in ECMAScript, and what `trim` removes.
fn is_js_space(char: char) -> bool {
    matches!(
        char,
        '\t' | '\n' | '\u{B}' | '\u{C}' | '\r' | ' ' | '\u{A0}' | '\u{1680}' | '\u{2000}'
            ..='\u{200A}'
                | '\u{2028}'
                | '\u{2029}'
                | '\u{202F}'
                | '\u{205F}'
                | '\u{3000}'
                | '\u{FEFF}'
    )
}

fn js_trim(text: &str) -> &str {
    text.trim_matches(is_js_space)
}

/// Runs of space become one; the ends are trimmed.
fn collapsed(text: &str) -> String {
    text.split(is_js_space)
        .filter(|word| !word.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// The turn a chat row names. Its own text finds it, nearest in time; a row that started no turn finds the
/// turn start at or before it. `body_of` reads a turn start's text as the message it carried.
pub fn pick_turn(
    index: &SessionIndex,
    pick: &TurnPick,
    body_of: &dyn Fn(&str) -> String,
) -> Option<PickedTurn> {
    let starts: Vec<(&RowMeta, &str)> = index
        .tail
        .iter()
        .filter(|row| row.delivered && row.open_tools == 0)
        .filter_map(|row| row.uuid.as_deref().map(|uuid| (row, uuid)))
        .collect();
    let time_of = |row: &RowMeta| row.at.as_deref().and_then(parse_iso_time);
    if let Some(wanted) = pick
        .text
        .as_deref()
        .map(collapsed)
        .filter(|wanted| !wanted.is_empty())
    {
        let named = starts
            .iter()
            .filter(|(row, _)| collapsed(&body_of(row.text.as_deref().unwrap_or(""))) == wanted)
            .filter_map(|(row, uuid)| time_of(row).map(|at| (at.abs_diff(pick.at), *uuid)))
            .min_by_key(|(gap, _)| *gap);
        if let Some((_, uuid)) = named {
            return Some(PickedTurn {
                uuid: uuid.to_owned(),
                exact: true,
            });
        }
    }
    // A sent message's turn starts after it.
    let slack = if pick.text.is_some() {
        PICK_SLACK_MS
    } else {
        0
    };
    starts
        .iter()
        .rfind(|(row, _)| time_of(row).is_some_and(|at| at <= pick.at.saturating_add(slack)))
        .map(|(_, uuid)| PickedTurn {
            uuid: (*uuid).to_owned(),
            exact: false,
        })
}

/// A compact's summary row: the summary, framed as the CLI frames its own.
pub fn compacted_summary(summary: &str, source_path: &str) -> String {
    format!(
        "This session is being continued from an earlier part of this conversation, which the owner compacted \
         above a message they chose. The summary below covers everything before that message.\n\n{}\n\nIf you \
         need specific details from before the compaction, read the full transcript at: {source_path}\n\nThe \
         messages after this summary are the conversation from that message on, kept verbatim.",
        js_trim(summary)
    )
}

/// A clear's summary: what stood, then one line saying rows were cleared and where they still are.
pub fn cleared_summary(standing: Option<&str>, plan: &CutPlan, source_path: &str) -> String {
    let note = format!(
        "[The owner cleared {} earlier rows of this conversation without summarizing them. They remain in the \
         transcript at: {source_path}]",
        plan.dropped_rows
    );
    match standing.filter(|standing| !standing.is_empty()) {
        Some(standing) => format!("{standing}\n\n{note}"),
        None => note,
    }
}

/// The whole cut over its ports. The caller closes the CLI first and relaunches it after.
pub fn cut_session(ports: &dyn CutPorts, request: &CutRequest) -> Result<CutOutcome, CutError> {
    let index = index_session(ports.read()?)?;
    let plan = match plan_cut(&index, request, &ports.cli_version()?) {
        Ok(plan) => plan,
        Err(refusal) => return Ok(Err(refusal)),
    };

    let mut summary_text: Option<String> = None;
    if plan.mode == CutMode::Clear {
        summary_text = Some(cleared_summary(
            index.standing.as_deref(),
            &plan,
            &ports.source_path(),
        ));
    }
    if plan.mode == CutMode::Compact {
        let summary = match &request.summary_text {
            Some(summary) => summary.clone(),
            None => {
                if !ports.can_summarize() {
                    return Ok(Err(refused(
                        CutRefusal::SummaryNeeded,
                        "No summary was handed over.",
                    )));
                }
                let fold: HashSet<usize> = plan.fold_lines.iter().copied().collect();
                let mut rows: Vec<String> = Vec::new();
                for (at, text) in ports.read()?.enumerate() {
                    let text = text?;
                    if fold.contains(&(at + 1)) {
                        rows.push(text);
                    }
                }
                ports.summarize(index.standing.as_deref(), &rows)?
            }
        };
        if js_trim(&summary).is_empty() {
            return Ok(Err(refused(
                CutRefusal::SummaryNeeded,
                "The summary is empty.",
            )));
        }
        summary_text = Some(compacted_summary(&summary, &ports.source_path()));
    }

    let session_id = ports.new_id();
    let built = build_cut(
        &plan,
        &index,
        ports.read()?,
        &BuildInputs {
            session_id: &session_id,
            summary_text: summary_text.as_deref(),
            new_id: &|| ports.new_id(),
            now_ms: &|| ports.now_ms(),
        },
    )?;
    ports.create(&session_id, &built)?;
    let problems = verify_cut(
        ports.read_built(&session_id)?,
        &session_id,
        plan.preserved.is_some(),
    )?;
    if !problems.is_empty() {
        ports.discard(&session_id)?;
        return Err(CutError::ReadBack(problems));
    }
    Ok(Ok(CutDone { session_id, plan }))
}
