//! Base64 cases.

use nyaaai_contract::base64;
use nyaaai_testkit::cases::Outcome;

use super::shapes::*;

pub fn decode(input: Base64DecodeInput) -> Outcome<Base64DecodeResult> {
    let texts = input
        .texts
        .iter()
        .map(|text| base64::base64_decode(text).map(|bytes| base64::base64_encode(&bytes)))
        .collect();
    Ok(Base64DecodeResult { texts })
}
