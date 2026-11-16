//! Header lists.

use crate::order::compare_text;
use crate::shapes::step::HeaderList;
use crate::text::ascii_lower;

/// The first value whose name matches, ASCII case aside.
pub fn header_of<'a>(headers: &'a [(String, String)], name: &str) -> Option<&'a str> {
    let wanted = ascii_lower(name);
    headers
        .iter()
        .find(|(held, _)| ascii_lower(held) == wanted)
        .map(|(_, value)| value.as_str())
}

/// Lowercase names, sorted; repeats joined with `, `, `set-cookie` kept apart.
pub fn response_headers(raw: &[(String, String)]) -> HeaderList {
    let mut named: Vec<(String, Vec<&str>)> = Vec::new();
    for (name, value) in raw {
        let key = ascii_lower(name);
        match named.iter_mut().find(|(held, _)| *held == key) {
            Some((_, values)) => values.push(value),
            None => named.push((key, vec![value])),
        }
    }
    named.sort_by(|(left, _), (right, _)| compare_text(left, right));
    let mut out = HeaderList::new();
    for (key, values) in named {
        if key == "set-cookie" {
            out.extend(
                values
                    .iter()
                    .map(|value| (key.clone(), (*value).to_owned())),
            );
        } else {
            out.push((key, values.join(", ")));
        }
    }
    out
}
