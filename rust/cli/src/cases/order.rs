//! Ordering cases.

use nyaaai_contract::order;
use nyaaai_testkit::cases::Outcome;

use super::shapes::*;

pub fn sorted_texts(input: OrderSortedTextsInput) -> Outcome<OrderSortedTextsResult> {
    Ok(OrderSortedTextsResult {
        texts: order::sorted_texts(&input.texts),
    })
}
