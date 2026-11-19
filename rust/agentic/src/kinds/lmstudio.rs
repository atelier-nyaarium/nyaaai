//! LM Studio: the model list, read by hand.

use std::sync::{Arc, LazyLock};

use nyaaai_contract::cancel::CancelToken;
use nyaaai_contract::future::BoxFuture;
use nyaaai_contract::json::{Object, Value};
use nyaaai_contract::shapes::provider::ProviderKindName;

use super::local::catalog::{
    Loaded, LocalModel, LocalServer, Think, efforts_of, items_of, put_loaded,
};
use super::local::kind::{LocalKind, LocalKindSpec, local_kind};
use super::readers::{field_of, fields_of, number_field, object_field, text_field, texts_field};
use crate::provider::kind::KindError;
use crate::transport::{KeyAuth, Transport};

/// A chat model of the list.
#[derive(Debug, Clone, PartialEq)]
pub struct ListedModel {
    pub key: String,
    /// The widest context the model trained for.
    pub trained: Option<f64>,
    /// The first loaded instance's context.
    pub loaded: Option<f64>,
    /// The reasoning options, when listed.
    pub options: Option<Vec<String>>,
}

const LIST: &str = "/api/v1/models";

/// The first instance's context; none when any instance does not read.
fn loaded_of(fields: &Object) -> Option<f64> {
    let list = field_of(fields, "loaded_instances")?.as_array()?;
    let configs: Vec<Option<&Object>> = list
        .iter()
        .map(|instance| fields_of(instance).and_then(|held| object_field(held, "config")))
        .collect();
    if configs.contains(&None) {
        return None;
    }
    number_field((*configs.first()?)?, "context_length")
}

fn listed_of(item: &Value) -> Option<ListedModel> {
    let fields = fields_of(item)?;
    let key = text_field(fields, "key").filter(|key| !key.is_empty())?;
    if text_field(fields, "type") != Some("llm") {
        return None;
    }
    let reasoning =
        object_field(fields, "capabilities").and_then(|held| object_field(held, "reasoning"));
    Some(ListedModel {
        key: key.to_owned(),
        trained: number_field(fields, "max_context_length"),
        loaded: loaded_of(fields),
        options: reasoning.and_then(|held| texts_field(held, "allowed_options")),
    })
}

pub fn chat_models(raw: &Value) -> Option<Vec<ListedModel>> {
    Some(
        items_of(raw, "models")?
            .iter()
            .filter_map(listed_of)
            .collect(),
    )
}

/// Every model may call tools.
pub fn model_of(model: &ListedModel) -> LocalModel {
    let options = model.options.clone().unwrap_or_default();
    let thinks = options.iter().any(|option| option != "off");
    let values: Vec<Think> = options
        .iter()
        .map(|option| match option.as_str() {
            "off" => Think::Flag(false),
            "on" => Think::Flag(true),
            other => Think::Text(other.to_owned()),
        })
        .collect();
    LocalModel {
        tag: model.key.clone(),
        tools: true,
        thinking: thinks,
        efforts: if thinks {
            efforts_of(&values)
        } else {
            Vec::new()
        },
        context: None,
        loaded: model.loaded,
        trained: model.trained,
        unprobed: false,
    }
}

/// Loaded context by key.
pub fn loaded_contexts(models: &[ListedModel]) -> Loaded {
    let mut contexts = Loaded::new();
    for model in models {
        if let Some(loaded) = model.loaded {
            put_loaded(&mut contexts, &model.key, loaded);
        }
    }
    contexts
}

struct LmStudioServer {
    transport: Arc<Transport>,
}

impl LocalServer for LmStudioServer {
    fn models<'a>(
        &'a self,
        _known: &'a [LocalModel],
        cancel: &'a CancelToken,
    ) -> BoxFuture<'a, Result<Vec<LocalModel>, KindError>> {
        Box::pin(async move {
            let raw = self
                .transport
                .get_json(LIST, cancel, &KeyAuth::None)
                .await?;
            let Some(models) = chat_models(&raw) else {
                return Err(KindError::Message(
                    "The model list did not parse.".to_owned(),
                ));
            };
            Ok(models.iter().map(model_of).collect())
        })
    }

    fn live(&self) -> bool {
        true
    }

    fn reads_loaded(&self) -> bool {
        true
    }

    fn loaded<'a>(
        &'a self,
        _tag: &'a str,
        cancel: &'a CancelToken,
    ) -> BoxFuture<'a, Result<Loaded, KindError>> {
        Box::pin(async move {
            let raw = self
                .transport
                .get_json(LIST, cancel, &KeyAuth::None)
                .await?;
            Ok(loaded_contexts(&chat_models(&raw).unwrap_or_default()))
        })
    }
}

pub fn server(transport: Arc<Transport>) -> Arc<dyn LocalServer> {
    Arc::new(LmStudioServer { transport })
}

pub static LMSTUDIO: LazyLock<Arc<LocalKind>> = LazyLock::new(|| {
    Arc::new(local_kind(LocalKindSpec {
        kind: ProviderKindName::Lmstudio,
        label: "LM Studio",
        url: Some("http://127.0.0.1:1234"),
        optional_key: false,
        replay: None,
        chat_path: None,
        base: None,
        terms: None,
        reach: |transport, cancel| {
            Box::pin(async move {
                let raw = transport.get_json(LIST, &cancel, &KeyAuth::None).await?;
                Ok(chat_models(&raw).map(|models| models.len()))
            })
        },
        server,
    }))
});
