//! Row key cases.

use nyaaai_agentic::models::row_key::{parse_row_key, row_key, served_id_of, shown_id, shown_key};
use nyaaai_contract::json::Value;
use nyaaai_testkit::cases::Outcome;

use super::shapes::*;

pub fn build(input: RowKeyBuildInput) -> Outcome<RowKeyBuildResult> {
    let keys: Vec<String> = input.identities.iter().map(row_key).collect();
    let back = keys
        .iter()
        .map(|key| parse_row_key(&Value::from(key.as_str())))
        .collect();
    Ok(RowKeyBuildResult { keys, back })
}

pub fn parse(input: RowKeyParseInput) -> Outcome<RowKeyParseResult> {
    Ok(RowKeyParseResult {
        identities: input.values.iter().map(parse_row_key).collect(),
    })
}

pub fn shown(input: RowKeyShownInput) -> Outcome<RowKeyShownResult> {
    Ok(RowKeyShownResult {
        ids: input.identities.iter().map(shown_id).collect(),
        keyed: input.keys.iter().map(|key| shown_key(key)).collect(),
    })
}

pub fn served_id(input: RowKeyServedIdInput) -> Outcome<RowKeyServedIdResult> {
    Ok(RowKeyServedIdResult {
        ids: input
            .identities
            .iter()
            .map(|identity| served_id_of(identity).map(str::to_owned))
            .collect(),
    })
}
