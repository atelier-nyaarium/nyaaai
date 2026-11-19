//! What one request cost.

use nyaaai_contract::shapes::model::ModelViewPrice;
use nyaaai_contract::shapes::step::NormalizedUsage;
use nyaaai_contract::shapes::usage::{CallTag, Cost};

/// No-balance sources.
const UNBILLED: [&str; 3] = ["free", "go", "go-plus"];

/// A reported amount: finite and not negative.
pub fn price(value: Option<f64>) -> Option<f64> {
    value.filter(|held| held.is_finite() && *held >= 0.0)
}

/// A count below zero, or no number, is zero.
fn count(value: f64) -> f64 {
    if value >= 0.0 { value } else { 0.0 }
}

/// None when the provider's own charge is missing for a BYOK reply, or the sum passes the largest
/// number.
pub fn reported_cost(usage: &NormalizedUsage) -> Option<f64> {
    let account = price(usage.cost);
    if account.is_none() || usage.byok != Some(true) {
        return account;
    }
    let upstream = price(usage.upstream_cost)?;
    price(Some(account? + upstream))
}

fn unpriced() -> Cost {
    Cost {
        usd: 0.0,
        estimated: false,
        unpriced: Some(()),
    }
}

/// BYOK adds provider charges. Free, or unknown unreported, is unpriced.
pub fn step_cost(
    usage: Option<&NormalizedUsage>,
    view_price: Option<&ModelViewPrice>,
    long_cache: bool,
) -> Cost {
    if matches!(view_price, Some(ModelViewPrice::Free)) {
        return unpriced();
    }
    if let Some(reported) = usage.and_then(reported_cost) {
        return Cost {
            usd: reported,
            estimated: false,
            unpriced: None,
        };
    }
    if matches!(view_price, Some(ModelViewPrice::Unknown)) {
        return unpriced();
    }
    let Some(usage) = usage else {
        return Cost {
            usd: 0.0,
            estimated: true,
            unpriced: None,
        };
    };
    if usage
        .billing_source
        .as_deref()
        .is_some_and(|source| UNBILLED.contains(&source))
    {
        return Cost {
            usd: 0.0,
            estimated: false,
            unpriced: None,
        };
    }
    let fee = if usage.byok == Some(true) {
        price(usage.cost).unwrap_or(0.0)
    } else {
        0.0
    };
    let input = count(usage.input);
    let tiers = match view_price {
        Some(ModelViewPrice::Price(held)) => held.tiers.as_slice(),
        _ => &[],
    };
    let Some(tier) = tiers
        .iter()
        .filter(|candidate| {
            price(Some(candidate.prompt)).is_some() && price(Some(candidate.completion)).is_some()
        })
        .rev()
        .find(|candidate| candidate.min_prompt_tokens <= input)
    else {
        return Cost {
            usd: fee,
            estimated: true,
            unpriced: None,
        };
    };
    let cached = input.min(count(usage.cached.unwrap_or(0.0)));
    let written = (input - cached).min(count(usage.cache_write.unwrap_or(0.0)));
    let write_price = price(if long_cache {
        tier.cache_write1h
    } else {
        tier.cache_write
    })
    .unwrap_or(tier.prompt);
    // The four products are added left to right, then divided once, then the fee.
    let usd = ((input - cached - written) * tier.prompt
        + cached * price(tier.cache_read).unwrap_or(tier.prompt)
        + written * write_price
        + count(usage.output) * tier.completion)
        / 1_000_000.0;
    let total = usd + fee;
    Cost {
        usd: if total.is_finite() { total } else { fee },
        estimated: true,
        unpriced: None,
    }
}

/// Only user chat pays for 1-hour writes; other features keep the default TTL.
pub fn wants_long_cache(tag: &CallTag, cache_write1h: bool) -> bool {
    tag.feature == "chat" && cache_write1h
}
