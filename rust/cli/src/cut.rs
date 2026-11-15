//! Read-only session cutter.

use std::path::PathBuf;
use std::process::ExitCode;

use nyaaai_contract::json::{Value, stringify_json};
use nyaaai_contract::shape::Shape;
use nyaaai_operator::claude::cli_version::installed_version;
use nyaaai_operator::claude::session_cut::{CutMode, cut_session, request_for};
use nyaaai_operator::claude::session_file::FileCutPorts;

pub const USAGE: &str = "nyaaai cut <trim|clear|compact> --source <file> --out <folder> [--target <uuid>] \
[--through <uuid>] [--end-before <text>] [--summary <text>] [--live-tail] [--cli-version <version>]";

const REFUSED: u8 = 2;

pub fn run(args: &[String]) -> Result<ExitCode, String> {
    let mut args = args.iter();
    let mode = args
        .next()
        .and_then(|mode| CutMode::parse(mode))
        .ok_or(USAGE)?;
    let mut request = request_for(mode);
    let mut source: Option<PathBuf> = None;
    let mut out: Option<PathBuf> = None;
    let mut cli_version: Option<String> = None;
    while let Some(flag) = args.next() {
        if flag == "--live-tail" {
            request.live_tail = Some(true);
            continue;
        }
        let value = args.next().ok_or(format!("{flag} needs a value"))?.clone();
        match flag.as_str() {
            "--source" => source = Some(PathBuf::from(value)),
            "--out" => out = Some(PathBuf::from(value)),
            "--target" => request.target_uuid = Some(value),
            "--through" => request.through_uuid = Some(value),
            "--end-before" => request.end_before = Some(value),
            "--summary" => request.summary_text = Some(value),
            "--cli-version" => cli_version = Some(value),
            _ => return Err(format!("unknown flag {flag}")),
        }
    }
    let ports = FileCutPorts {
        source: source.ok_or("--source is needed")?,
        out: out.ok_or("--out is needed")?,
        cli_version: match cli_version {
            Some(version) => version,
            None => installed_version().map_err(|error| format!("claude --version: {error}"))?,
        },
        keep_out_of: None,
    };
    match cut_session(&ports, &request).map_err(|error| error.to_string())? {
        Ok(done) => {
            let file = ports.built(&done.session_id);
            let mut said = done.encode();
            if let Value::Object(held) = &mut said {
                held.insert("file", Value::from(file.to_string_lossy().into_owned()));
            }
            println!("{}", stringify_json(&said));
            Ok(ExitCode::SUCCESS)
        }
        Err(refused) => {
            println!("{}", stringify_json(&refused.encode()));
            Ok(ExitCode::from(REFUSED))
        }
    }
}
