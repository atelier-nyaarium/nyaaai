//! JavaScript stand-in cases.

use nyaaai_contract::js::to_js_string;
use nyaaai_contract::json::parse_json;
use nyaaai_testkit::cases::Outcome;

use super::shapes::*;

pub fn string(input: JsStringInput) -> Outcome<JsStringResult> {
    let strings = input
        .texts
        .iter()
        .map(|text| {
            let value = parse_json(text).ok_or("a value text is not JSON")?;
            Ok(to_js_string(Some(&value)))
        })
        .collect::<Outcome<_>>()?;
    Ok(JsStringResult { strings })
}
