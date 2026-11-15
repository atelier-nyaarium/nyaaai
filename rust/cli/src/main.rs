//! CLI. Run from `rust/`: `cargo run -p nyaaai-cli -- serve`.

#[cfg(feature = "cases")]
mod cases;
mod commands;
mod cut;
#[cfg(feature = "cases")]
mod test_commands;

use std::io;
use std::process::ExitCode;

use nyaaai_adapter::runtime::{RunContext, RunError, serve};
use nyaaai_contract::json::Value;

const SERVE_USAGE: &str = "nyaaai serve";

/// Project root: CWD at start.
#[cfg(feature = "cases")]
fn project_root() -> &'static std::path::Path {
    static ROOT: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();
    ROOT.get_or_init(|| std::env::current_dir().unwrap_or_default())
}

/// Gate uses `case:a.b`.
#[cfg(feature = "cases")]
fn run(name: &str, input: &Value, context: &RunContext) -> Option<Result<Value, RunError>> {
    if let Some(kind) = name.strip_prefix("case:") {
        return cases::dispatch::dispatch(kind, input, project_root())
            .map(|held| held.map_err(RunError::Failed));
    }
    commands::run(name, input, context).or_else(|| test_commands::run(name, input, context))
}

#[cfg(not(feature = "cases"))]
fn run(name: &str, input: &Value, context: &RunContext) -> Option<Result<Value, RunError>> {
    commands::run(name, input, context)
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    #[cfg(feature = "cases")]
    project_root();
    let outcome = match args.split_first() {
        Some((command, rest)) if command == "cut" => cut::run(rest),
        Some((command, [])) if command == "serve" => Ok(ExitCode::from(serve(
            io::stdin().lock(),
            Box::new(io::stdout()),
            run,
        ))),
        _ => Err(format!("usage:\n  {}\n  {SERVE_USAGE}", cut::USAGE)),
    };
    match outcome {
        Ok(code) => code,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}
