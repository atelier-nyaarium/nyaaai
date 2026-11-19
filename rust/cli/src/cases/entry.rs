//! Provider entry cases.

use nyaaai_agentic::provider::entry::{
    is_entry_id, parse_entries, parse_entry, same_origin as same, secret_key as key_of,
};
use nyaaai_contract::json::Value;
use nyaaai_testkit::cases::Outcome;

use super::shapes::*;

pub fn parse(input: EntryParseInput) -> Outcome<EntryParseResult> {
    Ok(EntryParseResult {
        entry: parse_entry(&input.value),
    })
}

pub fn parse_list(input: EntryParseListInput) -> Outcome<EntryParseListResult> {
    Ok(EntryParseListResult {
        entries: parse_entries(&input.value),
    })
}

pub fn same_origin(input: EntrySameOriginInput) -> Outcome<EntrySameOriginResult> {
    Ok(EntrySameOriginResult {
        same: input
            .rows
            .iter()
            .map(|row| same(row.left.as_deref(), row.right.as_deref()))
            .collect(),
    })
}

pub fn id(input: EntryIdInput) -> Outcome<EntryIdResult> {
    Ok(EntryIdResult {
        valid: input
            .values
            .iter()
            .map(|value| matches!(value, Value::String(text) if is_entry_id(text)))
            .collect(),
    })
}

pub fn secret_key(input: EntrySecretKeyInput) -> Outcome<EntrySecretKeyResult> {
    Ok(EntrySecretKeyResult {
        keys: input
            .ids
            .iter()
            .map(|id| key_of(id, &input.namespace))
            .collect(),
    })
}
