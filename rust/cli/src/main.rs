//! NyaaAI CLI entry point.

mod cases;
mod cut;

use std::process::ExitCode;

const USAGE: &str = "nyaaai cases <file-or-folder>...";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let outcome = match args.split_first() {
        Some((command, rest)) if command == "cases" => cases::run(rest).map(|()| ExitCode::SUCCESS),
        Some((command, rest)) if command == "cut" => cut::run(rest),
        _ => Err(format!("usage:\n  {USAGE}\n  {}", cut::USAGE)),
    };
    match outcome {
        Ok(code) => code,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}
