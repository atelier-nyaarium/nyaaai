//! The curated model list: generated data read through the `ModelRow` shape.

use std::sync::LazyLock;

use nyaaai_contract::json::parse_json;
use nyaaai_contract::shape::{self, Shape};
use nyaaai_contract::shapes::model::ModelRow;

use super::curated_table::CURATED_JSON;

/// Tool+ picker order.
pub static MODELS: LazyLock<Vec<ModelRow>> = LazyLock::new(|| {
    let value = parse_json(CURATED_JSON).expect("the curated table is JSON");
    shape::list(&value, &mut Vec::new(), ModelRow::decode_at).expect("the curated table fits")
});
