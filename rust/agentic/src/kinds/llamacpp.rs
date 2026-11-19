//! llama.cpp: the model list and each model's props, read by hand.

use std::sync::{Arc, LazyLock};

use nyaaai_contract::cancel::CancelToken;
use nyaaai_contract::future::BoxFuture;
use nyaaai_contract::json::Value;
use nyaaai_contract::shapes::provider::ProviderKindName;
use nyaaai_contract::text::encode_path_segment;

use super::join::{MAX_LISTED, READS_AT_ONCE, first_distinct, join_limited, ok};
use super::local::catalog::{Loaded, LocalModel, LocalServer, efforts_of_texts};
use super::local::kind::{LocalKind, LocalKindSpec, local_kind};
use super::readers::{data_list, fields_of, object_field, positive_field, text_field, texts_field};
use crate::dialects::local::LocalReplay;
use crate::provider::kind::KindError;
use crate::transport::{KeyAuth, Transport};

/// A model of the list.
#[derive(Debug, Clone, PartialEq)]
pub struct ListedModel {
    pub id: String,
    /// The trained length: the meta's, else the router's context length.
    pub trained: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Props {
    pub context: Option<f64>,
    pub efforts: Vec<String>,
}

const LIST: &str = "/v1/models";

/// Models with an id.
pub fn listed(raw: &Value) -> Option<Vec<ListedModel>> {
    Some(
        data_list(raw)?
            .iter()
            .filter_map(|item| {
                let fields = fields_of(item)?;
                let id = text_field(fields, "id").filter(|id| !id.is_empty())?;
                let meta = object_field(fields, "meta");
                // Router lists, loaded or not.
                let trained = meta
                    .and_then(|meta| positive_field(meta, "n_ctx_train"))
                    .or_else(|| positive_field(fields, "context_length"));
                Some(ListedModel {
                    id: id.to_owned(),
                    trained,
                })
            })
            .collect(),
    )
}

/// What a props body says; none unless it is an object.
pub fn props_of_body(raw: &Value) -> Option<Props> {
    let fields = fields_of(raw)?;
    let settings = object_field(fields, "default_generation_settings");
    let caps = object_field(fields, "chat_template_caps");
    Some(Props {
        context: settings.and_then(|held| positive_field(held, "n_ctx")),
        efforts: caps
            .and_then(|held| texts_field(held, "reasoning_effort_levels"))
            .unwrap_or_default(),
    })
}

/// None while a router's model sleeps.
async fn props_of(
    transport: &Transport,
    id: &str,
    autoload: bool,
    cancel: &CancelToken,
) -> Option<Props> {
    let path = format!(
        "/props?model={}&autoload={autoload}",
        encode_path_segment(id)
    );
    let raw = transport
        .get_json(&path, cancel, &KeyAuth::None)
        .await
        .ok()?;
    props_of_body(&raw)
}

/// Every template may call tools. A sleeping model carries known facts.
pub fn model_of(
    id: &str,
    trained: Option<f64>,
    props: Option<&Props>,
    known: Option<&LocalModel>,
) -> LocalModel {
    let efforts = match props {
        None => known.map(|held| held.efforts.clone()).unwrap_or_default(),
        Some(props) => efforts_of_texts(&props.efforts),
    };
    let context = match props {
        None => known.and_then(|held| held.context),
        Some(props) => props.context,
    };
    LocalModel {
        tag: id.to_owned(),
        tools: true,
        thinking: efforts
            .iter()
            .any(|effort| *effort != nyaaai_contract::shapes::step::Effort::None),
        efforts,
        context,
        loaded: None,
        trained,
        unprobed: props.is_none(),
    }
}

/// One listed model with its props.
async fn one(
    transport: &Transport,
    model: &ListedModel,
    known: &[LocalModel],
    cancel: &CancelToken,
) -> LocalModel {
    let props = props_of(transport, &model.id, false, cancel).await;
    model_of(
        &model.id,
        model.trained,
        props.as_ref(),
        known.iter().find(|each| each.tag == model.id),
    )
}

struct LlamaCppServer {
    transport: Arc<Transport>,
    max_listed: usize,
}

impl LocalServer for LlamaCppServer {
    fn models<'a>(
        &'a self,
        known: &'a [LocalModel],
        cancel: &'a CancelToken,
    ) -> BoxFuture<'a, Result<Vec<LocalModel>, KindError>> {
        Box::pin(async move {
            let raw = self
                .transport
                .get_json(LIST, cancel, &KeyAuth::None)
                .await?;
            let Some(models) = listed(&raw) else {
                return Err(KindError::Message(
                    "The model list did not parse.".to_owned(),
                ));
            };
            let models = first_distinct(models, |model| model.id.clone(), self.max_listed);
            let reads: Vec<_> = models
                .iter()
                .map(|model| ok(one(&self.transport, model, known, cancel)))
                .collect();
            Ok(join_limited(reads, READS_AT_ONCE).await.unwrap_or_default())
        })
    }

    fn reads_loaded(&self) -> bool {
        true
    }

    fn loaded<'a>(
        &'a self,
        tag: &'a str,
        cancel: &'a CancelToken,
    ) -> BoxFuture<'a, Result<Loaded, KindError>> {
        Box::pin(async move {
            let context = props_of(&self.transport, tag, false, cancel)
                .await
                .and_then(|props| props.context);
            Ok(context.map_or_else(Vec::new, |context| vec![(tag.to_owned(), context)]))
        })
    }

    fn warms(&self) -> bool {
        true
    }

    fn warm<'a>(
        &'a self,
        tag: &'a str,
        cancel: &'a CancelToken,
    ) -> BoxFuture<'a, Result<(), KindError>> {
        Box::pin(async move {
            props_of(&self.transport, tag, true, cancel).await;
            Ok(())
        })
    }
}

pub fn server(transport: Arc<Transport>) -> Arc<dyn LocalServer> {
    server_limited(transport, MAX_LISTED)
}

/// A server that reads at most `max_listed` distinct ids of a listing.
pub fn server_limited(transport: Arc<Transport>, max_listed: usize) -> Arc<dyn LocalServer> {
    Arc::new(LlamaCppServer {
        transport,
        max_listed,
    })
}

pub static LLAMACPP: LazyLock<Arc<LocalKind>> = LazyLock::new(|| {
    Arc::new(local_kind(LocalKindSpec {
        kind: ProviderKindName::Llamacpp,
        label: "llama.cpp",
        url: Some("http://127.0.0.1:8080"),
        optional_key: false,
        replay: Some(LocalReplay::ReasoningContent),
        chat_path: None,
        base: None,
        terms: None,
        reach: |transport, cancel| {
            Box::pin(async move {
                let raw = transport.get_json(LIST, &cancel, &KeyAuth::None).await?;
                Ok(listed(&raw).map(|models| models.len()))
            })
        },
        server,
    }))
});
