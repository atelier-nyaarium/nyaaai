//! Header lists.

use crate::text::ascii_lower;

/// The first value whose name matches, ASCII case aside.
pub fn header_of<'a>(headers: &'a [(String, String)], name: &str) -> Option<&'a str> {
    let wanted = ascii_lower(name);
    headers
        .iter()
        .find(|(held, _)| ascii_lower(held) == wanted)
        .map(|(_, value)| value.as_str())
}
