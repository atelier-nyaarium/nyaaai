//! Provider entry cases.

use std::sync::Arc;

use nyaaai_agentic::provider::entry::{
    bound_for, bound_key as bound, is_entry_id, key_binding_of, parse_entries, parse_entry,
    read_bound_key as read, same_origin as same, secret_key as key_of,
};
use nyaaai_agentic::provider::host::key_may_travel;
use nyaaai_agentic::provider::kind::{KINDS, ProviderKind};
use nyaaai_contract::entry_keys::KeyAnswer;
use nyaaai_contract::json::Value;
use nyaaai_contract::provider_list::ProviderList;
use nyaaai_contract::shapes::provider::{EntryFields, ProviderEntry};
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

/// The entry the list holds with that id.
pub(super) fn listed(list: &dyn ProviderList, id: &str) -> Option<ProviderEntry> {
    list.entries().into_iter().find(|each| each.id == id)
}

/// A planted key, bound as given, else to its entry: its kind name when it binds none, the empty text with no entry.
pub(super) fn planted(
    key: &str,
    bound_to: Option<&str>,
    entry: Option<&ProviderEntry>,
) -> KeyAnswer {
    let bound_to = match (bound_to, entry) {
        (Some(given), _) => given.to_owned(),
        (None, None) => String::new(),
        (None, Some(entry)) => {
            let fields = EntryFields {
                kind: entry.kind,
                label: entry.label.clone(),
                plan: entry.plan,
                url: entry.url.clone(),
                extra_usage: entry.extra_usage,
            };
            key_binding_of(&fields, &KINDS).unwrap_or_else(|| entry.kind.as_str().to_owned())
        }
    };
    KeyAnswer::Key {
        key: key.to_owned(),
        bound_to,
    }
}

pub fn binding(input: EntryBindingInput) -> Outcome<EntryBindingResult> {
    Ok(EntryBindingResult {
        rows: input
            .rows
            .iter()
            .map(|row| {
                let binding = key_binding_of(row, &KINDS);
                EntryBindingResultRowsItem {
                    for_: binding.as_deref().map(|held| bound_for(held, &KINDS)),
                    binding,
                }
            })
            .collect(),
    })
}

pub fn key_for(input: EntryKeyForInput) -> Outcome<EntryKeyForResult> {
    let table: Vec<Arc<dyn ProviderKind>> = KINDS
        .iter()
        .filter(|held| input.kinds.contains(&held.kind()))
        .cloned()
        .collect();
    Ok(EntryKeyForResult {
        for_: input
            .bindings
            .iter()
            .map(|binding| bound_for(binding, &table))
            .collect(),
    })
}

pub fn bound_key(input: EntryBoundKeyInput) -> Outcome<EntryBoundKeyResult> {
    Ok(EntryBoundKeyResult {
        texts: input
            .rows
            .iter()
            .map(|row| bound(&row.fields, &row.key, &KINDS))
            .collect(),
    })
}

pub fn read_bound_key(input: EntryReadBoundKeyInput) -> Outcome<EntryReadBoundKeyResult> {
    Ok(EntryReadBoundKeyResult {
        answers: input
            .texts
            .iter()
            .map(|text| match read(text) {
                KeyAnswer::Key { key, bound_to } => {
                    EntryReadBoundKeyResultAnswersItem::Key(EntryReadBoundKeyResultAnswersItemKey {
                        key,
                        bound_to,
                    })
                }
                _ => EntryReadBoundKeyResultAnswersItem::None(
                    EntryReadBoundKeyResultAnswersItemNone {},
                ),
            })
            .collect(),
    })
}

pub fn travel(input: EntryTravelInput) -> Outcome<EntryTravelResult> {
    Ok(EntryTravelResult {
        travels: input.urls.iter().map(|url| key_may_travel(url)).collect(),
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
