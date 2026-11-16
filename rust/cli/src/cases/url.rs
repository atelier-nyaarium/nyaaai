//! URL cases.

use nyaaai_contract::url::origin_of;
use nyaaai_testkit::cases::Outcome;

use super::shapes::*;

pub fn origin(input: UrlOriginInput) -> Outcome<UrlOriginResult> {
    Ok(UrlOriginResult {
        origins: input.urls.iter().map(|held| origin_of(held)).collect(),
    })
}
