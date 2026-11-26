//! Ollama: tags, show and running models, read by hand.

use std::sync::{Arc, LazyLock};

use nyaaai_contract::fetch::Span;
use nyaaai_contract::future::BoxFuture;
use nyaaai_contract::json::Value;
use nyaaai_contract::number::number_of_text;
use nyaaai_contract::shapes::provider::ProviderKindName;
use nyaaai_contract::text::is_space;

use super::join::{MAX_LISTED, READS_AT_ONCE, first_distinct, join_limited, ok};
use super::local::catalog::{
    Loaded, LocalModel, LocalServer, Think, body_of, efforts_of, items_of, put_loaded,
};
use super::local::kind::{LocalKind, LocalKindSpec, local_kind};
use super::readers::{field_of, fields_of, object_field, positive_field, text_field, texts_field};
use crate::dialects::local::LocalReplay;
use crate::provider::kind::KindError;
use crate::transport::{KeyAuth, Transport};

const NUM_CTX: &str = "num_ctx";

fn is_line_end(char: char) -> bool {
    matches!(char, '\n' | '\r' | '\u{2028}' | '\u{2029}')
}

/// The context of the first line reading `num_ctx` and a number; none unless above zero.
pub fn num_ctx_of(text: &str) -> Option<f64> {
    let chars: Vec<char> = text.chars().collect();
    let key: Vec<char> = NUM_CTX.chars().collect();
    let mut from = 0;
    while from <= chars.len() {
        let digits = if chars[from..].starts_with(&key) {
            digits_after(&chars, from + key.len())
        } else {
            None
        };
        if let Some(digits) = digits {
            let mut first = 0;
            while first + 1 < digits.len() && digits[first] == '0' {
                first += 1;
            }
            let found = number_of_text(&digits[first..].iter().collect::<String>());
            return found.filter(|number| *number > 0.0);
        }
        let mut next = from;
        while next < chars.len() && !is_line_end(chars[next]) {
            next += 1;
        }
        if next >= chars.len() {
            return None;
        }
        from = next + 1;
    }
    None
}

/// The digits after blanks, when blanks then the end of the line follow.
fn digits_after(chars: &[char], start: usize) -> Option<Vec<char>> {
    let mut at = start;
    while at < chars.len() && is_space(chars[at]) {
        at += 1;
    }
    if at == start {
        return None;
    }
    let first = at;
    while at < chars.len() && chars[at].is_ascii_digit() {
        at += 1;
    }
    if at == first {
        return None;
    }
    let digits = chars[first..at].to_vec();
    let mut ends = at == chars.len();
    while at < chars.len() && is_space(chars[at]) {
        if is_line_end(chars[at]) {
            ends = true;
        }
        at += 1;
    }
    (ends || at == chars.len()).then_some(digits)
}

/// Local tags only.
pub fn local_tags(raw: &Value) -> Option<Vec<String>> {
    let items = items_of(raw, "models")?;
    Some(
        items
            .iter()
            .filter_map(|item| {
                let fields = fields_of(item)?;
                let name = text_field(fields, "name").filter(|name| !name.is_empty())?;
                text_field(fields, "remote_host")
                    .is_none()
                    .then(|| name.to_owned())
            })
            .collect(),
    )
}

/// The thinking values, when the field holds a list of them.
fn thinking_values(fields: &nyaaai_contract::json::Object) -> Option<Vec<Think>> {
    let thinking = object_field(fields, "thinking")?;
    let list = field_of(thinking, "values")?.as_array()?;
    list.iter()
        .map(|value| match value {
            Value::Bool(flag) => Some(Think::Flag(*flag)),
            Value::String(text) => Some(Think::Text(text.clone())),
            _ => None,
        })
        .collect()
}

/// Completion models only.
pub fn model_of(tag: &str, raw: &Value) -> Option<LocalModel> {
    let fields = fields_of(raw)?;
    let can = texts_field(fields, "capabilities").unwrap_or_default();
    if !can.iter().any(|name| name == "completion") {
        return None;
    }
    let has = |name: &str| can.iter().any(|held| held == name);
    let listed = thinking_values(fields);
    let values = listed.clone().unwrap_or_default();
    // False-only disables thinking.
    let thinks = has("thinking")
        && (listed.is_none() || values.iter().any(|value| *value != Think::Flag(false)));
    let parameters = text_field(fields, "parameters");
    let context = num_ctx_of(parameters.unwrap_or_default());
    let info = object_field(fields, "model_info");
    let arch = info.and_then(|info| field_of(info, "general.architecture"));
    let trained = match (arch, info) {
        (Some(Value::String(arch)), Some(info)) => {
            field_of(info, &format!("{arch}.context_length")).and_then(Value::as_f64)
        }
        _ => None,
    };
    Some(LocalModel {
        tag: tag.to_owned(),
        tools: has("tools"),
        thinking: thinks,
        efforts: if thinks {
            efforts_of(&values)
        } else {
            Vec::new()
        },
        context,
        loaded: None,
        trained: trained.filter(|length| *length > 0.0),
        unprobed: false,
    })
}

/// Loaded context by name, from `/api/ps`; a list that does not read is empty.
pub fn running_of(raw: &Value) -> Loaded {
    let mut contexts = Loaded::new();
    for item in items_of(raw, "models").unwrap_or_default() {
        let Some(fields) = fields_of(item) else {
            continue;
        };
        let Some(name) = text_field(fields, "name") else {
            continue;
        };
        if let Some(context) = positive_field(fields, "context_length") {
            put_loaded(&mut contexts, name, context);
        }
    }
    contexts
}

/// One show answer; a failed read keeps the model known before.
async fn show(
    transport: &Transport,
    tag: &str,
    known: &[LocalModel],
    span: &Span,
) -> Option<LocalModel> {
    let body = body_of("model", tag);
    match transport
        .post_json("/api/show", Some(&body), span, &KeyAuth::None)
        .await
    {
        Ok(raw) => model_of(tag, &raw),
        Err(_) => known.iter().find(|each| each.tag == tag).cloned(),
    }
}

struct OllamaServer {
    transport: Arc<Transport>,
    max_listed: usize,
}

impl LocalServer for OllamaServer {
    fn models<'a>(
        &'a self,
        known: &'a [LocalModel],
        span: &'a Span,
    ) -> BoxFuture<'a, Result<Vec<LocalModel>, KindError>> {
        Box::pin(async move {
            let raw = self
                .transport
                .get_json("/api/tags", span, &KeyAuth::None)
                .await?;
            let Some(tags) = local_tags(&raw) else {
                return Err(KindError::Message(
                    "The model list did not parse.".to_owned(),
                ));
            };
            let tags = first_distinct(tags, Clone::clone, self.max_listed);
            let reads: Vec<_> = tags
                .iter()
                .map(|tag| ok(show(&self.transport, tag, known, span)))
                .collect();
            let shown = join_limited(reads, READS_AT_ONCE).await.unwrap_or_default();
            Ok(shown.into_iter().flatten().collect())
        })
    }

    fn reads_loaded(&self) -> bool {
        true
    }

    fn loaded<'a>(
        &'a self,
        _tag: &'a str,
        span: &'a Span,
    ) -> BoxFuture<'a, Result<Loaded, KindError>> {
        Box::pin(async move {
            let raw = self
                .transport
                .get_json("/api/ps", span, &KeyAuth::None)
                .await?;
            Ok(running_of(&raw))
        })
    }
}

pub fn server(transport: Arc<Transport>) -> Arc<dyn LocalServer> {
    server_limited(transport, MAX_LISTED)
}

/// A server that reads at most `max_listed` distinct tags of a listing.
pub fn server_limited(transport: Arc<Transport>, max_listed: usize) -> Arc<dyn LocalServer> {
    Arc::new(OllamaServer {
        transport,
        max_listed,
    })
}

pub static OLLAMA: LazyLock<Arc<LocalKind>> = LazyLock::new(|| {
    Arc::new(local_kind(LocalKindSpec {
        kind: ProviderKindName::Ollama,
        label: "Ollama",
        url: Some("http://127.0.0.1:11434"),
        optional_key: false,
        replay: Some(LocalReplay::Reasoning),
        chat_path: None,
        base: None,
        terms: None,
        reach: |transport, span| {
            Box::pin(async move {
                let raw = transport
                    .get_json("/api/tags", span, &KeyAuth::None)
                    .await?;
                Ok(local_tags(&raw).map(|tags| tags.len()))
            })
        },
        server,
    }))
});
