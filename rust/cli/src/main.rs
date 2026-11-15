//! nyaaai command line tool.

mod cut;
mod render;

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let outcome = match args.split_first() {
        Some((command, rest)) if command == "cut" => cut::run(rest),
        _ => Err(format!("usage:\n  {}", cut::USAGE)),
    };
    match outcome {
        Ok(code) => code,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}
