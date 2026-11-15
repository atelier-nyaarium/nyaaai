//! Gate-only case handlers.

use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use nyaaai_contract::base64;
use nyaaai_contract::id::{CountedIds, Ids, random_uuid};
use nyaaai_contract::js::to_js_string;
use nyaaai_contract::json::{Object, Record, Value, canonical_json, parse_json, stringify_json};
use nyaaai_contract::number;
use nyaaai_contract::order::{compare_text, sorted_texts};
use nyaaai_contract::pattern::{Pattern, PatternMatch, pattern_problem};
use nyaaai_contract::random::{Random, SequenceRandom};
use nyaaai_contract::shape::{PathPart, compatible_versions};
use nyaaai_contract::shapes::{frames, session_cut as cut_shapes};
use nyaaai_contract::strict_schema::{self, Issue};
use nyaaai_contract::text;
use nyaaai_contract::time::{Clock, SteppedClock};
use nyaaai_operator::claude::cli_version::version_in;
use nyaaai_operator::claude::session_cut::{
    CutPorts, Lines, Summarizer, TurnPick, cut_session, index_session, pick_turn, plan_cut,
    rehomed, verify_cut,
};
use nyaaai_operator::claude::session_file::{lines_of, session_lines};
use nyaaai_process::paths::is_inside;
use nyaaai_testkit::claude_session::compacted_session;
use nyaaai_testkit::scheduler::TimerWheel;
use nyaaai_testkit::torture_shapes;
use nyaaai_testkit::trace::{ReplayTransport, read_trace};
use nyaaai_testkit::trace_shapes;

use super::adapter::{Inbound, RunContext, RunError};
use super::case_shapes::*;
use super::commands::typed;

type Outcome<T> = Result<T, String>;

/// Gate case dispatcher.
pub fn run(name: &str, input: &Value, context: &RunContext) -> Option<Result<Value, RunError>> {
    Some(match name {
        "testkit.converse" => typed(input, |input| testkit_converse(input, context)),
        _ => return None,
    })
}

fn testkit_converse(
    input: TestkitConverseInput,
    context: &RunContext,
) -> Result<TestkitConverseResult, RunError> {
    let said = |key: &str, value: Value| {
        let mut event = Object::new();
        event.insert(key, value);
        context.emit(Value::Object(event));
    };
    let echo = |count: u64| {
        let mut asked = Object::new();
        asked.insert("count", Value::Number(count as f64));
        context.ask("echo", Value::Object(asked))
    };
    let together: Vec<String> = if input.together {
        (1..=input.requests).map(echo).collect()
    } else {
        Vec::new()
    };
    for id in &together {
        match context.reply(id) {
            Ok(value) => said("reply", value),
            Err(RunError::HostRequest(message)) => said("failed", Value::from(message)),
            Err(error) => return Err(error),
        }
    }
    for count in (1..=input.requests).filter(|_| !input.together) {
        let id = echo(count);
        loop {
            match context.receive() {
                Inbound::Interrupt => return Err(RunError::Interrupted),
                Inbound::Steer(text) => said("steer", Value::from(text)),
                Inbound::Reply { id: held, value } if held == id => {
                    said("reply", value);
                    break;
                }
                Inbound::ReplyFailed { id: held, message } if held == id => {
                    said("failed", Value::from(message));
                    break;
                }
                Inbound::Reply { .. } | Inbound::ReplyFailed { .. } => {}
            }
        }
    }
    Ok(TestkitConverseResult {
        ids: vec![context.ids.new_id(), context.ids.new_id()],
        now_ms: context.clock.now_ms(),
        steers: context.steers(),
    })
}

fn failed(error: io::Error) -> String {
    error.to_string()
}

pub fn json_round_trip(input: JsonRoundTripInput) -> Outcome<JsonRoundTripResult> {
    let results = input
        .texts
        .iter()
        .map(|text| {
            let text = parse_json(text).map(|value| stringify_json(&value));
            JsonRoundTripResultResultsItem {
                ok: text.is_some(),
                text,
            }
        })
        .collect();
    Ok(JsonRoundTripResult { results })
}

pub fn json_canonical(input: JsonCanonicalInput) -> Outcome<JsonCanonicalResult> {
    let texts = input
        .texts
        .iter()
        .map(|text| parse_json(text).map(|value| canonical_json(&value)))
        .collect();
    Ok(JsonCanonicalResult { texts })
}

pub fn js_string(input: JsStringInput) -> Outcome<JsStringResult> {
    let strings = input
        .texts
        .iter()
        .map(|text| {
            let value = parse_json(text).ok_or("a value text is not JSON")?;
            Ok(to_js_string(Some(&value)))
        })
        .collect::<Outcome<_>>()?;
    Ok(JsStringResult { strings })
}

pub fn order_sorted_texts(input: OrderSortedTextsInput) -> Outcome<OrderSortedTextsResult> {
    Ok(OrderSortedTextsResult {
        texts: sorted_texts(&input.texts),
    })
}

pub fn contract_round_trip(input: ContractRoundTripInput) -> Outcome<ContractRoundTripResult> {
    let normalize = match input.set.as_str() {
        "frames" => frames::normalize,
        "sessionCut" => cut_shapes::normalize,
        "torture" => torture_shapes::normalize,
        "trace" => trace_shapes::normalize,
        _ => return Err(format!("no shape {}.{}", input.set, input.shape)),
    };
    let unknown = || format!("no shape {}.{}", input.set, input.shape);
    if normalize(&input.shape, &Value::Null).is_none() {
        return Err(unknown());
    }
    let refusal = |path: Vec<frames::PathPart>| ContractRoundTripResultResultsItem {
        ok: false,
        text: None,
        path: Some(path),
    };
    let mut results = Vec::new();
    for text in &input.texts {
        let Some(value) = parse_json(text) else {
            results.push(refusal(Vec::new()));
            continue;
        };
        results.push(match normalize(&input.shape, &value) {
            None => return Err(unknown()),
            Some(Ok(held)) => ContractRoundTripResultResultsItem {
                ok: true,
                text: Some(stringify_json(&held)),
                path: None,
            },
            Some(Err(refused)) => refusal(
                refused
                    .path
                    .into_iter()
                    .map(|part| match part {
                        PathPart::Key(key) => frames::PathPart::Text(key),
                        PathPart::Index(index) => frames::PathPart::Integer(index as u64),
                    })
                    .collect(),
            ),
        });
    }
    Ok(ContractRoundTripResult { results })
}

pub fn time_parse_iso(input: TimeParseIsoInput) -> Outcome<TimeParseIsoResult> {
    let results = input
        .texts
        .iter()
        .map(|text| nyaaai_contract::time::parse_iso_time(text))
        .collect();
    Ok(TimeParseIsoResult { results })
}

pub fn time_http_date(input: TimeHttpDateInput) -> Outcome<TimeHttpDateResult> {
    let results = input
        .texts
        .iter()
        .map(|text| nyaaai_contract::time::parse_http_date(text, input.now_ms))
        .collect();
    Ok(TimeHttpDateResult { results })
}

pub fn time_retry_after(input: TimeRetryAfterInput) -> Outcome<TimeRetryAfterResult> {
    let results = input
        .headers
        .iter()
        .map(|header| nyaaai_contract::time::parse_retry_after(header.as_deref(), input.now_ms))
        .collect();
    Ok(TimeRetryAfterResult { results })
}

pub fn time_reset_stamp(input: TimeResetStampInput) -> Outcome<TimeResetStampResult> {
    let results = input
        .values
        .iter()
        .map(|value| nyaaai_contract::time::reset_stamp_ms(value, input.now_ms))
        .collect();
    Ok(TimeResetStampResult { results })
}

pub fn time_day_key(input: TimeDayKeyInput) -> Outcome<TimeDayKeyResult> {
    let rows = input
        .times
        .iter()
        .map(|ms| {
            input
                .offsets
                .iter()
                .map(|offset| nyaaai_contract::time::day_key(*ms, *offset))
                .collect()
        })
        .collect();
    Ok(TimeDayKeyResult { rows })
}

pub fn text_cuts(input: TextCutsInput) -> Outcome<TextCutsResult> {
    let points = &input.points;
    let each = |held: &dyn Fn(usize) -> String| -> Vec<String> {
        points.iter().map(|at| held(*at as usize)).collect()
    };
    let offsets = |held: &dyn Fn(usize) -> usize| -> Vec<i64> {
        points.iter().map(|at| held(*at as usize) as i64).collect()
    };
    let rows = input
        .texts
        .iter()
        .map(|held| TextCutsResultRowsItem {
            length: text::text_length(held) as i64,
            utf8: text::utf8_length(held) as i64,
            before: offsets(&|at| text::cut_before(held, at)),
            after: offsets(&|at| text::cut_after(held, at)),
            heads: each(&|max| text::head_of(held, max).to_owned()),
            tails: each(&|max| text::tail_of(held, max).to_owned()),
            bytes: each(&|max| text::cut_utf8(held, max).to_owned()),
            lower: text::ascii_lower(held),
            upper: text::ascii_upper(held),
        })
        .collect();
    Ok(TextCutsResult { rows })
}

pub fn text_slices(input: TextSlicesInput) -> Outcome<TextSlicesResult> {
    let slices = input
        .spans
        .iter()
        .map(|span| {
            text::slice_text(&input.text, span.start as usize, span.end as usize).to_owned()
        })
        .collect();
    Ok(TextSlicesResult { slices })
}

pub fn text_find(input: TextFindInput) -> Outcome<TextFindResult> {
    let table = |find: &dyn Fn(&str, usize) -> Option<usize>| -> Vec<Vec<i64>> {
        input
            .needles
            .iter()
            .map(|needle| {
                input
                    .froms
                    .iter()
                    .map(|from| find(needle, *from as usize).map_or(-1, |at| at as i64))
                    .collect()
            })
            .collect()
    };
    Ok(TextFindResult {
        first: table(&|needle, from| text::index_of_text(&input.text, needle, from)),
        last: table(&|needle, from| text::last_index_of_text(&input.text, needle, from)),
    })
}

pub fn number_text(input: NumberTextInput) -> Outcome<NumberTextResult> {
    let texts = input
        .values
        .iter()
        .map(|value| number::number_text(*value))
        .collect();
    Ok(NumberTextResult { texts })
}

/// Preserves negative zero.
pub fn number_round(input: NumberRoundInput) -> Outcome<NumberRoundResult> {
    let texts = input
        .values
        .iter()
        .map(|value| {
            let rounded = number::round_half_up(*value);
            if rounded == 0.0 && rounded.is_sign_negative() {
                "-0".to_owned()
            } else {
                number::number_text(rounded)
            }
        })
        .collect();
    Ok(NumberRoundResult { texts })
}

pub fn number_fixed(input: NumberFixedInput) -> Outcome<NumberFixedResult> {
    let rows = input
        .values
        .iter()
        .map(|value| {
            input
                .digits
                .iter()
                .map(|digits| number::fixed_text(*value, *digits as usize))
                .collect()
        })
        .collect();
    Ok(NumberFixedResult { rows })
}

pub fn number_of_text(input: NumberOfTextInput) -> Outcome<NumberOfTextResult> {
    let values = input
        .texts
        .iter()
        .map(|text| number::number_of_text(text))
        .collect();
    Ok(NumberOfTextResult { values })
}

pub fn number_integer_of_text(
    input: NumberIntegerOfTextInput,
) -> Outcome<NumberIntegerOfTextResult> {
    let values = input
        .texts
        .iter()
        .map(|text| number::integer_of_text(text))
        .collect();
    Ok(NumberIntegerOfTextResult { values })
}

pub fn time_iso(input: TimeIsoInput) -> Outcome<TimeIsoResult> {
    let texts = input
        .times
        .iter()
        .map(|ms| nyaaai_contract::time::iso_time(*ms))
        .collect();
    Ok(TimeIsoResult { texts })
}

fn bytes_of(text: &str) -> Outcome<Vec<u8>> {
    base64::base64_decode(text).ok_or_else(|| "the source is not base64".to_owned())
}

pub fn shape_compatible_versions(
    input: ShapeCompatibleVersionsInput,
) -> Outcome<ShapeCompatibleVersionsResult> {
    let compatible = input
        .pairs
        .iter()
        .map(|pair| compatible_versions(&pair.left, &pair.right))
        .collect();
    Ok(ShapeCompatibleVersionsResult { compatible })
}

pub fn base64_decode(input: Base64DecodeInput) -> Outcome<Base64DecodeResult> {
    let texts = input
        .texts
        .iter()
        .map(|text| base64::base64_decode(text).map(|bytes| base64::base64_encode(&bytes)))
        .collect();
    Ok(Base64DecodeResult { texts })
}

fn rows_of(source: &SessionSource) -> Outcome<Vec<String>> {
    let bytes = match (&source.base64, &source.lines) {
        (Some(encoded), _) => bytes_of(encoded)?,
        (None, Some(lines)) => {
            let mut text = lines.join("\n");
            text.push('\n');
            text.into_bytes()
        }
        (None, None) => return Err("a file source is read as a stream only".to_owned()),
    };
    Ok(lines_of(&bytes))
}

/// Files stream. Rows preload.
fn stream_of(source: &SessionSource) -> Outcome<Lines<'static>> {
    Ok(match &source.file {
        Some(file) => Box::new(session_lines(Path::new(file)).map_err(failed)?),
        None => Box::new(rows_of(source)?.into_iter().map(Ok)),
    })
}

fn each(rows: &[String]) -> impl Iterator<Item = io::Result<String>> + '_ {
    rows.iter().cloned().map(Ok)
}

pub fn session_cut_rehomed(input: SessionCutRehomedInput) -> Outcome<SessionCutRehomedResult> {
    let rows = input
        .rows
        .iter()
        .map(|row| rehomed(row, &input.session_id))
        .collect();
    Ok(SessionCutRehomedResult { rows })
}

pub fn session_cut_index(input: SessionCutIndexInput) -> Outcome<SessionCutIndexResult> {
    let index = index_session(stream_of(&input.source)?).map_err(failed)?;
    Ok(SessionCutIndexResult { index })
}

pub fn session_cut_plan(input: SessionCutPlanInput) -> Outcome<SessionCutPlanResult> {
    let index = index_session(stream_of(&input.source)?).map_err(failed)?;
    let plans = input
        .requests
        .iter()
        .map(
            |request| match plan_cut(&index, request, &input.cli_version) {
                Ok(plan) => SessionCutPlanResultPlansItem::CutPlan(plan),
                Err(refused) => SessionCutPlanResultPlansItem::CutRefused(refused),
            },
        )
        .collect();
    Ok(SessionCutPlanResult { plans })
}

struct MemoryPorts {
    rows: Vec<String>,
    source_path: String,
    ids: CountedIds,
    clock: SteppedClock,
    cli_version: String,
    summarize: Option<Box<Summarizer<'static>>>,
    built: RefCell<HashMap<String, Vec<String>>>,
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

    fn ids(&self) -> &dyn Ids {
        &self.ids
    }

    fn clock(&self) -> &dyn Clock {
        &self.clock
    }

    fn cli_version(&self) -> io::Result<String> {
        Ok(self.cli_version.clone())
    }

    fn summarizer(&self) -> Option<&Summarizer<'_>> {
        self.summarize.as_deref()
    }
}

pub fn session_cut_cut(input: SessionCutCutInput) -> Outcome<SessionCutCutResult> {
    let summarized: Rc<RefCell<Option<SessionCutCutResultSummarized>>> = Rc::default();
    let summarize = input.summary.clone().map(|summary| {
        let seen = Rc::clone(&summarized);
        let answer = move |standing: Option<&str>, rows: &[String]| {
            *seen.borrow_mut() = Some(SessionCutCutResultSummarized {
                standing: standing.map(str::to_owned),
                rows: rows.to_vec(),
            });
            Ok(summary.clone())
        };
        Box::new(answer) as Box<Summarizer<'static>>
    });
    let ports = MemoryPorts {
        rows: rows_of(&input.source)?,
        source_path: input.source_path,
        ids: CountedIds::new(input.id_prefix),
        clock: SteppedClock::new(input.now_ms, 0),
        cli_version: input.cli_version,
        summarize,
        built: RefCell::default(),
    };
    let outcome = match cut_session(&ports, &input.request) {
        Ok(outcome) => outcome,
        Err(error) => {
            return Ok(SessionCutCutResult {
                error: Some(error.to_string()),
                result: None,
                built: None,
                summarized: None,
            });
        }
    };
    let (result, built) = match outcome {
        Ok(done) => {
            let built = ports.built.borrow().get(&done.session_id).cloned();
            (SessionCutCutResultResult::CutDone(done), built)
        }
        Err(refused) => (SessionCutCutResultResult::CutRefused(refused), None),
    };
    Ok(SessionCutCutResult {
        error: None,
        result: Some(result),
        built: Some(built),
        summarized: Some(summarized.borrow().clone()),
    })
}

pub fn session_cut_verify(input: SessionCutVerifyInput) -> Outcome<SessionCutVerifyResult> {
    let rows = rows_of(&input.source)?;
    let problems = verify_cut(each(&rows), &input.session_id, input.paired).map_err(failed)?;
    Ok(SessionCutVerifyResult { problems })
}

fn whole(text: &str) -> String {
    text.to_owned()
}

fn after_first_line(text: &str) -> String {
    text.split_once('\n')
        .map_or(text, |(_, rest)| rest)
        .to_owned()
}

pub fn session_cut_pick_turn(input: SessionCutPickTurnInput) -> Outcome<SessionCutPickTurnResult> {
    let rows = rows_of(&input.source)?;
    let index = index_session(each(&rows)).map_err(failed)?;
    let picked = input
        .picks
        .into_iter()
        .map(|pick| {
            let body_of: &dyn Fn(&str) -> String = match pick.body_of {
                BodyOf::Whole => &whole,
                BodyOf::AfterFirstLine => &after_first_line,
            };
            let wanted = TurnPick {
                at: pick.at,
                text: pick.text,
            };
            pick_turn(&index, &wanted, body_of)
        })
        .collect();
    Ok(SessionCutPickTurnResult { picked })
}

pub fn session_file_lines_of(input: SessionFileLinesOfInput) -> Outcome<SessionFileLinesOfResult> {
    let rows = lines_of(&bytes_of(&input.base64)?);
    Ok(SessionFileLinesOfResult { rows })
}

#[cfg(unix)]
pub fn paths_is_inside(input: PathsIsInsideInput) -> Outcome<PathsIsInsideResult> {
    let made = std::env::temp_dir().join(format!("nyaaai-paths-{}", random_uuid()));
    fs::create_dir(&made).map_err(failed)?;
    let started = std::env::current_dir().map_err(failed)?;
    let relative = input.relative.is_some();
    let judged = (|| -> Outcome<PathsIsInsideResult> {
        let root = fs::canonicalize(&made).map_err(failed)?;
        let root = root.to_string_lossy();
        for entry in &input.tree {
            let at = format!("{root}/{}", entry.path);
            match &entry.link {
                Some(target) => std::os::unix::fs::symlink(target.replace("$ROOT", &root), &at),
                None if entry.file.is_some() => fs::write(&at, ""),
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
        let results = input
            .checks
            .iter()
            .map(
                |check| match is_inside(&place(&check.child), &place(&check.parent)) {
                    Ok(inside) => PathsIsInsideResultResultsItem::Flag(inside),
                    Err(_) => PathsIsInsideResultResultsItem::Error,
                },
            )
            .collect();
        Ok(PathsIsInsideResult { results })
    })();
    let _ = std::env::set_current_dir(started);
    let _ = fs::remove_dir_all(&made);
    judged
}

#[cfg(not(unix))]
pub fn paths_is_inside(_input: PathsIsInsideInput) -> Outcome<PathsIsInsideResult> {
    Err("links need a Unix host".to_owned())
}

pub fn claude_version_in(input: ClaudeVersionInInput) -> Outcome<ClaudeVersionInResult> {
    let found = input
        .texts
        .iter()
        .map(|text| version_in(text).map(str::to_owned))
        .collect();
    Ok(ClaudeVersionInResult { found })
}

pub fn testkit_wheel(input: TestkitWheelInput) -> Outcome<TestkitWheelResult> {
    let wheel = TimerWheel::new(input.start_ms);
    let steps = input
        .steps
        .iter()
        .map(|step| {
            let ms = step.ms.unwrap_or(0);
            let tag = step.tag.as_deref().unwrap_or("");
            let mut fired = Vec::new();
            let mut cancelled = false;
            match step.op {
                TestkitWheelInputStepsItemOp::Advance => fired = wheel.advance(ms),
                TestkitWheelInputStepsItemOp::Cancel => cancelled = wheel.cancel(tag),
                TestkitWheelInputStepsItemOp::After => wheel.after(ms, tag),
            }
            TestkitWheelResultStepsItem {
                now_ms: wheel.now_ms(),
                waiting: wheel.waiting(),
                fired: fired
                    .into_iter()
                    .map(|held| TestkitWheelResultStepsItemFiredItem {
                        tag: held.tag,
                        at_ms: held.at_ms,
                    })
                    .collect(),
                cancelled,
            }
        })
        .collect();
    Ok(TestkitWheelResult { steps })
}

pub fn testkit_random(input: TestkitRandomInput) -> Outcome<TestkitRandomResult> {
    let random = SequenceRandom::new(input.values).ok_or("a random sequence holds a value")?;
    let drawn = (0..input.draws).map(|_| random.next()).collect();
    Ok(TestkitRandomResult { drawn })
}

pub fn testkit_replay(input: TestkitReplayInput) -> Outcome<TestkitReplayResult> {
    let trace = read_trace(Path::new(&input.file)).map_err(failed)?;
    let asked: Vec<Value> = trace
        .exchanges
        .iter()
        .map(|exchange| exchange.request.clone())
        .collect();
    let mut transport = ReplayTransport::new(trace);
    let mut exchanges = Vec::new();
    for request in asked {
        let replayed = transport.fetch(request).map_err(failed)?;
        let mut named = replayed.headers;
        named.sort_by(|(left, _), (right, _)| compare_text(left, right));
        let mut headers = Record::new();
        for (name, value) in named {
            headers.insert(name, value);
        }
        exchanges.push(TestkitReplayResultExchangesItem {
            status: i64::from(replayed.status),
            headers,
            chunks: replayed
                .chunks
                .iter()
                .map(|chunk| chunk.len() as i64)
                .collect(),
            body: String::from_utf8_lossy(&replayed.chunks.concat()).into_owned(),
        });
    }
    Ok(TestkitReplayResult {
        requests: transport.requests,
        exchanges,
    })
}

fn pattern_match_shape(matched: PatternMatch) -> PatternCaseMatch {
    PatternCaseMatch {
        start: matched.start as u64,
        end: matched.end as u64,
        text: matched.text,
        captures: matched.captures,
    }
}

pub fn pattern_problems(input: PatternProblemsInput) -> Outcome<PatternProblemsResult> {
    Ok(PatternProblemsResult {
        refused: input
            .patterns
            .iter()
            .map(|held| pattern_problem(&held.source, &held.flags).is_some())
            .collect(),
    })
}

pub fn pattern_matches(input: PatternMatchesInput) -> Outcome<PatternMatchesResult> {
    let pattern =
        Pattern::compile(&input.source, &input.flags).map_err(|error| error.to_string())?;
    let results = input
        .texts
        .iter()
        .map(|text| PatternMatchesResultResultsItem {
            test: pattern.test(text),
            all: pattern
                .find_all(text)
                .into_iter()
                .map(pattern_match_shape)
                .collect(),
        })
        .collect();
    Ok(PatternMatchesResult { results })
}

pub fn pattern_find(input: PatternFindInput) -> Outcome<PatternFindResult> {
    let pattern =
        Pattern::compile(&input.source, &input.flags).map_err(|error| error.to_string())?;
    let found = input
        .froms
        .iter()
        .map(|from| {
            pattern
                .find(&input.text, *from as usize)
                .map(pattern_match_shape)
        })
        .collect();
    Ok(PatternFindResult { found })
}

pub fn pattern_edits(input: PatternEditsInput) -> Outcome<PatternEditsResult> {
    let pattern =
        Pattern::compile(&input.source, &input.flags).map_err(|error| error.to_string())?;
    let results = input
        .texts
        .iter()
        .map(|text| PatternEditsResultResultsItem {
            replaced: pattern.replace_all(text, &input.template),
            pieces: pattern.split(text),
        })
        .collect();
    Ok(PatternEditsResult { results })
}

fn strict_schema_issue(issue: Issue) -> StrictSchemaIssue {
    StrictSchemaIssue {
        path: issue
            .path
            .into_iter()
            .map(|part| match part {
                PathPart::Key(key) => frames::PathPart::Text(key),
                PathPart::Index(index) => frames::PathPart::Integer(index as u64),
            })
            .collect(),
        code: StrictSchemaIssueCode::parse(issue.code.as_str()).expect("shared issue code"),
        message: issue.message,
    }
}

fn issues_of(schema: &Value, text: &str) -> Outcome<Vec<StrictSchemaIssue>> {
    let value = parse_json(text).ok_or("a value text is not JSON")?;
    Ok(strict_schema::validate(schema, &value)
        .into_iter()
        .map(strict_schema_issue)
        .collect())
}

pub fn strict_schema_normalize(
    input: StrictSchemaNormalizeInput,
) -> Outcome<StrictSchemaNormalizeResult> {
    Ok(StrictSchemaNormalizeResult {
        schemas: input
            .schemas
            .iter()
            .map(strict_schema::normalize_schema)
            .collect(),
    })
}

pub fn strict_schema_problems(
    input: StrictSchemaProblemsInput,
) -> Outcome<StrictSchemaProblemsResult> {
    Ok(StrictSchemaProblemsResult {
        problems: input
            .schemas
            .iter()
            .map(|schema| {
                strict_schema::schema_problems(&strict_schema::normalize_schema(schema))
                    .into_iter()
                    .map(strict_schema_issue)
                    .collect()
            })
            .collect(),
    })
}

pub fn strict_schema_validate(
    input: StrictSchemaValidateInput,
) -> Outcome<StrictSchemaValidateResult> {
    let issues = input
        .texts
        .iter()
        .map(|text| issues_of(&input.schema, text))
        .collect::<Outcome<_>>()?;
    Ok(StrictSchemaValidateResult { issues })
}

pub fn strict_schema_validate_list(
    input: StrictSchemaValidateListInput,
) -> Outcome<StrictSchemaValidateListResult> {
    let issues = input
        .counts
        .iter()
        .map(|count| {
            let items = vec![input.item.as_str(); *count as usize].join(",");
            issues_of(&input.schema, &format!("[{items}]"))
        })
        .collect::<Outcome<_>>()?;
    Ok(StrictSchemaValidateListResult { issues })
}

pub fn testkit_compacted_session(
    _input: TestkitCompactedSessionInput,
) -> Outcome<TestkitCompactedSessionResult> {
    Ok(TestkitCompactedSessionResult {
        lines: compacted_session().session.lines,
    })
}
