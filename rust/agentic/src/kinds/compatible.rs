//! OpenAI-compatible servers: the model list, the network a host sits on, the request root.

use std::sync::{Arc, LazyLock};

use nyaaai_contract::fetch::Span;
use nyaaai_contract::future::BoxFuture;
use nyaaai_contract::json::Value;
use nyaaai_contract::shapes::provider::ProviderKindName;
use nyaaai_contract::url::parse_url;

use super::join::{MAX_LISTED, first_distinct};
use super::local::catalog::{LocalModel, LocalServer};
use super::local::kind::{LocalKind, LocalKindSpec, local_kind};
pub use super::local::kind::{remote, terms_of};
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
    max_listed: usize,
}

impl LocalServer for CompatibleServer {
    fn models<'a>(
        &'a self,
        _known: &'a [LocalModel],
        span: &'a Span,
    ) -> BoxFuture<'a, Result<Vec<LocalModel>, KindError>> {
        Box::pin(async move {
            let raw = self
                .transport
                .get_json(LIST, span, &KeyAuth::Stored)
                .await?;
            let Some(models) = listed(&raw) else {
                return Err(KindError::Message(
                    "The model list did not parse.".to_owned(),
                ));
            };
            let models = first_distinct(models, |model| model.id.clone(), self.max_listed);
            Ok(models.iter().map(model_of).collect())
        })
    }
}

pub fn server(transport: Arc<Transport>) -> Arc<dyn LocalServer> {
    server_limited(transport, MAX_LISTED)
}

/// A server that keeps at most `max_listed` distinct ids of a listing.
pub fn server_limited(transport: Arc<Transport>, max_listed: usize) -> Arc<dyn LocalServer> {
    Arc::new(CompatibleServer {
        transport,
        max_listed,
    })
}

/// A bare origin means `/v1`; a query, fragment or credentials refuse.
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
        terms: None,
        reach: |transport, span| {
            Box::pin(async move {
                let raw = transport.get_json(LIST, span, &KeyAuth::Stored).await?;
                Ok(listed(&raw).map(|models| models.len()))
            })
        },
        server,
    }))
});
