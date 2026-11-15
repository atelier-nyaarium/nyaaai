//! Contract version cases.

use nyaaai_contract::shape;
use nyaaai_testkit::cases::Outcome;

use super::shapes::*;

pub fn compatible_versions(
    input: ShapeCompatibleVersionsInput,
) -> Outcome<ShapeCompatibleVersionsResult> {
    let compatible = input
        .pairs
        .iter()
        .map(|pair| shape::compatible_versions(&pair.left, &pair.right))
        .collect();
    Ok(ShapeCompatibleVersionsResult { compatible })
}
