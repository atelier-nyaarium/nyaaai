//! Session file cases.

use nyaaai_contract::base64;
use nyaaai_operator::claude::session_file;
use nyaaai_testkit::cases::Outcome;

use super::shapes::*;

pub fn bytes_of(text: &str) -> Outcome<Vec<u8>> {
    base64::base64_decode(text).ok_or_else(|| "the source is not base64".to_owned())
}

pub fn lines_of(input: SessionFileLinesOfInput) -> Outcome<SessionFileLinesOfResult> {
    let rows = session_file::lines_of(&bytes_of(&input.base64)?);
    Ok(SessionFileLinesOfResult { rows })
}
