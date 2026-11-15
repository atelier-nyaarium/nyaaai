//! CLI. Run: cargo run --manifest-path rust/Cargo.toml -p nyaaai-cli -- serve.

mod adapter;
#[cfg(feature = "cases")]
mod case_dispatch;
#[cfg(feature = "cases")]
mod case_handlers;
#[cfg(feature = "cases")]
mod case_shapes;
mod commands;
mod cut;

use std::io;
use std::process::ExitCode;

use adapter::{RunContext, RunError};
use nyaaai_contract::json::Value;

const SERVE_USAGE: &str = "nyaaai serve";

/// Gate commands use `case:a.b`.
#[cfg(feature = "cases")]
fn run(name: &str, input: &Value, context: &RunContext) -> Option<Result<Value, RunError>> {
    if let Some(kind) = name.strip_prefix("case:") {
        return case_dispatch::dispatch(kind, input).map(|held| held.map_err(RunError::Failed));
    }
    commands::run(name, input, context).or_else(|| case_handlers::run(name, input, context))
}

#[cfg(not(feature = "cases"))]
fn run(name: &str, input: &Value, context: &RunContext) -> Option<Result<Value, RunError>> {
    commands::run(name, input, context)
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let outcome = match args.split_first() {
        Some((command, rest)) if command == "cut" => cut::run(rest),
        Some((command, [])) if command == "serve" => Ok(ExitCode::from(adapter::serve(
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
