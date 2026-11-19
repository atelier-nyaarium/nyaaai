//! Provider entry ids, URL rules and the lenient stored-list reader.

use std::sync::LazyLock;

use nyaaai_contract::json::Value;
use nyaaai_contract::pattern::Pattern;
use nyaaai_contract::shapes::provider::{ProviderEntry, ProviderKindName, ProviderPlan};
use nyaaai_contract::text::{text_length, trim_text};
use nyaaai_contract::url::{UrlParts, parse_url};

pub const PROVIDER_KINDS: [ProviderKindName; 6] = ProviderKindName::ALL;

pub const PROVIDER_PLANS: [ProviderPlan; 2] = ProviderPlan::ALL;

const URL_KINDS: [ProviderKindName; 3] = [
    ProviderKindName::Ollama,
    ProviderKindName::Lmstudio,
    ProviderKindName::Llamacpp,
];

static ENTRY_ID: LazyLock<Pattern> =
    LazyLock::new(|| Pattern::compile("^[\\w-]{1,64}$", "").expect("a fixed pattern"));

const LABEL_CHARS: usize = 120;

/// A word-character id; `__proto__` is no id.
pub fn is_entry_id(value: &str) -> bool {
    value != "__proto__" && ENTRY_ID.test(value)
}

/// The entry's secret key under the consumer's namespace.
pub fn secret_key(id: &str, namespace: &str) -> String {
    format!("{namespace}.provider.{id}")
}

/// HTTP(S), anonymous, no query.
fn web_url(value: Option<&str>) -> Option<UrlParts> {
    let url = parse_url(trim_text(value?))?;
    let web = url.scheme == "http" || url.scheme == "https";
    let plain = url.query.is_empty() && url.fragment.is_empty();
    let anonymous = url.username.is_empty() && url.password.is_empty();
    (web && plain && anonymous && !url.host.is_empty()).then_some(url)
}

fn server_origin(value: Option<&str>) -> Option<String> {
    let url = web_url(value)?;
    (url.path == "/").then_some(url.origin)
}

/// Path kept, no trailing slash.
fn server_base(value: Option<&str>) -> Option<String> {
    let url = web_url(value)?;
    let mut base = format!("{}{}", url.origin, url.path);
    while base.ends_with('/') {
        base.pop();
    }
    Some(base)
}

/// Both URLs parse and share an origin.
pub fn same_origin(left: Option<&str>, right: Option<&str>) -> bool {
    let origin = |url: Option<&str>| web_url(url).map(|parts| parts.origin);
    let held = origin(left);
    held.is_some() && held == origin(right)
}

/// Parse a provider entry; the label counts UTF-16 units.
pub fn parse_entry(value: &Value) -> Option<ProviderEntry> {
    let raw = value.as_object()?;
    let kind = raw
        .get("kind")
        .and_then(Value::as_str)
        .and_then(ProviderKindName::parse);
    let label = raw
        .get("label")
        .and_then(Value::as_str)
        .map_or("", trim_text);
    let id = raw
        .get("id")
        .and_then(Value::as_str)
        .filter(|id| is_entry_id(id));
    let (Some(id), Some(kind)) = (id, kind) else {
        return None;
    };
    if label.is_empty() || text_length(label) > LABEL_CHARS {
        return None;
    }
    let mut entry = ProviderEntry {
        id: id.to_owned(),
        kind,
        label: label.to_owned(),
        plan: None,
        url: None,
        extra_usage: None,
    };
    if kind == ProviderKindName::Opencode {
        let plan = raw
            .get("plan")
            .and_then(Value::as_str)
            .and_then(ProviderPlan::parse)?;
        entry.plan = Some(plan);
        if plan == ProviderPlan::Go
            && let Some(Value::Bool(extra)) = raw.get("extraUsage")
        {
            entry.extra_usage = Some(*extra);
        }
    }
    let text_of = |value: &Value| value.as_str().map(str::to_owned);
    if URL_KINDS.contains(&kind)
        && let Some(url) = raw.get("url")
    {
        entry.url = Some(server_origin(text_of(url).as_deref())?);
    }
    if kind == ProviderKindName::Compatible {
        let url = raw.get("url").and_then(text_of);
        entry.url = Some(server_base(url.as_deref())?);
    }
    Some(entry)
}

/// The entries that read, in order; a repeated id keeps its first.
pub fn parse_entries(value: &Value) -> Vec<ProviderEntry> {
    let Some(items) = value.as_array() else {
        return Vec::new();
    };
    let mut entries: Vec<ProviderEntry> = Vec::new();
    for item in items {
        if let Some(entry) = parse_entry(item)
            && !entries.iter().any(|held| held.id == entry.id)
        {
            entries.push(entry);
        }
    }
    entries
}
