//! Synthetic Claude session tests.

use nyaaai_contract::json::{Object, Value, stringify_json};
use nyaaai_contract::time::iso_time;

pub const SESSION_ID: &str = "11111111-1111-4111-8111-111111111111";
pub const CLI_VERSION: &str = "2.1.323";
/// Fixed fixture timestamp.
const START_MS: i64 = 1_794_528_000_000;

fn object<const N: usize>(pairs: [(&str, Value); N]) -> Object {
    let mut held = Object::new();
    for (key, value) in pairs {
        held.insert(key, value);
    }
    held
}

fn message(role: &str, content: Value) -> Value {
    Value::Object(object([("role", Value::from(role)), ("content", content)]))
}

/// Shared chain and compaction fields.
fn envelope(row: &mut Object) {
    row.insert("userType", Value::from("external"));
    row.insert("entrypoint", Value::from("cli"));
    row.insert("cwd", Value::from("/work"));
    row.insert("sessionId", Value::from(SESSION_ID));
    row.insert("version", Value::from(CLI_VERSION));
    row.insert("gitBranch", Value::from("main"));
    row.insert("slug", Value::from("a-slug"));
}

#[derive(Debug, Default)]
pub struct SyntheticSession {
    pub lines: Vec<String>,
    last: Option<String>,
    count: i64,
}

impl SyntheticSession {
    pub fn new() -> Self {
        Self::default()
    }

    /// LF-terminated session bytes.
    pub fn bytes(&self) -> Vec<u8> {
        self.lines
            .iter()
            .flat_map(|line| line.bytes().chain(*b"\n"))
            .collect()
    }

    /// Defaults uuid to `u<n>`.
    pub fn chain(&mut self, row: Object, id: Option<&str>) -> String {
        let uuid = match id {
            Some(id) => id.to_owned(),
            None => {
                self.count += 1;
                format!("u{}", self.count)
            }
        };
        let mut held = object([
            ("parentUuid", Value::from(self.last.clone())),
            ("isSidechain", Value::Bool(false)),
        ]);
        for (key, value) in row.iter() {
            held.insert(key, value.clone());
        }
        held.insert("uuid", Value::from(uuid.as_str()));
        held.insert(
            "timestamp",
            Value::from(iso_time(START_MS + self.count * 1000)),
        );
        envelope(&mut held);
        self.lines.push(stringify_json(&Value::Object(held)));
        self.last = Some(uuid.clone());
        uuid
    }

    /// Appends one raw line.
    pub fn raw(&mut self, line: &str) {
        self.lines.push(line.to_owned());
    }

    pub fn header(&mut self) {
        for kind in ["mode", "permission-mode", "session-owner", "atis-latch"] {
            let row = object([
                ("type", Value::from(kind)),
                ("sessionId", Value::from(SESSION_ID)),
            ]);
            self.lines.push(stringify_json(&Value::Object(row)));
        }
    }

    pub fn user(&mut self, text: &str) -> String {
        self.chain(
            object([
                ("type", Value::from("user")),
                ("message", message("user", Value::from(text))),
            ]),
            None,
        )
    }

    pub fn says(&mut self, text: &str) -> String {
        let part = object([("type", Value::from("text")), ("text", Value::from(text))]);
        let content = Value::Array(vec![Value::Object(part)]);
        self.chain(
            object([
                ("type", Value::from("assistant")),
                ("message", message("assistant", content)),
            ]),
            None,
        )
    }

    pub fn calls(&mut self, id: &str) -> String {
        let part = object([
            ("type", Value::from("tool_use")),
            ("id", Value::from(id)),
            ("name", Value::from("Bash")),
            (
                "input",
                Value::Object(object([("session_id", Value::from(SESSION_ID))])),
            ),
        ]);
        let content = Value::Array(vec![Value::Object(part)]);
        self.chain(
            object([
                ("type", Value::from("assistant")),
                ("message", message("assistant", content)),
            ]),
            None,
        )
    }

    /// Appends a turn prompt.
    pub fn absorbs(&mut self, prompt: &str) -> String {
        let held = object([
            ("type", Value::from("queued_command")),
            ("prompt", Value::from(prompt)),
        ]);
        self.chain(
            object([
                ("type", Value::from("attachment")),
                ("attachment", Value::Object(held)),
            ]),
            None,
        )
    }

    pub fn answers(&mut self, id: &str) -> String {
        let part = object([
            ("type", Value::from("tool_result")),
            ("tool_use_id", Value::from(id)),
            ("content", Value::from(format!("ran in {SESSION_ID}"))),
        ]);
        let content = Value::Array(vec![Value::Object(part)]);
        self.chain(
            object([
                ("type", Value::from("user")),
                ("message", message("user", content)),
            ]),
            None,
        )
    }

    pub fn request_log(&mut self, id: &str) {
        let part = object([
            ("type", Value::from("tool_use")),
            ("id", Value::from(id)),
            ("name", Value::from("Bash")),
            ("input", Value::Object(Object::new())),
        ]);
        let row = object([
            ("type", Value::from("api-request-blob")),
            ("sessionId", Value::from(SESSION_ID)),
            ("hash", Value::from("h")),
            (
                "message",
                message("assistant", Value::Array(vec![Value::Object(part)])),
            ),
        ]);
        self.lines.push(stringify_json(&Value::Object(row)));
    }

    pub fn last_prompt(&mut self, leaf: &str) {
        let row = object([
            ("type", Value::from("last-prompt")),
            ("leafUuid", Value::from(leaf)),
            ("sessionId", Value::from(SESSION_ID)),
        ]);
        self.lines.push(stringify_json(&Value::Object(row)));
    }

    /// CLI compaction preserving selected rows.
    pub fn compacts(&mut self, summary: &str, preserved: &[&str], dropped: f64) {
        self.count += 1;
        let boundary = format!("b{}", self.count);
        let anchor = format!("s{}", self.count);
        let ids = |extra: &[&str]| -> Value {
            Value::Array(
                preserved
                    .iter()
                    .chain(extra)
                    .map(|id| Value::from(*id))
                    .collect(),
            )
        };
        let end = |id: Option<&&str>| id.map(|id| Value::from(*id));
        let mut segment = Object::new();
        segment.insert_some("headUuid", end(preserved.first()));
        segment.insert("anchorUuid", Value::from(anchor.as_str()));
        segment.insert_some("tailUuid", end(preserved.last()));
        let meta = object([
            ("trigger", Value::from("manual")),
            ("preTokens", Value::Number(5000.0)),
            ("postTokens", Value::Number(400.0)),
            ("cumulativeDroppedTokens", Value::Number(dropped)),
            ("durationMs", Value::Number(9.0)),
            ("preservedSegment", Value::Object(segment)),
            (
                "preservedMessages",
                Value::Object(object([
                    ("anchorUuid", Value::from(anchor.as_str())),
                    ("uuids", ids(&[])),
                    ("allUuids", ids(&["gone"])),
                ])),
            ),
        ]);
        let mut row = object([
            ("parentUuid", Value::Null),
            ("logicalParentUuid", Value::from(self.last.clone())),
            ("isSidechain", Value::Bool(false)),
            ("type", Value::from("system")),
            ("subtype", Value::from("compact_boundary")),
            ("content", Value::from("Conversation compacted")),
            ("level", Value::from("info")),
            ("compactMetadata", Value::Object(meta)),
            ("uuid", Value::from(boundary.as_str())),
            ("timestamp", Value::from("2026-11-13T00:01:00.000Z")),
        ]);
        envelope(&mut row);
        self.lines.push(stringify_json(&Value::Object(row)));
        self.last = Some(boundary);
        self.chain(
            object([
                ("type", Value::from("user")),
                ("message", message("user", Value::from(summary))),
                ("isCompactSummary", Value::Bool(true)),
                ("isVisibleInTranscriptOnly", Value::Bool(true)),
            ]),
            Some(&anchor),
        );
    }
}

/// Compacted fixture and named rows.
#[derive(Debug)]
pub struct CompactedSession {
    pub session: SyntheticSession,
    pub second: String,
    pub third: String,
    pub mid_turn: String,
    pub kept_a: String,
}

/// Three post-compaction turns.
pub fn compacted_session() -> CompactedSession {
    let mut session = SyntheticSession::new();
    session.header();
    session.user("ancient");
    session.says("ancient answer");
    let kept_a = session.user("kept before the compaction");
    let kept_b = session.says("kept answer");
    session.compacts("what stood", &[&kept_a, &kept_b], 1000.0);
    session.user("first turn");
    session.calls("t1");
    session.request_log("ghost");
    session.answers("t1");
    session.says("done");
    let second = session.user("second turn");
    session.last_prompt(&second);
    session.calls("t2");
    session.answers("t2");
    let mid_turn = session.user("typed while it worked");
    session.says("both done");
    let third = session.user("third turn");
    session.header();
    let end = session.says("finished");
    session.last_prompt(&end);
    CompactedSession {
        session,
        second,
        third,
        mid_turn,
        kept_a,
    }
}
