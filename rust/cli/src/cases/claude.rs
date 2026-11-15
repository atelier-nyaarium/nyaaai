//! Claude Code cases.

use nyaaai_operator::claude::cli_version;
use nyaaai_testkit::cases::Outcome;

use super::shapes::*;

pub fn version_in(input: ClaudeVersionInInput) -> Outcome<ClaudeVersionInResult> {
    let found = input
        .texts
        .iter()
        .map(|text| cli_version::version_in(text).map(str::to_owned))
        .collect();
    Ok(ClaudeVersionInResult { found })
}
