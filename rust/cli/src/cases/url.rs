//! URL cases.

use nyaaai_contract::url::{origin_of, parse_url};
use nyaaai_testkit::cases::Outcome;

use super::shapes::*;

pub fn origin(input: UrlOriginInput) -> Outcome<UrlOriginResult> {
    Ok(UrlOriginResult {
        origins: input.urls.iter().map(|held| origin_of(held)).collect(),
    })
}

pub fn parts(input: UrlPartsInput) -> Outcome<UrlPartsResult> {
    Ok(UrlPartsResult {
        parts: input
            .urls
            .iter()
            .map(|held| {
                parse_url(held).map(|parts| UrlPartsResultPartsItem {
                    scheme: parts.scheme,
                    username: parts.username,
                    password: parts.password,
                    host: parts.host,
                    port: parts.port,
                    origin: parts.origin,
                    path: parts.path,
                    query: parts.query,
                    fragment: parts.fragment,
                })
            })
            .collect(),
    })
}
