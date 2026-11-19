//! Generated from the case groups by scripts/contract.ts. Do not edit.

use nyaaai_contract::json::Value;
use nyaaai_contract::shape::Refused;

/// Normalizes a value as a shape of a set. None for an unknown set or shape.
pub fn normalize(set: &str, shape: &str, value: &Value) -> Option<Result<Value, Refused>> {
    match set {
        "frames" => nyaaai_contract::shapes::frames::normalize(shape, value),
        "tree" => nyaaai_testkit::tree_shapes::normalize(shape, value),
        "sessionCut" => nyaaai_contract::shapes::session_cut::normalize(shape, value),
        "provider" => nyaaai_contract::shapes::provider::normalize(shape, value),
        "model" => nyaaai_contract::shapes::model::normalize(shape, value),
        "step" => nyaaai_contract::shapes::step::normalize(shape, value),
        "usage" => nyaaai_contract::shapes::usage::normalize(shape, value),
        "health" => nyaaai_contract::shapes::health::normalize(shape, value),
        "lifecycle" => nyaaai_contract::shapes::lifecycle::normalize(shape, value),
        "torture" => nyaaai_testkit::torture_shapes::normalize(shape, value),
        "trace" => nyaaai_testkit::trace_shapes::normalize(shape, value),
        _ => None,
    }
}
