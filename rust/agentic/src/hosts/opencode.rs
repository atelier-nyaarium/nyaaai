//! OpenCode's failure and usage rules.

use nyaaai_contract::headers::header_of;
use nyaaai_contract::json::{Object, Value};
use nyaaai_contract::shapes::step::{
    CanonicalRequest, Failure, FailureCategory, HeaderList, NormalizedUsage,
};
use nyaaai_contract::text::ascii_lower;
use nyaaai_contract::time::{Clock, reset_stamp_ms};

use crate::chat::codec::{code_of, failure_status, usable};
use crate::chat::wire::{Billing, Usage, read_billing};
use crate::failure::{NumberOrText, WireError};
use crate::native::{Host, HostStatus, number_of};

pub const SESSION_HEADER: &str = "x-opencode-session";

/// Go and workspace limits.
const LIMITS: [&str; 2] = ["GoUsageLimitError", "BudgetLimitError"];

/// Documented `billing_source` values.
const SOURCES: [&str; 5] = ["free", "balance", "go", "go-plus", "byok"];

fn stamp_of(error: &WireError) -> Option<Value> {
    error.resets_at.as_ref().map(|at| match at {
        NumberOrText::Number(number) => Value::Number(*number),
        NumberOrText::Text(text) => Value::from(text.as_str()),
    })
}

fn until_reset(error: &WireError, now_ms: i64) -> Option<i64> {
    let ms = reset_stamp_ms(&stamp_of(error)?, now_ms)?;
    Some((ms - now_ms).max(0))
}

fn amount_of(value: Option<f64>) -> Option<f64> {
    value
        .filter(|amount| *amount >= 0.0)
        .map(|amount| amount + 0.0)
}

fn overloaded(error: &WireError) -> bool {
    error
        .message
        .as_deref()
        .unwrap_or("")
        .contains("[overloaded]")
        || error.type_.as_deref() == Some("overloaded_error")
}

fn category_of(code: Option<f64>, error: &WireError, streamed: bool) -> FailureCategory {
    let message = error.message.as_deref().unwrap_or("");
    // Overload marker takes precedence.
    if (streamed || code == Some(429.0)) && overloaded(error) {
        return FailureCategory::Overloaded;
    }
    if LIMITS.contains(&error.type_.as_deref().unwrap_or("")) {
        return FailureCategory::Allowance;
    }
    let Some(code) = code else {
        return FailureCategory::Stream;
    };
    if code == 400.0 {
        return if ascii_lower(message).contains("model is unavailable") {
            FailureCategory::NotFound
        } else {
            FailureCategory::BadRequest
        };
    }
    if code == 401.0 {
        FailureCategory::KeyRejected
    } else if code == 402.0 {
        FailureCategory::OutOfCredit
    } else if code == 403.0 {
        FailureCategory::Refused
    } else if code == 404.0 {
        FailureCategory::NotFound
    } else if code == 429.0 {
        FailureCategory::RateLimited
    } else if code == 408.0 || code >= 500.0 {
        FailureCategory::Unavailable
    } else {
        FailureCategory::BadRequest
    }
}

/// Rate limits prefer Retry-After.
pub fn opencode_failure(
    clock: &dyn Clock,
    status: HostStatus,
    error: &WireError,
    headers: Option<&HeaderList>,
) -> Failure {
    let category = category_of(code_of(status, error), error, status == HostStatus::Stream);
    let waits = category == FailureCategory::RateLimited
        && headers.is_none_or(|held| header_of(held, "retry-after").is_none());
    let retry_after_ms = if waits {
        until_reset(error, clock.now_ms())
    } else {
        None
    };
    let resets_at = if category == FailureCategory::Allowance {
        stamp_of(error).and_then(|at| reset_stamp_ms(&at, clock.now_ms()))
    } else {
        None
    };
    Failure {
        category,
        status: failure_status(status),
        retry_after_ms,
        resets_at,
        message: usable(error.message.as_deref())
            .unwrap_or("OpenCode reported an error.")
            .to_owned(),
        reasons: None,
        billed: None,
    }
}

/// OpenCode usage fields of loose provider JSON.
pub fn opencode_usage(own: NormalizedUsage, raw: &Object) -> NormalizedUsage {
    let block = raw
        .get("opencode")
        .and_then(Value::as_object)
        .map(read_billing)
        .unwrap_or_default();
    billed(own, raw.get("cost").and_then(number_of), &block)
}

/// OpenCode usage fields of a decoded `Usage`.
pub fn opencode_chat_usage(own: NormalizedUsage, raw: &Usage) -> NormalizedUsage {
    billed(own, raw.cost, &raw.opencode.clone().unwrap_or_default())
}

fn billed(own: NormalizedUsage, cost: Option<f64>, block: &Billing) -> NormalizedUsage {
    let tokens = block.tokens.clone().unwrap_or_default();
    let cost = amount_of(cost);
    let source = block
        .billing_source
        .clone()
        .filter(|source| SOURCES.contains(&source.as_str()));
    let writes: Vec<f64> = [tokens.cache_write_5m, tokens.cache_write_1h]
        .into_iter()
        .filter_map(amount_of)
        .collect();
    let cached = own.cached.or(amount_of(tokens.cache_read));
    let cache_write = own
        .cache_write
        .or_else(|| (!writes.is_empty()).then(|| writes.iter().fold(0.0, |sum, each| sum + each)));
    let reasoning = own.reasoning.or(amount_of(tokens.reasoning));
    NormalizedUsage {
        cached,
        cache_write,
        reasoning,
        cost: cost.or(own.cost),
        billing_source: source.or(own.billing_source.clone()),
        list_cost: amount_of(block.list_cost).or(own.list_cost),
        allowance: amount_of(block.allowance).or(own.allowance),
        ..own
    }
}

/// The plan's routes, for the native families.
pub fn opencode_host<C: Clock>(prefix: &str, clock: C) -> OpenCodeHost<C> {
    OpenCodeHost {
        prefix: prefix.to_owned(),
        clock,
    }
}

pub struct OpenCodeHost<C> {
    prefix: String,
    clock: C,
}

impl<C: Clock + Send + Sync> Host for OpenCodeHost<C> {
    fn prefix(&self) -> &str {
        &self.prefix
    }

    fn headers(&self, request: &CanonicalRequest) -> Vec<(String, String)> {
        vec![(SESSION_HEADER.to_owned(), request.session.clone())]
    }

    fn usage(&self, own: NormalizedUsage, raw: &Object) -> NormalizedUsage {
        opencode_usage(own, raw)
    }

    fn failure(
        &self,
        status: HostStatus,
        error: &WireError,
        headers: Option<&HeaderList>,
    ) -> Failure {
        opencode_failure(&self.clock, status, error, headers)
    }
}
