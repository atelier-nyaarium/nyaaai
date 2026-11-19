//! Request cost cases.

use nyaaai_agentic::usage::cost::{price as price_of, reported_cost, step_cost, wants_long_cache};
use nyaaai_contract::shapes::model::ModelViewPrice;
use nyaaai_contract::shapes::step::NormalizedUsage;
use nyaaai_testkit::cases::Outcome;

use super::shapes::*;

pub fn step(input: CostStepInput) -> Outcome<CostStepResult> {
    Ok(CostStepResult {
        costs: input
            .rows
            .iter()
            .map(|row| {
                let price = row.price.as_ref().map(|held| match held {
                    CostStepInputRowsItemPrice::Price(price) => {
                        ModelViewPrice::Price(price.clone())
                    }
                    CostStepInputRowsItemPrice::Free => ModelViewPrice::Free,
                    CostStepInputRowsItemPrice::Unknown => ModelViewPrice::Unknown,
                });
                step_cost(row.usage.as_ref(), price.as_ref(), row.long_cache)
            })
            .collect(),
    })
}

pub fn price(input: CostPriceInput) -> Outcome<CostPriceResult> {
    Ok(CostPriceResult {
        prices: input
            .values
            .iter()
            .map(|value| price_of(value.as_f64()))
            .collect(),
    })
}

pub fn reported(input: CostReportedInput) -> Outcome<CostReportedResult> {
    Ok(CostReportedResult {
        values: input
            .rows
            .iter()
            .map(|row| {
                // A sum past the largest number is written as null, as the other side's text does.
                reported_cost(&NormalizedUsage {
                    input: 0.0,
                    output: 0.0,
                    cached: None,
                    cache_write: None,
                    reasoning: None,
                    cost: row.cost,
                    upstream_cost: row.upstream_cost,
                    byok: row.byok,
                    billing_source: None,
                    list_cost: None,
                    allowance: None,
                })
                .filter(|held| held.is_finite())
            })
            .collect(),
    })
}

pub fn long_cache(input: CostLongCacheInput) -> Outcome<CostLongCacheResult> {
    Ok(CostLongCacheResult {
        values: input
            .rows
            .iter()
            .map(|row| {
                wants_long_cache(
                    &row.tag,
                    row.view
                        .as_ref()
                        .is_some_and(|view| view.cache_write1h.is_some()),
                )
            })
            .collect(),
    })
}
