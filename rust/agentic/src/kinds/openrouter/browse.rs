//! OpenRouter model browser.

use nyaaai_contract::json::Value;
use nyaaai_contract::shapes::model::{
    BrowseModel, BrowseModelPrice, BrowseModelPriceObject, BrowseModelTools, ProviderRow,
    RowIdentity, RowIdentityHosted,
};
use nyaaai_contract::shapes::provider::ProviderKindName;

use super::catalog::{
    CatalogListing, Endpoint, ParamNeed, Standing, chat_standing, per_million, range_of,
};
use crate::kinds::readers::{
    field_of, fields_of, number_field, object_field, text_field, texts_field,
};
use crate::models::curated::MODELS;
use crate::models::registry::is_model_id;
use crate::models::row_key::{row_key, shown_id};

/// Exclude batch twins.
const BATCH: &str = ":batch";

/// A missing, null or mistyped field is none.
struct BrowseEntry {
    id: String,
    name: Option<String>,
    description: Option<String>,
    context_length: Option<f64>,
    input_modalities: Option<Vec<String>>,
    output_modalities: Option<Vec<String>>,
    supported_parameters: Option<Vec<String>>,
    expiration_date: Option<String>,
    supported_efforts: Option<Vec<String>>,
}

struct ProviderEndpoint {
    provider_name: Option<String>,
    name: Option<String>,
    tag: Option<String>,
    context_length: Option<f64>,
    max_completion_tokens: Option<f64>,
    quantization: Option<String>,
    uptime_last_30m: Option<f64>,
    prices: Option<(Option<Value>, Option<Value>, Option<Value>)>,
}

fn owned(fields: &nyaaai_contract::json::Object, key: &str) -> Option<String> {
    text_field(fields, key).map(str::to_owned)
}

fn entry_of(item: &Value) -> Option<BrowseEntry> {
    let fields = fields_of(item)?;
    let id = text_field(fields, "id")?.to_owned();
    let architecture = object_field(fields, "architecture");
    let reasoning = object_field(fields, "reasoning");
    Some(BrowseEntry {
        id,
        name: owned(fields, "name"),
        description: owned(fields, "description"),
        context_length: number_field(fields, "context_length"),
        input_modalities: architecture.and_then(|held| texts_field(held, "input_modalities")),
        output_modalities: architecture.and_then(|held| texts_field(held, "output_modalities")),
        supported_parameters: texts_field(fields, "supported_parameters"),
        expiration_date: owned(fields, "expiration_date"),
        supported_efforts: reasoning.and_then(|held| texts_field(held, "supported_efforts")),
    })
}

fn endpoint_of(item: &Value) -> Option<ProviderEndpoint> {
    let fields = fields_of(item)?;
    let pricing = object_field(fields, "pricing");
    Some(ProviderEndpoint {
        provider_name: owned(fields, "provider_name"),
        name: owned(fields, "name"),
        tag: owned(fields, "tag"),
        context_length: number_field(fields, "context_length"),
        max_completion_tokens: number_field(fields, "max_completion_tokens"),
        quantization: owned(fields, "quantization"),
        uptime_last_30m: number_field(fields, "uptime_last_30m"),
        prices: pricing.map(|held| {
            (
                field_of(held, "prompt").cloned(),
                field_of(held, "completion").cloned(),
                field_of(held, "input_cache_read").cloned(),
            )
        }),
    })
}

fn chat_params(reasoning: bool) -> Vec<ParamNeed> {
    let mut params = vec![ParamNeed::One("tools".to_owned())];
    if reasoning {
        params.push(ParamNeed::One("reasoning".to_owned()));
    }
    params
}

fn price_ranges(endpoints: &[Endpoint]) -> BrowseModelPrice {
    let of = |pick: fn(&Endpoint) -> Option<&Value>| {
        range_of(
            &endpoints
                .iter()
                .map(|endpoint| per_million(pick(endpoint)))
                .collect::<Vec<_>>(),
        )
    };
    let prompt = of(|endpoint| {
        endpoint
            .pricing
            .as_ref()
            .and_then(|held| held.prompt.as_ref())
    });
    let completion = of(|endpoint| {
        endpoint
            .pricing
            .as_ref()
            .and_then(|held| held.completion.as_ref())
    });
    match (prompt, completion) {
        (Some(prompt), Some(completion)) => {
            BrowseModelPrice::Object(BrowseModelPriceObject { prompt, completion })
        }
        _ => BrowseModelPrice::Unknown,
    }
}

/// The listing as the browser shows it, at a time.
pub fn browse_models(listing: &CatalogListing, now: i64) -> Vec<BrowseModel> {
    let mut models = Vec::new();
    for item in listing.raw() {
        let Some(entry) = entry_of(item) else {
            continue;
        };
        // Host-accepted IDs only.
        if entry.id.ends_with(BATCH) || !is_model_id(&entry.id) {
            continue;
        }
        let id = entry.id.clone();
        let reasoning = entry
            .supported_efforts
            .as_ref()
            .is_some_and(|held| !held.is_empty());
        let zdr: &[Endpoint] = listing
            .endpoints()
            .get(&id)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        let identity = RowIdentity::Hosted(RowIdentityHosted {
            provider: ProviderKindName::Openrouter,
            id: id.clone(),
        });
        let (standing, _) = chat_standing(
            entry.expiration_date.as_deref(),
            zdr,
            &chat_params(reasoning),
            now,
        );
        models.push(BrowseModel {
            id: row_key(&identity),
            display: shown_id(&identity),
            label: entry.name.clone().unwrap_or_else(|| id.clone()),
            curated: MODELS.iter().any(|row| row.key == id),
            description: entry.description.clone().unwrap_or_default(),
            context: entry.context_length,
            inputs: entry.input_modalities.clone().unwrap_or_default(),
            outputs: entry.output_modalities.clone().unwrap_or_default(),
            parameters: entry.supported_parameters.clone().unwrap_or_default(),
            price: price_ranges(zdr),
            tools: if standing == Standing::Ok {
                BrowseModelTools::Ok
            } else {
                BrowseModelTools::NoZdrHost
            },
            retestable: None,
            family: None,
        });
    }
    models
}

/// Provider endpoints, ZDR status.
pub fn parse_providers(raw: &Value, zdr: &[Endpoint]) -> Option<Vec<ProviderRow>> {
    let body = fields_of(raw)?;
    let data = object_field(body, "data")?;
    let endpoints = field_of(data, "endpoints")?.as_array()?;
    let tags: Vec<&str> = zdr
        .iter()
        .filter_map(|endpoint| endpoint.tag.as_deref().filter(|tag| !tag.is_empty()))
        .collect();
    let rows = endpoints
        .iter()
        .filter_map(endpoint_of)
        .map(|provider| {
            let tag = provider.tag.clone().unwrap_or_default();
            let price = |at: usize| {
                provider.prices.as_ref().and_then(|held| {
                    let value = match at {
                        0 => held.0.as_ref(),
                        1 => held.1.as_ref(),
                        _ => held.2.as_ref(),
                    };
                    per_million(value)
                })
            };
            ProviderRow {
                name: provider
                    .provider_name
                    .clone()
                    .or_else(|| provider.name.clone())
                    .unwrap_or_else(|| tag.clone()),
                zdr: tags.contains(&tag.as_str()),
                tag,
                context: provider.context_length,
                max_output: provider.max_completion_tokens,
                quantization: provider.quantization.clone(),
                prompt: price(0),
                completion: price(1),
                cache_read: price(2),
                uptime: provider.uptime_last_30m,
            }
        })
        .collect();
    Some(rows)
}
