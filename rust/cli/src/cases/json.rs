//! JSON spelling cases.

use nyaaai_contract::json::{canonical_json, parse_json, stringify_json};
use nyaaai_testkit::cases::Outcome;

use super::shapes::*;

pub fn round_trip(input: JsonRoundTripInput) -> Outcome<JsonRoundTripResult> {
    let results = input
        .texts
        .iter()
        .map(|text| {
            let text = parse_json(text).map(|value| stringify_json(&value));
            JsonRoundTripResultResultsItem {
                ok: text.is_some(),
                text,
            }
        })
        .collect();
    Ok(JsonRoundTripResult { results })
}

pub fn canonical(input: JsonCanonicalInput) -> Outcome<JsonCanonicalResult> {
    let texts = input
        .texts
        .iter()
        .map(|text| parse_json(text).map(|value| canonical_json(&value)))
        .collect();
    Ok(JsonCanonicalResult { texts })
}
