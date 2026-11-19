//! OpenAI-compatible servers: the model list, the network a host sits on, the request root.

use std::sync::{Arc, LazyLock};

use nyaaai_contract::cancel::CancelToken;
use nyaaai_contract::future::BoxFuture;
use nyaaai_contract::json::Value;
use nyaaai_contract::pattern::Pattern;
use nyaaai_contract::shapes::provider::{EntryFields, ProviderKindName};
use nyaaai_contract::url::parse_url;

use super::local::catalog::{LocalModel, LocalServer};
use super::local::kind::{HOME, LocalKind, LocalKindSpec, Terms, TermsPrice, local_kind};
use super::readers::{data_list, fields_of, positive_field, text_field};
use crate::provider::kind::KindError;
use crate::transport::{KeyAuth, Transport};

/// A model of the list; the widest of the three context fields the servers use.
#[derive(Debug, Clone, PartialEq)]
pub struct ListedModel {
    pub id: String,
    pub context: Option<f64>,
}

const LIST: &str = "/models";

/// Loopback, private ranges, LAN names; a host as the URL parser wrote it.
static PRIVATE_HOST: LazyLock<Pattern> = LazyLock::new(|| {
    Pattern::compile(
        "^(localhost|0\\.0\\.0\\.0|(127|10)\\.[\\d.]+|192\\.168\\.[\\d.]+|172\\.(1[6-9]|2\\d|3[01])\\.[\\d.]+|169\\.254\\.[\\d.]+|100\\.(6[4-9]|[7-9]\\d|1[01]\\d|12[0-7])\\.[\\d.]+|\\[(::1?|f[cd][0-9a-f]{2}:[0-9a-f:]*|fe[89ab][0-9a-f]:[0-9a-f:]*)\\]|[^.[]+|.+\\.(localhost|local|lan|internal|home\\.arpa))$",
        "i",
    )
    .expect("a fixed pattern")
});

/// Models with an id.
pub fn listed(raw: &Value) -> Option<Vec<ListedModel>> {
    Some(
        data_list(raw)?
            .iter()
            .filter_map(|item| {
                let fields = fields_of(item)?;
                let id = text_field(fields, "id").filter(|id| !id.is_empty())?;
                let context = positive_field(fields, "context_length")
                    .or_else(|| positive_field(fields, "max_context_length"))
                    .or_else(|| positive_field(fields, "max_model_len"));
                Some(ListedModel {
                    id: id.to_owned(),
                    context,
                })
            })
            .collect(),
    )
}

/// Capabilities unknown; the test decides.
pub fn model_of(model: &ListedModel) -> LocalModel {
    LocalModel {
        tag: model.id.clone(),
        tools: true,
        thinking: false,
        efforts: Vec::new(),
        context: model.context,
        loaded: None,
        trained: None,
        unprobed: false,
    }
}

struct CompatibleServer {
    transport: Arc<Transport>,
}

impl LocalServer for CompatibleServer {
    fn models<'a>(
        &'a self,
        _known: &'a [LocalModel],
        cancel: &'a CancelToken,
    ) -> BoxFuture<'a, Result<Vec<LocalModel>, KindError>> {
        Box::pin(async move {
            let raw = self
                .transport
                .get_json(LIST, cancel, &KeyAuth::Stored)
                .await?;
            let Some(models) = listed(&raw) else {
                return Err(KindError::Message(
                    "The model list did not parse.".to_owned(),
                ));
            };
            Ok(models.iter().map(model_of).collect())
        })
    }
}

pub fn server(transport: Arc<Transport>) -> Arc<dyn LocalServer> {
    Arc::new(CompatibleServer { transport })
}

/// Off this network: no URL, an unreadable one, or a host that is not private.
pub fn remote(entry: &EntryFields) -> bool {
    let Some(url) = &entry.url else {
        return true;
    };
    match parse_url(url) {
        None => true,
        Some(parts) => {
            let host = parts.host.strip_suffix('.').unwrap_or(&parts.host);
            !PRIVATE_HOST.test(host)
        }
    }
}

/// A remote host may keep data and bill.
pub fn terms_of(entry: &EntryFields) -> Terms {
    if remote(entry) {
        Terms {
            kept: true,
            price: TermsPrice::Unknown,
        }
    } else {
        HOME
    }
}

/// A bare origin means `/v1`; built from the parsed origin and path. A URL that does not read, or
/// carries a query, a fragment or credentials, is refused.
pub fn base_of(url: &str) -> Result<String, String> {
    let plain = parse_url(url).filter(|parts| {
        parts.query.is_empty()
            && parts.fragment.is_empty()
            && parts.username.is_empty()
            && parts.password.is_empty()
    });
    let Some(parts) = plain else {
        return Err("Invalid URL.".to_owned());
    };
    Ok(if parts.path == "/" {
        format!("{}/v1", parts.origin)
    } else {
        format!("{}{}", parts.origin, parts.path)
            .trim_end_matches('/')
            .to_owned()
    })
}

pub static COMPATIBLE: LazyLock<Arc<LocalKind>> = LazyLock::new(|| {
    Arc::new(local_kind(LocalKindSpec {
        kind: ProviderKindName::Compatible,
        label: "OpenAI-compatible",
        url: None,
        optional_key: true,
        replay: None,
        chat_path: Some("/chat/completions"),
        base: Some(base_of),
        terms: Some(terms_of),
        reach: |transport, cancel| {
            Box::pin(async move {
                let raw = transport.get_json(LIST, &cancel, &KeyAuth::Stored).await?;
                Ok(listed(&raw).map(|models| models.len()))
            })
        },
        server,
    }))
});
