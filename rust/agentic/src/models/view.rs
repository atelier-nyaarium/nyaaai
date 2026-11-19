//! What a catalog says of one model, and what a run reads of it.

use nyaaai_contract::shapes::model::{
    ModelRow, ModelView, ModelViewPrice, ModelViewStatus, PriceTier,
};
use nyaaai_contract::shapes::step::{ApiFacts, ApiFamily, CodecFacts, Effort};

use crate::effort::resolve_effort;

/// A view without an API reads as one with no capabilities.
fn no_api() -> ApiFacts {
    ApiFacts {
        family: ApiFamily::ChatCompletions,
        structured: false,
        choice: Vec::new(),
        strict: false,
        reasoning: None,
        toggle: None,
    }
}

pub fn unknown_view(row: &ModelRow) -> ModelView {
    ModelView {
        row: row.clone(),
        status: ModelViewStatus::Unknown,
        retired_on: None,
        efforts: Vec::new(),
        prompt_limit: None,
        output_limit: None,
        price: None,
        cache_write1h: None,
        hosts: Vec::new(),
        data_kept: None,
        api: None,
        cache: None,
    }
}

/// Listed tiers; none when unpriced.
pub fn tiers_of(view: Option<&ModelView>) -> Option<&[PriceTier]> {
    match view?.price.as_ref()? {
        ModelViewPrice::Price(price) => Some(&price.tiers),
        _ => None,
    }
}

/// A whole limit, capped at the largest safe integer; none if not finite or below one.
fn usable_limit(limit: f64) -> Option<i64> {
    (limit.is_finite() && limit >= 1.0).then(|| limit.floor().min(9_007_199_254_740_991.0) as i64)
}

/// What a Chat codec reads of a view.
pub fn codec_facts_of(view: Option<&ModelView>) -> Option<CodecFacts> {
    let view = view?;
    Some(CodecFacts {
        api: view.api.clone().unwrap_or_else(no_api),
        efforts: view.efforts.clone(),
        output_limit: view.output_limit.and_then(usable_limit),
        free: matches!(view.price, Some(ModelViewPrice::Free)).then_some(true),
    })
}

/// None for missing inputs; unknown rows pass the wish only if reasoning; others resolve offers.
pub fn run_effort(view: Option<&ModelView>, wanted: Option<Effort>) -> Option<Effort> {
    let (view, wanted) = (view?, wanted?);
    if view.status == ModelViewStatus::Unknown {
        return view.row.reasoning.then_some(wanted);
    }
    let offered: Vec<&str> = view.efforts.iter().map(|effort| effort.as_str()).collect();
    resolve_effort(&offered, wanted.as_str()).and_then(|held| Effort::parse(held))
}
