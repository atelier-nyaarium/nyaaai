//! Header cases.

use nyaaai_contract::headers::{header_of, response_headers};
use nyaaai_testkit::cases::Outcome;

use super::shapes::*;

pub fn of(input: HeadersOfInput) -> Outcome<HeadersOfResult> {
    Ok(HeadersOfResult {
        value: header_of(&input.headers, &input.name).map(str::to_owned),
    })
}

pub fn response(input: HeadersResponseInput) -> Outcome<HeadersResponseResult> {
    Ok(HeadersResponseResult {
        lists: input
            .lists
            .iter()
            .map(|raw| response_headers(raw))
            .collect(),
    })
}
