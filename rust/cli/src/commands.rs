//! Adapter commands.

use std::cell::RefCell;
use std::io;
use std::path::{Path, PathBuf};

use nyaaai_contract::id::Ids;
use nyaaai_contract::json::Value;
use nyaaai_contract::shape::Shape;
use nyaaai_contract::shapes::session_cut::{
    ApplyCommand, CutOutcome, PlanCommand, PlanOutcome, SummarizeRequest,
};
use nyaaai_contract::time::Clock;
use nyaaai_operator::claude::session_cut::{
    CutError, CutPorts, Lines, Summarizer, cut_session, index_session, plan_cut,
};
use nyaaai_operator::claude::session_file::{FileCutPorts, session_lines};

use crate::adapter::{RunContext, RunError};

/// Host summary request.
pub const SUMMARIZE: &str = "sessionCut.summarize";

pub fn run(name: &str, input: &Value, context: &RunContext) -> Option<Result<Value, RunError>> {
    Some(match name {
        "sessionCut.plan" => plan(input),
        "sessionCut.apply" => apply(input, context),
        _ => return None,
    })
}

/// Decodes input; encodes result.
pub fn typed<Input: Shape, Output: Shape>(
    input: &Value,
    run: impl FnOnce(Input) -> Result<Output, RunError>,
) -> Result<Value, RunError> {
    let input = Input::decode(input).map_err(|refused| RunError::BadInput(refused.path))?;
    run(input).map(|result| result.encode())
}

fn plan(input: &Value) -> Result<Value, RunError> {
    typed(input, |input: PlanCommand| {
        let index = index_session(session_lines(Path::new(&input.source))?)?;
        Ok(match plan_cut(&index, &input.request, &input.cli_version) {
            Ok(plan) => PlanOutcome::CutPlan(plan),
            Err(refused) => PlanOutcome::CutRefused(refused),
        })
    })
}

struct ApplyPorts<'a> {
    file: FileCutPorts,
    context: &'a RunContext,
    summarize: Box<Summarizer<'a>>,
}

impl CutPorts for ApplyPorts<'_> {
    fn read(&self) -> io::Result<Lines<'_>> {
        self.file.read()
    }

    fn source_path(&self) -> String {
        self.file.source_path()
    }

    fn create(&self, session_id: &str, lines: &[String]) -> io::Result<()> {
        self.file.create(session_id, lines)
    }

    fn read_built(&self, session_id: &str) -> io::Result<Lines<'_>> {
        self.file.read_built(session_id)
    }

    fn discard(&self, session_id: &str) -> io::Result<()> {
        self.file.discard(session_id)
    }

    fn ids(&self) -> &dyn Ids {
        self.context.ids.as_ref()
    }

    fn clock(&self) -> &dyn Clock {
        self.context.clock.as_ref()
    }

    fn cli_version(&self) -> io::Result<String> {
        self.file.cli_version()
    }

    fn summarizer(&self) -> Option<&Summarizer<'_>> {
        Some(&*self.summarize)
    }
}

fn apply(input: &Value, context: &RunContext) -> Result<Value, RunError> {
    typed(input, |input: ApplyCommand| {
        // Preserve host request failures.
        let refused: RefCell<Option<RunError>> = RefCell::new(None);
        let summarize = |standing: Option<&str>, rows: &[String]| -> io::Result<String> {
            let asked = SummarizeRequest {
                standing: standing.map(str::to_owned),
                rows: rows.to_vec(),
            };
            let failure = match context.request(SUMMARIZE, asked.encode()) {
                Ok(Value::String(summary)) => return Ok(summary),
                Ok(_) => RunError::HostRequest("the summary is not text".to_owned()),
                Err(error) => error,
            };
            *refused.borrow_mut() = Some(failure);
            Err(io::Error::other("the host gave no summary"))
        };
        let ports = ApplyPorts {
            file: FileCutPorts {
                source: PathBuf::from(&input.source),
                out: PathBuf::from(&input.out),
                cli_version: input.cli_version,
                keep_out_of: input.keep_out_of.map(PathBuf::from),
            },
            context,
            summarize: Box::new(summarize),
        };
        let outcome = cut_session(&ports, &input.request);
        if let Some(failure) = refused.take() {
            return Err(failure);
        }
        match outcome {
            Ok(Ok(done)) => Ok(CutOutcome::CutDone(done)),
            Ok(Err(refused)) => Ok(CutOutcome::CutRefused(refused)),
            Err(CutError::Io(error)) => Err(error.into()),
            Err(error) => Err(RunError::Failed(error.to_string())),
        }
    })
}
