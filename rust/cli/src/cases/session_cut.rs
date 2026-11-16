//! Session cut cases.

use std::cell::RefCell;
use std::collections::HashMap;
use std::io;
use std::path::Path;
use std::rc::Rc;

use nyaaai_contract::id::{CountedIds, Ids};
use nyaaai_contract::time::{Clock, SteppedClock};
use nyaaai_operator::claude::session_cut::{
    self as operator, CutPorts, Lines, Summarizer, Summary, TurnPick, cut_session, index_session,
    plan_cut, verify_cut,
};
use nyaaai_operator::claude::session_file::{lines_of, session_lines};

use super::session_file::bytes_of;
use nyaaai_testkit::cases::{Outcome, block_on, failed};

use super::shapes::*;

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

pub fn rehomed(input: SessionCutRehomedInput) -> Outcome<SessionCutRehomedResult> {
    let rows = input
        .rows
        .iter()
        .map(|row| operator::rehomed(row, &input.session_id))
        .collect();
    Ok(SessionCutRehomedResult { rows })
}

pub fn index(input: SessionCutIndexInput) -> Outcome<SessionCutIndexResult> {
    let index = index_session(stream_of(&input.source)?).map_err(failed)?;
    Ok(SessionCutIndexResult { index })
}

pub fn plan(input: SessionCutPlanInput) -> Outcome<SessionCutPlanResult> {
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

/// Answers at once.
type Answer = dyn Fn(Option<&str>, &[String]) -> io::Result<String>;

struct MemoryPorts {
    rows: Vec<String>,
    source_path: String,
    ids: CountedIds,
    clock: SteppedClock,
    cli_version: String,
    summarize: Option<Immediate>,
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

    fn summarizer(&self) -> Option<&dyn Summarizer> {
        self.summarize.as_ref().map(|held| held as &dyn Summarizer)
    }
}

/// Answers at once.
struct Immediate(Box<Answer>);

impl Summarizer for Immediate {
    fn summarize<'a>(&'a self, standing: Option<&'a str>, rows: &'a [String]) -> Summary<'a> {
        Box::pin(std::future::ready((self.0)(standing, rows)))
    }
}

pub fn cut(input: SessionCutCutInput) -> Outcome<SessionCutCutResult> {
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
        Immediate(Box::new(answer))
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
    let outcome = match block_on(cut_session(&ports, &input.request))? {
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

pub fn verify(input: SessionCutVerifyInput) -> Outcome<SessionCutVerifyResult> {
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

pub fn pick_turn(input: SessionCutPickTurnInput) -> Outcome<SessionCutPickTurnResult> {
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
            operator::pick_turn(&index, &wanted, body_of)
        })
        .collect();
    Ok(SessionCutPickTurnResult { picked })
}
