//! Model registry cases.

use nyaaai_agentic::models::curated::MODELS;
use nyaaai_agentic::models::registry::{
    RawRegistry, RegistrySettings, curated_under, failed_gate, is_model_id, model_list,
    offered as is_offered, parse_registry, parse_test_record, passed as has_passed, registry_for,
    registry_of, registry_rows, select as pick, tested_under,
};
use nyaaai_agentic::models::view::unknown_view;
use nyaaai_contract::json::{Record, Value};
use nyaaai_contract::shape::Shape;
use nyaaai_contract::shapes::model::{
    ModelRow, ModelRowCache, ModelRowVerifiedPrice, ModelView, ModelViewPrice, ModelViewStatus,
    RowIdentity, RowIdentityCurated, TestRecord, TestRecordCache, TestRecordChecks,
    TestRecordVerifiedPrice,
};
use nyaaai_contract::shapes::provider::ProviderKindName;
use nyaaai_testkit::cases::Outcome;

use super::shapes::*;

/// One generated shape as another with the same wire form.
fn convert<From: Shape, To: Shape>(held: &From) -> Outcome<To> {
    To::decode(&held.encode()).map_err(|refused| refused.to_string())
}

fn raw_of<'a>(
    records: &'a Option<Value>,
    added: &'a Option<Value>,
    hidden: &'a Option<Value>,
) -> RawRegistry<'a> {
    RawRegistry {
        records: records.as_ref(),
        added: added.as_ref(),
        hidden: hidden.as_ref(),
    }
}

fn settings_tested(settings: &RegistrySettings) -> Record<TestRecord> {
    let mut tested = Record::new();
    for (key, record) in &settings.tested {
        tested.insert(key.as_str(), record.clone());
    }
    tested
}

pub fn test_record(input: RegistryTestRecordInput) -> Outcome<RegistryTestRecordResult> {
    Ok(RegistryTestRecordResult {
        records: input.values.iter().map(parse_test_record).collect(),
    })
}

pub fn parse(input: RegistryParseInput) -> Outcome<RegistryParseResult> {
    let entries: Vec<&str> = input.entries.iter().map(|each| each.id.as_str()).collect();
    let raw = raw_of(&input.raw.records, &input.raw.added, &input.raw.hidden);
    Ok(RegistryParseResult {
        stored: parse_registry(raw, &entries),
    })
}

pub fn scoped(input: RegistryScopedInput) -> Outcome<RegistryScopedResult> {
    let ids: Vec<&str> = input.entries.iter().map(|each| each.id.as_str()).collect();
    let raw = raw_of(&input.raw.records, &input.raw.added, &input.raw.hidden);
    let stored = parse_registry(raw, &ids);
    let scoped = input
        .scopes
        .iter()
        .map(|scope| {
            let settings = registry_for(&stored, scope.as_deref());
            RegistryScopedResultScopedItem {
                tested: settings_tested(&settings),
                added: settings.added.clone(),
                hidden: settings.hidden.clone(),
            }
        })
        .collect();
    let merged = registry_of(&stored, &ids);
    Ok(RegistryScopedResult {
        scoped,
        merged: RegistryScopedResultMerged {
            tested: settings_tested(&merged),
            added: merged.added.clone(),
            hidden: merged.hidden.clone(),
        },
    })
}

fn listed(entries: &[(String, ProviderKindName)]) -> Vec<(&str, ProviderKindName)> {
    entries
        .iter()
        .map(|(id, kind)| (id.as_str(), *kind))
        .collect()
}

pub fn rows(input: RegistryRowsInput) -> Outcome<RegistryRowsResult> {
    let entries: Vec<(String, ProviderKindName)> = input
        .entries
        .iter()
        .map(|each| (each.id.clone(), each.kind))
        .collect();
    let ids: Vec<&str> = entries.iter().map(|(id, _)| id.as_str()).collect();
    let raw = raw_of(&input.raw.records, &input.raw.added, &input.raw.hidden);
    let stored = parse_registry(raw, &ids);
    let all = registry_rows(&stored, &listed(&entries));
    let curated_first = all.len() >= MODELS.len() && all[..MODELS.len()] == MODELS[..];
    Ok(RegistryRowsResult {
        curated_first,
        rest: all.into_iter().skip(MODELS.len()).collect(),
    })
}

pub fn offered(input: RegistryOfferedInput) -> Outcome<RegistryOfferedResult> {
    let entries: Vec<(String, ProviderKindName)> = input
        .entries
        .iter()
        .map(|each| (each.id.clone(), each.kind))
        .collect();
    let ids: Vec<&str> = entries.iter().map(|(id, _)| id.as_str()).collect();
    let raw = raw_of(&input.raw.records, &input.raw.added, &input.raw.hidden);
    let stored = parse_registry(raw, &ids);
    let held = registry_rows(&stored, &listed(&entries));
    let settings = registry_for(&stored, input.scope.as_deref());
    Ok(RegistryOfferedResult {
        offered: input
            .keys
            .iter()
            .map(|key| {
                held.iter()
                    .find(|row| row.key == *key)
                    .map(|row| is_offered(row, &settings))
            })
            .collect(),
    })
}

pub fn list(input: RegistryListInput) -> Outcome<RegistryListResult> {
    let entries: Vec<(String, ProviderKindName)> = input
        .entries
        .iter()
        .map(|each| (each.id.clone(), each.kind))
        .collect();
    let ids: Vec<&str> = entries.iter().map(|(id, _)| id.as_str()).collect();
    let raw = raw_of(&input.raw.records, &input.raw.added, &input.raw.hidden);
    let stored = parse_registry(raw, &ids);
    let held = registry_rows(&stored, &listed(&entries));
    let settings = registry_for(&stored, input.scope.as_deref());
    let views = input
        .views
        .iter()
        .map(|view| {
            let row = held
                .iter()
                .find(|each| each.key == view.key)
                .ok_or_else(|| format!("No row {}.", view.key))?;
            let mut made = unknown_view(row);
            made.status = convert::<_, ModelViewStatus>(&view.status)?;
            Ok(made)
        })
        .collect::<Outcome<Vec<ModelView>>>()?;
    Ok(RegistryListResult {
        keys: model_list(&views, input.use_, &settings)
            .into_iter()
            .map(|view| view.row.key.clone())
            .collect(),
    })
}

pub fn select(input: RegistrySelectInput) -> Outcome<RegistrySelectResult> {
    let views = input
        .list
        .iter()
        .map(|item| {
            let row = ModelRow {
                key: item.key.clone(),
                bindings: Record::new(),
                family: item.key.clone(),
                label: item.key.clone(),
                reasoning: false,
                schema: true,
                cache: ModelRowCache::Implicit,
                verified: "2026-01-01".to_owned(),
                verified_price: ModelRowVerifiedPrice {
                    prompt: 0.0,
                    completion: item.verified_completion,
                },
                identity: RowIdentity::Curated(RowIdentityCurated {
                    key: item.key.clone(),
                }),
            };
            let mut view = unknown_view(&row);
            view.status = ModelViewStatus::Ok;
            view.price = match &item.price {
                None => None,
                Some(price) => Some(convert::<_, ModelViewPrice>(price)?),
            };
            Ok(view)
        })
        .collect::<Outcome<Vec<ModelView>>>()?;
    let picked = pick(&input.wanted, &views);
    Ok(RegistrySelectResult {
        id: picked.id,
        marked: picked.marked,
    })
}

pub fn passed(input: RegistryPassedInput) -> Outcome<RegistryPassedResult> {
    let checks = input
        .rows
        .iter()
        .map(|row| {
            Ok(TestRecordChecks {
                tool_plus: convert(&row.tool_plus)?,
                recovery: convert(&row.recovery)?,
                reasoning: convert(&row.reasoning)?,
                schema: convert(&row.schema)?,
            })
        })
        .collect::<Outcome<Vec<_>>>()?;
    let record = |checks: &TestRecordChecks| TestRecord {
        label: "L".to_owned(),
        date: "2026-01-01".to_owned(),
        cost: 0.0,
        estimated: None,
        checks: checks.clone(),
        reasoning: true,
        cache: TestRecordCache::None,
        verified_price: TestRecordVerifiedPrice {
            prompt: 0.0,
            completion: 0.0,
        },
    };
    Ok(RegistryPassedResult {
        passed: checks
            .iter()
            .map(|each| has_passed(&record(each)))
            .collect(),
        gates: checks
            .iter()
            .map(|each| failed_gate(each).map(|gate| gate.as_str().to_owned()))
            .collect(),
    })
}

pub fn ids(input: RegistryIdsInput) -> Outcome<RegistryIdsResult> {
    Ok(RegistryIdsResult {
        model_ids: input
            .model_ids
            .iter()
            .map(|value| matches!(value, Value::String(text) if is_model_id(text)))
            .collect(),
        curated: input
            .curated
            .iter()
            .map(|each| curated_under(each.kind, &each.id))
            .collect(),
        tested: input
            .tested
            .iter()
            .map(|each| tested_under(&each.key, &each.entry))
            .collect(),
    })
}
