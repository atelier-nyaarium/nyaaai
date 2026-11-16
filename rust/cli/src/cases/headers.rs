//! Header cases.

use nyaaai_contract::headers::header_of;
use nyaaai_testkit::cases::Outcome;

use super::shapes::*;

pub fn of(input: HeadersOfInput) -> Outcome<HeadersOfResult> {
    Ok(HeadersOfResult {
        value: header_of(&input.headers, &input.name).map(str::to_owned),
    })
}
