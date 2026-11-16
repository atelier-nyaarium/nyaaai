//! Shape round trips.

use nyaaai_contract::json::{Value, parse_json, stringify_json};
use nyaaai_contract::shape::PathPart;
use nyaaai_contract::shapes::frames;
use nyaaai_testkit::cases::Outcome;

use super::normalizers;
use super::shapes::*;

pub fn round_trip(input: ContractRoundTripInput) -> Outcome<ContractRoundTripResult> {
    let normalize = |shape: &str, value: &Value| normalizers::normalize(&input.set, shape, value);
    let unknown = || format!("no shape {}.{}", input.set, input.shape);
    if normalize(&input.shape, &Value::Null).is_none() {
        return Err(unknown());
    }
    let refusal =
        |path: Vec<frames::PathPart>, message: Option<String>| ContractRoundTripResultResultsItem {
            ok: false,
            text: None,
            path: Some(path),
            message,
        };
    let mut results = Vec::new();
    for text in &input.texts {
        let Some(value) = parse_json(text) else {
            results.push(refusal(Vec::new(), None));
            continue;
        };
        results.push(match normalize(&input.shape, &value) {
            None => return Err(unknown()),
            Some(Ok(held)) => ContractRoundTripResultResultsItem {
                ok: true,
                text: Some(stringify_json(&held)),
                path: None,
                message: None,
            },
            Some(Err(refused)) => refusal(
                refused
                    .path
                    .into_iter()
                    .map(|part| match part {
                        PathPart::Key(key) => frames::PathPart::Text(key),
                        PathPart::Index(index) => frames::PathPart::Integer(index as u64),
                    })
                    .collect(),
                refused.fixed.then(|| refused.message.to_owned()),
            ),
        });
    }
    Ok(ContractRoundTripResult { results })
}
