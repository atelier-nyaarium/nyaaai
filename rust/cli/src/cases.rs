//! Rust case runner. Gate only.

mod render;

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::rc::Rc;

use nyaaai_contract::id::random_uuid;
use nyaaai_contract::json::{Object, Value, parse_json, stringify_json};
use nyaaai_contract::time::{iso_time, parse_iso_time};
use nyaaai_operator::claude::cli_version::version_in;
use nyaaai_operator::claude::session_cut::{
    BoundaryMeta, CutError, CutMode, CutPorts, CutRequest, Lines, RowMeta, SessionIndex,
    Summarizer, TurnPick, cut_session, index_session, pick_turn, plan_cut, rehomed, verify_cut,
};
use nyaaai_operator::claude::session_file::{lines_of, session_lines};
use nyaaai_process::paths::is_inside;
use nyaaai_testkit::claude_session::compacted_session;

use render::{object, plan_json, refused_json};

type Outcome = Result<Value, String>;

fn main() -> ExitCode {
    let paths: Vec<String> = std::env::args().skip(1).collect();
    match run(&paths) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

fn run(paths: &[String]) -> Result<(), String> {
    let mut files = Vec::new();
    for path in paths {
        collect(Path::new(path), &mut files).map_err(|error| format!("{path}: {error}"))?;
    }
    files.sort();
    for file in files {
        let mut line = Object::new();
        line.insert("file", Value::from(file.to_string_lossy().into_owned()));
        match run_file(&file) {
            Ok(actual) => line.insert("actual", actual),
            Err(error) => line.insert("error", Value::from(error)),
        }
        println!("{}", stringify_json(&Value::Object(line)));
    }
    Ok(())
}

fn collect(path: &Path, files: &mut Vec<PathBuf>) -> io::Result<()> {
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            collect(&entry?.path(), files)?;
        }
    } else if path
        .extension()
        .is_some_and(|extension| extension == "json")
    {
        files.push(path.to_owned());
    }
    Ok(())
}

fn run_file(file: &Path) -> Outcome {
    let text = fs::read_to_string(file).map_err(|error| error.to_string())?;
    let case = parse_json(&text).ok_or("the case file does not parse")?;
    let kind = case
        .get("kind")
        .and_then(Value::as_str)
        .ok_or("the case names no kind")?;
    let input = case.get("input").ok_or("the case holds no input")?;
    match kind {
        "json.roundTrip" => json_round_trip(input),
        "time.parseIso" => time_parse_iso(input),
        "time.iso" => time_iso(input),
        "sessionCut.rehomed" => cut_rehomed(input),
        "sessionCut.index" => cut_index(input),
        "sessionCut.plan" => cut_plan(input),
        "sessionCut.cut" => cut_cut(input),
        "sessionCut.verify" => cut_verify(input),
        "sessionCut.pickTurn" => cut_pick_turn(input),
        "sessionFile.linesOf" => file_lines_of(input),
        "claude.versionIn" => claude_version_in(input),
        "paths.isInside" => paths_is_inside(input),
        "testkit.compactedSession" => Ok(object(vec![(
            "lines",
            Value::from(compacted_session().session.lines),
        )])),
        other => Err(format!("unknown kind {other}")),
    }
}

fn text_at<'a>(input: &'a Value, key: &str) -> Result<&'a str, String> {
    input
        .get(key)
        .and_then(Value::as_str)
        .ok_or(format!("input.{key} is not text"))
}

fn list_at<'a>(input: &'a Value, key: &str) -> Result<&'a [Value], String> {
    input
        .get(key)
        .and_then(Value::as_array)
        .ok_or(format!("input.{key} is not a list"))
}

fn texts_at(input: &Value, key: &str) -> Result<Vec<String>, String> {
    list_at(input, key)?
        .iter()
        .map(|text| {
            text.as_str()
                .map(str::to_owned)
                .ok_or(format!("input.{key} holds a value that is not text"))
        })
        .collect()
}

fn optional_text(input: &Value, key: &str) -> Option<String> {
    input.get(key).and_then(Value::as_str).map(str::to_owned)
}

fn json_round_trip(input: &Value) -> Outcome {
    let results = texts_at(input, "texts")?
        .iter()
        .map(|text| match parse_json(text) {
            Some(value) => object(vec![
                ("ok", Value::Bool(true)),
                ("text", Value::from(stringify_json(&value))),
            ]),
            None => object(vec![("ok", Value::Bool(false))]),
        })
        .collect();
    Ok(object(vec![("results", Value::Array(results))]))
}

fn time_parse_iso(input: &Value) -> Outcome {
    let results = texts_at(input, "texts")?
        .iter()
        .map(|text| parse_iso_time(text).map_or(Value::Null, |at| Value::Number(at as f64)))
        .collect();
    Ok(object(vec![("results", Value::Array(results))]))
}

fn time_iso(input: &Value) -> Outcome {
    let texts = list_at(input, "times")?
        .iter()
        .map(|time| {
            time.as_f64()
                .map(|ms| Value::from(iso_time(ms as i64)))
                .ok_or("a time is not a number")
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(object(vec![("texts", Value::Array(texts))]))
}

fn base64_decode(text: &str) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    let mut held: u32 = 0;
    let mut bits = 0;
    for char in text.bytes().filter(|char| *char != b'=') {
        let six = match char {
            b'A'..=b'Z' => char - b'A',
            b'a'..=b'z' => char - b'a' + 26,
            b'0'..=b'9' => char - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            _ => return Err("the source is not base64".to_owned()),
        };
        held = (held << 6) | u32::from(six);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            bytes.push((held >> bits) as u8);
            held &= (1 << bits) - 1;
        }
    }
    Ok(bytes)
}

fn rows_of(input: &Value) -> Result<Vec<String>, String> {
    let source = input.get("source").ok_or("input.source is absent")?;
    let bytes = match source.get("base64").and_then(Value::as_str) {
        Some(encoded) => base64_decode(encoded)?,
        None => {
            let mut text = texts_at(source, "lines")?.join("\n");
            text.push('\n');
            text.into_bytes()
        }
    };
    Ok(lines_of(&bytes))
}

fn claude_version_in(input: &Value) -> Outcome {
    let found = texts_at(input, "texts")?
        .iter()
        .map(|text| Value::from(version_in(text)))
        .collect();
    Ok(object(vec![("found", Value::Array(found))]))
}

#[cfg(unix)]
fn paths_is_inside(input: &Value) -> Outcome {
    let made = std::env::temp_dir().join(format!("nyaaai-paths-{}", random_uuid()));
    fs::create_dir(&made).map_err(failed)?;
    let started = std::env::current_dir().map_err(failed)?;
    let relative = input.get("relative").and_then(Value::as_bool) == Some(true);
    let judged = (|| -> Outcome {
        let root = fs::canonicalize(&made).map_err(failed)?;
        let root = root.to_string_lossy();
        for entry in list_at(input, "tree")? {
            let at = format!("{root}/{}", text_at(entry, "path")?);
            let link = entry.get("link").and_then(Value::as_str);
            match link {
                Some(target) => std::os::unix::fs::symlink(target.replace("$ROOT", &root), &at),
                None if entry.get("file").and_then(Value::as_bool) == Some(true) => {
                    fs::write(&at, "")
                }
                None => fs::create_dir_all(&at),
            }
            .map_err(failed)?;
        }
        if relative {
            std::env::set_current_dir(&*root).map_err(failed)?;
        }
        let place = |text: &str| match text.strip_prefix('/') {
            None if !relative => PathBuf::from(format!("{root}/{text}")),
            _ => PathBuf::from(text),
        };
        let mut results = Vec::new();
        for check in list_at(input, "checks")? {
            let child = place(text_at(check, "child")?);
            let parent = place(text_at(check, "parent")?);
            results.push(match is_inside(&child, &parent) {
                Ok(inside) => Value::Bool(inside),
                Err(_) => Value::from("error"),
            });
        }
        Ok(object(vec![("results", Value::Array(results))]))
    })();
    let _ = std::env::set_current_dir(started);
    let _ = fs::remove_dir_all(&made);
    judged
}

#[cfg(not(unix))]
fn paths_is_inside(_input: &Value) -> Outcome {
    Err("links need a Unix host".to_owned())
}

fn file_lines_of(input: &Value) -> Outcome {
    let bytes = base64_decode(text_at(input, "base64")?)?;
    Ok(object(vec![("rows", Value::from(lines_of(&bytes)))]))
}

fn stream_of(input: &Value) -> Result<Lines<'static>, String> {
    let file = input
        .get("source")
        .and_then(|source| source.get("file"))
        .and_then(Value::as_str);
    Ok(match file {
        Some(file) => Box::new(session_lines(Path::new(file)).map_err(failed)?),
        None => Box::new(rows_of(input)?.into_iter().map(Ok)),
    })
}

fn each(rows: &[String]) -> impl Iterator<Item = io::Result<String>> + '_ {
    rows.iter().cloned().map(Ok)
}

fn failed(error: io::Error) -> String {
    error.to_string()
}

fn row_json(row: &RowMeta) -> Value {
    let mut held = Object::new();
    held.insert("line", Value::from(row.line));
    held.insert("bytes", Value::from(row.bytes));
    held.insert("type", Value::from(row.kind.as_str()));
    held.insert_some("uuid", row.uuid.as_deref().map(Value::from));
    held.insert_some("at", row.at.as_deref().map(Value::from));
    held.insert("openTools", Value::from(row.open_tools));
    held.insert("delivered", Value::Bool(row.delivered));
    held.insert_some("text", row.text.as_deref().map(Value::from));
    held.insert("points", Value::from(row.points.clone()));
    Value::Object(held)
}

fn boundary_json(boundary: &BoundaryMeta) -> Value {
    object(vec![
        ("line", Value::from(boundary.line)),
        ("uuid", Value::from(boundary.uuid.as_str())),
        (
            "preservedLines",
            Value::from(boundary.preserved_lines.clone()),
        ),
        ("preservedSound", Value::Bool(boundary.preserved_sound)),
        ("preserved", Value::from(boundary.preserved.clone())),
        ("summaryLine", Value::from(boundary.summary_line)),
        (
            "cumulativeDroppedTokens",
            Value::Number(boundary.cumulative_dropped_tokens),
        ),
        ("postTokens", Value::Number(boundary.post_tokens)),
        ("known", Value::Bool(boundary.known)),
    ])
}

fn index_json(index: &SessionIndex) -> Value {
    let header = index
        .header
        .iter()
        .map(|(kind, line)| Value::Array(vec![Value::from(kind.as_str()), Value::from(*line)]))
        .collect();
    object(vec![
        ("lines", Value::from(index.lines)),
        ("unparsed", Value::from(index.unparsed)),
        ("unreadable", Value::from(index.unreadable.clone())),
        ("version", Value::from(index.version.clone())),
        ("header", Value::Array(header)),
        (
            "boundary",
            index.boundary.as_ref().map_or(Value::Null, boundary_json),
        ),
        ("boundaries", Value::from(index.boundaries)),
        ("standing", Value::from(index.standing.clone())),
        (
            "tail",
            Value::Array(index.tail.iter().map(row_json).collect()),
        ),
    ])
}

fn request_of(request: &Value) -> Result<CutRequest, String> {
    let mode = request
        .get("mode")
        .and_then(Value::as_str)
        .and_then(CutMode::parse)
        .ok_or("the request names no mode")?;
    Ok(CutRequest {
        mode,
        target_uuid: optional_text(request, "targetUuid"),
        through_uuid: optional_text(request, "throughUuid"),
        end_before: optional_text(request, "endBefore"),
        summary_text: optional_text(request, "summaryText"),
        live_tail: request.get("liveTail").and_then(Value::as_bool) == Some(true),
    })
}

fn cut_rehomed(input: &Value) -> Outcome {
    let session_id = text_at(input, "sessionId")?;
    let rows = texts_at(input, "rows")?
        .iter()
        .map(|row| Value::from(rehomed(row, session_id)))
        .collect();
    Ok(object(vec![("rows", Value::Array(rows))]))
}

fn cut_index(input: &Value) -> Outcome {
    let index = index_session(stream_of(input)?).map_err(failed)?;
    Ok(object(vec![("index", index_json(&index))]))
}

fn cut_plan(input: &Value) -> Outcome {
    let index = index_session(stream_of(input)?).map_err(failed)?;
    let cli_version = text_at(input, "cliVersion")?;
    let mut plans = Vec::new();
    for request in list_at(input, "requests")? {
        plans.push(match plan_cut(&index, &request_of(request)?, cli_version) {
            Ok(plan) => plan_json(&plan),
            Err(refused) => refused_json(&refused),
        });
    }
    Ok(object(vec![("plans", Value::Array(plans))]))
}

struct MemoryPorts {
    rows: Vec<String>,
    source_path: String,
    id_prefix: String,
    now_ms: i64,
    cli_version: String,
    summarize: Option<Box<Summarizer<'static>>>,
    built: RefCell<HashMap<String, Vec<String>>>,
    minted: Cell<usize>,
}

impl CutPorts for MemoryPorts {
    fn read(&self) -> io::Result<Lines<'_>> {
        Ok(Box::new(each(&self.rows)))
    }

    fn source_path(&self) -> String {
        self.source_path.clone()
    }

    fn create(&self, session_id: &str, lines: &[String]) -> io::Result<()> {
        self.built
            .borrow_mut()
            .insert(session_id.to_owned(), lines.to_vec());
        Ok(())
    }

    fn read_built(&self, session_id: &str) -> io::Result<Lines<'_>> {
        let rows = self
            .built
            .borrow()
            .get(session_id)
            .cloned()
            .unwrap_or_default();
        Ok(Box::new(rows.into_iter().map(Ok)))
    }

    fn discard(&self, session_id: &str) -> io::Result<()> {
        self.built.borrow_mut().remove(session_id);
        Ok(())
    }

    fn new_id(&self) -> String {
        self.minted.set(self.minted.get() + 1);
        format!("{}{}", self.id_prefix, self.minted.get())
    }

    fn now_ms(&self) -> i64 {
        self.now_ms
    }

    fn cli_version(&self) -> io::Result<String> {
        Ok(self.cli_version.clone())
    }

    fn summarizer(&self) -> Option<&Summarizer<'_>> {
        self.summarize.as_deref()
    }
}

fn cut_cut(input: &Value) -> Outcome {
    let summarized: Rc<RefCell<Option<Value>>> = Rc::default();
    let summarize = optional_text(input, "summary").map(|summary| {
        let seen = Rc::clone(&summarized);
        let answer = move |standing: Option<&str>, rows: &[String]| {
            *seen.borrow_mut() = Some(object(vec![
                ("standing", Value::from(standing)),
                ("rows", Value::from(rows.to_vec())),
            ]));
            Ok(summary.clone())
        };
        Box::new(answer) as Box<Summarizer<'static>>
    });
    let ports = MemoryPorts {
        rows: rows_of(input)?,
        source_path: text_at(input, "sourcePath")?.to_owned(),
        id_prefix: text_at(input, "idPrefix")?.to_owned(),
        now_ms: input
            .get("nowMs")
            .and_then(Value::as_f64)
            .ok_or("input.nowMs is not a number")? as i64,
        cli_version: text_at(input, "cliVersion")?.to_owned(),
        summarize,
        built: RefCell::default(),
        minted: Cell::new(0),
    };
    let request = request_of(input.get("request").ok_or("input.request is absent")?)?;
    let outcome = match cut_session(&ports, &request) {
        Ok(outcome) => outcome,
        Err(CutError::Io(error)) => return Err(failed(error)),
        Err(error) => return Ok(object(vec![("error", Value::from(error.to_string()))])),
    };
    let (result, built) = match &outcome {
        Ok(done) => (
            object(vec![
                ("ok", Value::Bool(true)),
                ("sessionId", Value::from(done.session_id.as_str())),
                ("plan", plan_json(&done.plan)),
            ]),
            Value::from(ports.built.borrow().get(&done.session_id).cloned()),
        ),
        Err(refused) => (refused_json(refused), Value::Null),
    };
    let summarized = summarized.borrow().clone().unwrap_or(Value::Null);
    Ok(object(vec![
        ("result", result),
        ("built", built),
        ("summarized", summarized),
    ]))
}

fn cut_verify(input: &Value) -> Outcome {
    let rows = rows_of(input)?;
    let paired = input
        .get("paired")
        .and_then(Value::as_bool)
        .ok_or("input.paired is not a flag")?;
    let problems = verify_cut(each(&rows), text_at(input, "sessionId")?, paired).map_err(failed)?;
    Ok(object(vec![("problems", Value::from(problems))]))
}

fn whole(text: &str) -> String {
    text.to_owned()
}

fn after_first_line(text: &str) -> String {
    text.split_once('\n')
        .map_or(text, |(_, rest)| rest)
        .to_owned()
}

fn cut_pick_turn(input: &Value) -> Outcome {
    let rows = rows_of(input)?;
    let index = index_session(each(&rows)).map_err(failed)?;
    let mut picked = Vec::new();
    for pick in list_at(input, "picks")? {
        let body_of: &dyn Fn(&str) -> String = match text_at(pick, "bodyOf")? {
            "whole" => &whole,
            "afterFirstLine" => &after_first_line,
            other => return Err(format!("unknown bodyOf {other}")),
        };
        let at = pick
            .get("at")
            .and_then(Value::as_f64)
            .ok_or("a pick holds no time")? as i64;
        let turn = pick_turn(
            &index,
            &TurnPick {
                at,
                text: optional_text(pick, "text"),
            },
            body_of,
        );
        picked.push(turn.map_or(Value::Null, |turn| {
            object(vec![
                ("uuid", Value::from(turn.uuid)),
                ("exact", Value::Bool(turn.exact)),
            ])
        }));
    }
    Ok(object(vec![("picked", Value::Array(picked))]))
}
