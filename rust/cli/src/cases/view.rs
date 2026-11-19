//! Model view cases.

use nyaaai_agentic::models::curated::MODELS;
use nyaaai_agentic::models::view::{
    codec_facts_of, run_effort as effort_of, tiers_of, unknown_view,
};
use nyaaai_contract::json::{Value, parse_json};
use nyaaai_contract::shape::Shape;
use nyaaai_contract::shapes::model::{ModelView, ModelViewPrice, ModelViewStatus};
use nyaaai_contract::shapes::step::{ApiFacts, Effort};
use nyaaai_testkit::cases::Outcome;

use super::shapes::*;

/// One generated shape as another with the same wire form.
fn convert<From: Shape, To: Shape>(held: &From) -> Outcome<To> {
    To::decode(&held.encode()).map_err(|refused| refused.to_string())
}

/// What the helpers read of a view; its row is made up.
struct Spec {
    reasoning: bool,
    status: ModelViewStatus,
    efforts: Vec<Effort>,
    output_limit: Option<f64>,
    price: Option<ModelViewPrice>,
    api: Option<ApiFacts>,
}

fn view_of(spec: Spec) -> ModelView {
    let mut row = MODELS[0].clone();
    row.key = "k".to_owned();
    row.reasoning = spec.reasoning;
    let mut view = unknown_view(&row);
    view.status = spec.status;
    view.efforts = spec.efforts;
    view.output_limit = spec.output_limit;
    view.price = spec.price;
    view.api = spec.api;
    view
}

pub fn unknown(input: ViewUnknownInput) -> Outcome<ViewUnknownResult> {
    let views = input
        .keys
        .iter()
        .map(|key| {
            MODELS
                .iter()
                .find(|row| row.key == *key)
                .map(unknown_view)
                .ok_or_else(|| format!("No row {key}."))
        })
        .collect::<Outcome<Vec<_>>>()?;
    Ok(ViewUnknownResult { views })
}

pub fn tiers(input: ViewTiersInput) -> Outcome<ViewTiersResult> {
    let tiers = input
        .rows
        .iter()
        .map(|row| {
            let view = match &row.view {
                None => None,
                Some(held) => {
                    let mut view = unknown_view(&MODELS[0]);
                    view.price = match &held.price {
                        None => None,
                        Some(price) => Some(convert::<_, ModelViewPrice>(price)?),
                    };
                    Some(view)
                }
            };
            Ok(tiers_of(view.as_ref()).map(<[_]>::to_vec))
        })
        .collect::<Outcome<Vec<_>>>()?;
    Ok(ViewTiersResult { tiers })
}

pub fn codec_facts(input: ViewCodecFactsInput) -> Outcome<ViewCodecFactsResult> {
    let facts = input
        .views
        .into_iter()
        .map(|spec| {
            let view = match spec {
                None => None,
                Some(held) => Some(view_of(Spec {
                    reasoning: held.reasoning,
                    status: convert(&held.status)?,
                    efforts: held.efforts,
                    output_limit: held.output_limit,
                    price: match &held.price {
                        None => None,
                        Some(price) => Some(convert(price)?),
                    },
                    api: held.api,
                })),
            };
            Ok(codec_facts_of(view.as_ref()))
        })
        .collect::<Outcome<Vec<_>>>()?;
    Ok(ViewCodecFactsResult { facts })
}

pub fn output_limit(input: ViewOutputLimitInput) -> Outcome<ViewOutputLimitResult> {
    let limits = input
        .texts
        .iter()
        .map(|text| match parse_json(text) {
            Some(Value::Number(limit)) => {
                let view = view_of(Spec {
                    reasoning: false,
                    status: ModelViewStatus::Ok,
                    efforts: Vec::new(),
                    output_limit: Some(limit),
                    price: None,
                    api: None,
                });
                Ok(codec_facts_of(Some(&view)).and_then(|facts| facts.output_limit))
            }
            _ => Err(format!("{text} is not a number.")),
        })
        .collect::<Outcome<Vec<_>>>()?;
    Ok(ViewOutputLimitResult { limits })
}

pub fn run_effort(input: ViewRunEffortInput) -> Outcome<ViewRunEffortResult> {
    let efforts = input
        .rows
        .into_iter()
        .map(|row| {
            let view = match row.view {
                None => None,
                Some(held) => Some(view_of(Spec {
                    reasoning: held.reasoning,
                    status: convert(&held.status)?,
                    efforts: held.efforts,
                    output_limit: held.output_limit,
                    price: match &held.price {
                        None => None,
                        Some(price) => Some(convert(price)?),
                    },
                    api: held.api,
                })),
            };
            Ok(effort_of(view.as_ref(), row.wanted))
        })
        .collect::<Outcome<Vec<_>>>()?;
    Ok(ViewRunEffortResult { efforts })
}
