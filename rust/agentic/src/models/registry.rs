//! The stored model registry: test records, added rows, hidden families and the lists a pick reads.

use std::collections::HashSet;
use std::sync::LazyLock;

use nyaaai_contract::json::{Object, Record, Value};
use nyaaai_contract::pattern::Pattern;
use nyaaai_contract::shapes::model::{
    Binding, ModelRow, ModelRowCache, ModelUse, ModelView, ModelViewPrice, ModelViewStatus,
    RowIdentity, StoredRegistry, TestRecord, TestRecordCache, TestRecordChecks,
    TestRecordChecksReasoning, TestRecordChecksRecovery, TestRecordChecksSchema,
    TestRecordChecksToolPlus, TestRecordVerifiedPrice,
};
use nyaaai_contract::shapes::provider::ProviderKindName;
use nyaaai_contract::text::trim_text;

use super::curated::MODELS;
use super::row_key::{parse_row_key_text, row_key, served_id_of};
use crate::provider::entry::is_entry_id;

/// One entry's registry, or the merge of several.
#[derive(Debug, Clone, PartialEq)]
pub struct RegistrySettings {
    /// In insertion order.
    pub tested: Vec<(String, TestRecord)>,
    /// Manual IDs, ordered.
    pub added: Vec<String>,
    /// Chat-hidden families and IDs.
    pub hidden: Vec<String>,
}

impl RegistrySettings {
    fn record(&self, key: &str) -> Option<&TestRecord> {
        self.tested
            .iter()
            .find(|(held, _)| held == key)
            .map(|(_, record)| record)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selection {
    pub id: Option<String>,
    /// Configured model unavailable.
    pub marked: bool,
}

/// Chat's gates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gate {
    ToolPlus,
    Recovery,
    Reasoning,
}

impl Gate {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ToolPlus => "toolPlus",
            Self::Recovery => "recovery",
            Self::Reasoning => "reasoning",
        }
    }
}

/// The three settings values a registry is read from.
#[derive(Debug, Clone, Copy, Default)]
pub struct RawRegistry<'a> {
    pub records: Option<&'a Value>,
    pub added: Option<&'a Value>,
    pub hidden: Option<&'a Value>,
}

/// Fallback completion price cap.
const CHEAP_COMPLETION: f64 = 2.0;

const LABEL_CHARS: usize = 200;

static RECORD_DATE: LazyLock<Pattern> =
    LazyLock::new(|| Pattern::compile("^\\d{4}-\\d{2}-\\d{2}$", "").expect("a fixed pattern"));

/// Reject dot segments.
static MODEL_ID: LazyLock<Pattern> = LazyLock::new(|| {
    Pattern::compile("^\\w[\\w.-]*(?:\\/\\w[\\w.:-]*)+$", "").expect("a fixed pattern")
});

fn strings(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|each| each.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

fn is_cost(value: Option<&Value>) -> Option<f64> {
    value
        .and_then(Value::as_f64)
        .filter(|number| number.is_finite() && *number >= 0.0)
}

fn is_family(key: &str) -> bool {
    MODELS.iter().any(|row| row.family == key)
}

fn is_curated(key: &str) -> bool {
    MODELS.iter().any(|row| row.key == key)
}

/// A stored test record; `None` for anything that does not fit.
pub fn parse_test_record(value: &Value) -> Option<TestRecord> {
    let raw = value.as_object()?;
    let label = raw
        .get("label")
        .and_then(Value::as_str)
        .map_or("", trim_text);
    let date = raw.get("date").and_then(Value::as_str)?;
    let cost = raw.get("cost");
    let estimated = raw.get("estimated");
    let checks = raw.get("checks").and_then(Value::as_object);
    let verdict = |key: &str| {
        checks
            .and_then(|held| held.get(key))
            .and_then(Value::as_str)
    };
    let price = raw.get("verifiedPrice").and_then(Value::as_object);
    if label.is_empty() || label.chars().count() > LABEL_CHARS {
        return None;
    }
    if !RECORD_DATE.test(date) {
        return None;
    }
    let cost = is_cost(cost)?;
    let estimated = match estimated {
        None => None,
        Some(Value::Bool(held)) => Some(*held),
        Some(_) => return None,
    };
    let tool_plus = TestRecordChecksToolPlus::parse(verdict("toolPlus")?)?;
    let recovery = TestRecordChecksRecovery::parse(verdict("recovery")?)?;
    let reasoning_check = TestRecordChecksReasoning::parse(verdict("reasoning")?)?;
    let schema = TestRecordChecksSchema::parse(verdict("schema")?)?;
    let reasoning = raw.get("reasoning").and_then(Value::as_bool)?;
    let cache = TestRecordCache::parse(raw.get("cache").and_then(Value::as_str)?)?;
    let prompt = is_cost(price.and_then(|held| held.get("prompt")))?;
    let completion = is_cost(price.and_then(|held| held.get("completion")))?;
    Some(TestRecord {
        label: label.to_owned(),
        date: date.to_owned(),
        cost,
        estimated,
        checks: TestRecordChecks {
            tool_plus,
            recovery,
            reasoning: reasoning_check,
            schema,
        },
        reasoning,
        cache,
        verified_price: TestRecordVerifiedPrice { prompt, completion },
    })
}

pub fn is_model_id(value: &str) -> bool {
    MODEL_ID.test(value)
}

pub fn curated_under(kind: ProviderKindName, id: &str) -> bool {
    MODELS.iter().any(|row| {
        row.bindings
            .get(kind.as_str())
            .is_some_and(|binding| binding.id == id)
    })
}

fn local_to(entry: Option<&str>, key: &str) -> bool {
    matches!(parse_row_key_text(key), Some(RowIdentity::Local(held)) if Some(held.entry.as_str()) == entry)
}

/// The first gate a Chat model fails; Schema is optional.
pub fn failed_gate(checks: &TestRecordChecks) -> Option<Gate> {
    if checks.tool_plus != TestRecordChecksToolPlus::Pass {
        return Some(Gate::ToolPlus);
    }
    if checks.recovery == TestRecordChecksRecovery::Fail {
        return Some(Gate::Recovery);
    }
    if checks.reasoning == TestRecordChecksReasoning::Fail {
        return Some(Gate::Reasoning);
    }
    None
}

pub fn passed(record: &TestRecord) -> bool {
    failed_gate(&record.checks).is_none()
}

/// Hosted rows, or the entry's own local rows.
pub fn tested_under(key: &str, entry: &str) -> bool {
    match parse_row_key_text(key) {
        Some(RowIdentity::Hosted(_)) => true,
        Some(RowIdentity::Local(held)) => held.entry == entry,
        _ => false,
    }
}

fn dedup(items: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    items
        .into_iter()
        .filter(|item| seen.insert(item.clone()))
        .collect()
}

fn object_of(value: Option<&Value>) -> Object {
    value
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default()
}

/// Keeps removed entries' records; hidden keeps listed entries' local keys.
pub fn parse_registry(raw: RawRegistry<'_>, entries: &[&str]) -> StoredRegistry {
    let mut tested: Record<Record<TestRecord>> = Record::new();
    for (entry, records) in object_of(raw.records).iter() {
        let mut valid: Record<TestRecord> = Record::new();
        let mut count = 0;
        for (key, value) in object_of(Some(records)).iter() {
            let record = parse_test_record(value);
            if let Some(record) = record
                && tested_under(key, entry)
            {
                valid.insert(key, record);
                count += 1;
            }
        }
        if is_entry_id(entry) && entry != "__proto__" && count > 0 {
            tested.insert(entry, valid);
        }
    }
    let mut added: Record<Vec<String>> = Record::new();
    for (entry, keys) in object_of(raw.added).iter() {
        let kept: Vec<String> = dedup(strings(Some(keys)))
            .into_iter()
            .filter(|key| {
                tested
                    .get(entry)
                    .and_then(|records| records.get(key))
                    .is_some_and(passed)
            })
            .collect();
        if !kept.is_empty() {
            added.insert(entry, kept);
        }
    }
    let keys: HashSet<&String> = added.values().flatten().collect();
    let hidden: Vec<String> = strings(raw.hidden)
        .into_iter()
        .filter(|each| {
            is_family(each)
                || keys.contains(each)
                || entries.iter().any(|entry| local_to(Some(entry), each))
        })
        .collect();
    StoredRegistry {
        tested,
        added,
        hidden: dedup(hidden),
    }
}

/// No-entry result is empty.
pub fn registry_for(stored: &StoredRegistry, entry: Option<&str>) -> RegistrySettings {
    let records = entry.and_then(|entry| stored.tested.get(entry));
    let tested: Vec<(String, TestRecord)> = records
        .map(|records| {
            records
                .iter()
                .map(|(key, record)| (key.to_owned(), record.clone()))
                .collect()
        })
        .unwrap_or_default();
    let added: Vec<String> = entry
        .and_then(|entry| stored.added.get(entry))
        .cloned()
        .unwrap_or_default();
    let hidden = dedup(
        stored
            .hidden
            .iter()
            .filter(|each| is_family(each) || added.contains(each) || local_to(entry, each))
            .cloned()
            .collect(),
    );
    RegistrySettings {
        tested,
        added,
        hidden,
    }
}

/// Merge with first-entry precedence.
pub fn registry_of(stored: &StoredRegistry, entries: &[&str]) -> RegistrySettings {
    let mut tested: Vec<(String, TestRecord)> = Vec::new();
    let mut added: Vec<String> = Vec::new();
    for entry in entries {
        let one = registry_for(stored, Some(entry));
        for (model, record) in one.tested {
            if !tested.iter().any(|(held, _)| *held == model) {
                tested.push((model, record));
            }
        }
        for model in one.added {
            if !added.contains(&model) {
                added.push(model);
            }
        }
    }
    let hidden = dedup(
        stored
            .hidden
            .iter()
            .filter(|each| {
                is_family(each)
                    || added.contains(each)
                    || entries.iter().any(|entry| local_to(Some(entry), each))
            })
            .cloned()
            .collect(),
    );
    RegistrySettings {
        tested,
        added,
        hidden,
    }
}

/// Bound to the testing entry's kind; `None` for a curated identity.
pub fn row_of(
    identity: &RowIdentity,
    record: &TestRecord,
    kind: ProviderKindName,
) -> Option<ModelRow> {
    let key = row_key(identity);
    let served = served_id_of(identity)?;
    let mut bindings = Record::new();
    bindings.insert(
        kind.as_str(),
        Binding {
            id: served.to_owned(),
        },
    );
    Some(ModelRow {
        family: key.clone(),
        key,
        bindings,
        label: record.label.clone(),
        reasoning: record.reasoning,
        schema: record.checks.schema == TestRecordChecksSchema::Pass,
        cache: if record.cache == TestRecordCache::Explicit {
            ModelRowCache::Explicit
        } else {
            ModelRowCache::Implicit
        },
        verified: record.date.clone(),
        verified_price: nyaaai_contract::shapes::model::ModelRowVerifiedPrice {
            prompt: record.verified_price.prompt,
            completion: record.verified_price.completion,
        },
        identity: identity.clone(),
    })
}

/// Retains removed thread models; first entry wins a shared hosted row.
pub fn registry_rows(
    stored: &StoredRegistry,
    entries: &[(&str, ProviderKindName)],
) -> Vec<ModelRow> {
    let mut rows: Vec<ModelRow> = MODELS.clone();
    for (id, kind) in entries {
        let settings = registry_for(stored, Some(id));
        let removed = settings
            .tested
            .iter()
            .map(|(key, _)| key)
            .filter(|key| !settings.added.contains(key));
        let keys: Vec<&String> = settings.added.iter().chain(removed).collect();
        for key in keys {
            let record = settings.record(key);
            let identity = parse_row_key_text(key);
            let (Some(record), Some(identity)) = (record, identity) else {
                continue;
            };
            if !passed(record) || rows.iter().any(|row| row.key == *key) {
                continue;
            }
            let hosted = matches!(&identity, RowIdentity::Hosted(held)
                if held.provider == *kind && !curated_under(*kind, &held.id));
            let local = matches!(identity, RowIdentity::Local(_));
            if (hosted || local)
                && let Some(row) = row_of(&identity, record, *kind)
            {
                rows.push(row);
            }
        }
    }
    rows
}

pub fn offered(row: &ModelRow, settings: &RegistrySettings) -> bool {
    is_curated(&row.key) || settings.added.contains(&row.key)
}

/// The models a pick can list.
pub fn model_list<'a>(
    views: &'a [ModelView],
    use_: ModelUse,
    settings: &RegistrySettings,
) -> Vec<&'a ModelView> {
    let fits = |row: &ModelRow| {
        if use_ == ModelUse::Chat {
            !settings.hidden.contains(&row.family)
        } else {
            row.schema
        }
    };
    views
        .iter()
        .filter(|view| {
            offered(&view.row, settings)
                && view.status != ModelViewStatus::Retired
                && view.status != ModelViewStatus::NoZdrHost
                && fits(&view.row)
        })
        .collect()
}

/// Prefer catalog price; unknown is never cheap.
fn completion_price(view: &ModelView) -> f64 {
    match &view.price {
        Some(ModelViewPrice::Free) => 0.0,
        Some(ModelViewPrice::Unknown) => f64::INFINITY,
        Some(ModelViewPrice::Price(price)) => price.completion.1,
        None => view.row.verified_price.completion,
    }
}

/// Marked cheap fallback.
pub fn select(wanted: &str, list: &[ModelView]) -> Selection {
    if list.iter().any(|view| view.row.key == wanted) {
        return Selection {
            id: Some(wanted.to_owned()),
            marked: false,
        };
    }
    Selection {
        id: list
            .iter()
            .find(|view| completion_price(view) <= CHEAP_COMPLETION)
            .map(|view| view.row.key.clone()),
        marked: true,
    }
}
