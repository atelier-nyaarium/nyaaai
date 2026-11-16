//! URL origins.

use crate::text::ascii_lower;

/// Special schemes and their default ports.
fn default_port(scheme: &str) -> Option<u64> {
    match scheme {
        "http" | "ws" => Some(80),
        "https" | "wss" => Some(443),
        "ftp" => Some(21),
        _ => None,
    }
}

const FORBIDDEN_HOST: &str = " #/:<>?@[\\]^|%";

fn is_slash(char: char) -> bool {
    char == '/' || char == '\\'
}

/// Digits of one radix, or `None`; saturates on overflow.
fn digits_value(text: &str, radix: u32) -> Option<u64> {
    if text.is_empty() {
        return None;
    }
    let mut value: u64 = 0;
    for char in text.chars() {
        let digit = char.to_digit(radix)?;
        value = value
            .saturating_mul(u64::from(radix))
            .saturating_add(u64::from(digit));
    }
    Some(value)
}

fn decimal_value(text: &str) -> Option<u64> {
    digits_value(text, 10)
}

fn has_hex_prefix(part: &str) -> bool {
    part.len() >= 2 && (part.starts_with("0x") || part.starts_with("0X"))
}

/// One IPv4 number: `0x` hex, leading-zero octal, or decimal.
fn ipv4_number(part: &str) -> Option<u64> {
    if has_hex_prefix(part) {
        return if part.len() == 2 {
            Some(0)
        } else {
            digits_value(&part[2..], 16)
        };
    }
    if part.len() >= 2 && part.starts_with('0') {
        return digits_value(&part[1..], 8);
    }
    decimal_value(part)
}

/// The labels without one trailing empty label.
fn counted(labels: &[&str]) -> Vec<String> {
    let kept = if labels.len() > 1 && labels[labels.len() - 1].is_empty() {
        &labels[..labels.len() - 1]
    } else {
        labels
    };
    kept.iter().map(|label| (*label).to_owned()).collect()
}

/// Whether the last label reads as a number.
fn ends_in_number(labels: &[&str]) -> bool {
    let parts = counted(labels);
    let last = parts.last().map_or("", String::as_str);
    if last.is_empty() {
        return false;
    }
    decimal_value(last).is_some()
        || (has_hex_prefix(last) && (last.len() == 2 || digits_value(&last[2..], 16).is_some()))
}

/// Dotted decimal, or `None` when the numbers do not fit.
fn ipv4_of(labels: &[&str]) -> Option<String> {
    let parts = counted(labels);
    if parts.len() > 4 {
        return None;
    }
    let mut numbers = Vec::new();
    for part in &parts {
        numbers.push(ipv4_number(part)?);
    }
    let last = *numbers.last()?;
    let count = numbers.len();
    if numbers[..count - 1].iter().any(|value| *value > 255) {
        return None;
    }
    if last >= 256u64.pow(5 - count as u32) {
        return None;
    }
    let mut address = last;
    for (at, value) in numbers[..count - 1].iter().enumerate() {
        address += value * 256u64.pow(3 - at as u32);
    }
    let bytes: Vec<String> = [24, 16, 8, 0]
        .iter()
        .map(|shift| ((address >> shift) % 256).to_string())
        .collect();
    Some(bytes.join("."))
}

/// An embedded IPv4 in an IPv6 literal: four plain decimals.
fn embedded_ipv4(text: &str) -> Option<[u64; 2]> {
    let parts: Vec<&str> = text.split('.').collect();
    if parts.len() != 4 {
        return None;
    }
    let mut bytes = [0u64; 4];
    for (at, part) in parts.iter().enumerate() {
        let value = decimal_value(part)?;
        if value > 255 || (part.len() > 1 && part.starts_with('0')) {
            return None;
        }
        bytes[at] = value;
    }
    Some([bytes[0] * 256 + bytes[1], bytes[2] * 256 + bytes[3]])
}

fn groups(part: &str, ends: bool) -> Option<Vec<u64>> {
    if part.is_empty() {
        return Some(Vec::new());
    }
    let names: Vec<&str> = part.split(':').collect();
    let mut pieces = Vec::new();
    for (at, name) in names.iter().enumerate() {
        if name.contains('.') {
            if !ends || at != names.len() - 1 {
                return None;
            }
            pieces.extend(embedded_ipv4(name)?);
            continue;
        }
        if name.len() > 4 {
            return None;
        }
        pieces.push(digits_value(name, 16)?);
    }
    Some(pieces)
}

/// Eight 16-bit pieces, or `None`.
fn ipv6_pieces(text: &str) -> Option<Vec<u64>> {
    let Some(gap) = text.find("::") else {
        let pieces = groups(text, true)?;
        return (pieces.len() == 8).then_some(pieces);
    };
    if text[gap + 1..].contains("::") {
        return None;
    }
    let head = groups(&text[..gap], false)?;
    let tail = groups(&text[gap + 2..], true)?;
    if head.len() + tail.len() > 7 {
        return None;
    }
    let mut pieces = head.clone();
    pieces.extend(std::iter::repeat_n(0, 8 - head.len() - tail.len()));
    pieces.extend(tail);
    Some(pieces)
}

/// Lowercase hex, the first longest run of zeros compressed.
fn ipv6_text(pieces: &[u64]) -> String {
    let mut from = 0;
    let mut length = 0;
    let mut at = 0;
    while at < 8 {
        if pieces[at] != 0 {
            at += 1;
            continue;
        }
        let mut end = at;
        while end < 8 && pieces[end] == 0 {
            end += 1;
        }
        if end - at > length {
            from = at;
            length = end - at;
        }
        at = end + 1;
    }
    let hex = |slice: &[u64]| {
        slice
            .iter()
            .map(|piece| format!("{piece:x}"))
            .collect::<Vec<_>>()
            .join(":")
    };
    if length < 2 {
        return hex(pieces);
    }
    format!(
        "{}::{}",
        hex(&pieces[..from]),
        hex(&pieces[from + length..])
    )
}

/// The host as URLs write it, or `None`.
fn host_of(raw: &str) -> Option<String> {
    if raw.starts_with('[') {
        let inner = raw.strip_prefix('[')?.strip_suffix(']')?;
        return Some(format!("[{}]", ipv6_text(&ipv6_pieces(inner)?)));
    }
    if raw.is_empty() {
        return None;
    }
    for char in raw.chars() {
        if char <= ' ' || char >= '\u{7f}' || FORBIDDEN_HOST.contains(char) {
            return None;
        }
    }
    let host = ascii_lower(raw);
    let labels: Vec<&str> = host.split('.').collect();
    if ends_in_number(&labels) {
        ipv4_of(&labels)
    } else {
        Some(host)
    }
}

/// The origin of an http, https, ws, wss or ftp URL as the WHATWG parser writes it:
/// lowercase scheme and host, default port dropped, credentials ignored.
/// `None` for any other scheme, a malformed URL, or a host with non-ASCII text.
pub fn origin_of(text: &str) -> Option<String> {
    let url: String = text
        .trim_matches(|char: char| char <= ' ')
        .chars()
        .filter(|char| !matches!(char, '\t' | '\n' | '\r'))
        .collect();
    if !url.chars().next()?.is_ascii_alphabetic() {
        return None;
    }
    let colon = url.find(':')?;
    let scheme = ascii_lower(&url[..colon]);
    if !scheme
        .chars()
        .skip(1)
        .all(|char| char.is_ascii_alphanumeric() || matches!(char, '+' | '-' | '.'))
    {
        return None;
    }
    let default = default_port(&scheme)?;
    let rest = url[colon + 1..].trim_start_matches(is_slash);
    let stop = rest
        .find(|char: char| is_slash(char) || char == '?' || char == '#')
        .unwrap_or(rest.len());
    let authority = &rest[..stop];
    let authority = authority
        .rfind('@')
        .map_or(authority, |at| &authority[at + 1..]);
    let (host_text, port_text) = if authority.starts_with('[') {
        let close = authority.find(']')?;
        let rest = &authority[close + 1..];
        if rest.is_empty() {
            (&authority[..=close], "")
        } else {
            (&authority[..=close], rest.strip_prefix(':')?)
        }
    } else {
        match authority.find(':') {
            Some(at) => (&authority[..at], &authority[at + 1..]),
            None => (authority, ""),
        }
    };
    let host = host_of(host_text)?;
    let mut port = None;
    if !port_text.is_empty() {
        let value = decimal_value(port_text)?;
        if value > 65535 {
            return None;
        }
        port = Some(value);
    }
    Some(match port {
        Some(value) if value != default => format!("{scheme}://{host}:{value}"),
        _ => format!("{scheme}://{host}"),
    })
}
