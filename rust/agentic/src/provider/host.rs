//! Hosts of this network, judged by their text, and where a key may travel.

use std::sync::LazyLock;

use nyaaai_contract::pattern::Pattern;
use nyaaai_contract::url::parse_url;

/// Loopback, private ranges and LAN names.
static PRIVATE_HOST: LazyLock<Pattern> = LazyLock::new(|| {
    Pattern::compile(
        "^(localhost|0\\.0\\.0\\.0|(127|10)\\.[\\d.]+|192\\.168\\.[\\d.]+|172\\.(1[6-9]|2\\d|3[01])\\.[\\d.]+|169\\.254\\.[\\d.]+|100\\.(6[4-9]|[7-9]\\d|1[01]\\d|12[0-7])\\.[\\d.]+|\\[(::1?|f[cd][0-9a-f]{2}:[0-9a-f:]*|fe[89ab][0-9a-f]:[0-9a-f:]*)\\]|[^.[]+|.+\\.(localhost|local|lan|internal|home\\.arpa))$",
        "i",
    )
    .expect("a fixed pattern")
});

const MAPPED_PREFIX: &str = "[::ffff:";

/// One to four lowercase hex digits.
fn piece(text: &str) -> Option<u32> {
    if text.is_empty() || text.len() > 4 {
        return None;
    }
    let mut value = 0;
    for char in text.chars() {
        let digit = "0123456789abcdef".find(char)?;
        value = value * 16 + digit as u32;
    }
    Some(value)
}

/// The IPv4 address of an IPv4-mapped IPv6 literal.
fn mapped_ipv4(host: &str) -> Option<String> {
    let inner = host.strip_prefix(MAPPED_PREFIX)?.strip_suffix(']')?;
    let mut pieces = inner.split(':');
    let high = piece(pieces.next()?)?;
    let low = piece(pieces.next()?)?;
    if pieces.next().is_some() {
        return None;
    }
    Some(format!(
        "{}.{}.{}.{}",
        high >> 8,
        high & 255,
        low >> 8,
        low & 255
    ))
}

/// A host of this network, as `parse_url` writes it; never resolved, a trailing dot cut.
pub fn is_local_host(host: &str) -> bool {
    let cut = host.strip_suffix('.').unwrap_or(host);
    match mapped_ipv4(cut) {
        Some(ipv4) => PRIVATE_HOST.test(&ipv4),
        None => PRIVATE_HOST.test(cut),
    }
}

/// https, or http to a host of this network.
pub fn key_may_travel(url: &str) -> bool {
    parse_url(url).is_some_and(|parts| {
        parts.scheme == "https" || (parts.scheme == "http" && is_local_host(&parts.host))
    })
}
