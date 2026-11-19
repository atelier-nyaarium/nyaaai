//! The spellings a secret can take in text.

use crate::base64::base64_encode;
use crate::text::encode_path_segment;

/// What replaces a secret.
pub const REDACTED: &str = "[key]";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spelling {
    pub text: String,
    /// Positions matched without ASCII case.
    pub folded: Vec<usize>,
}

/// Hex digits of each `%XX` triple.
fn hex_positions(text: &str) -> Vec<usize> {
    let mut found = Vec::new();
    for (at, char) in text.chars().enumerate() {
        if char == '%' {
            found.push(at + 1);
            found.push(at + 2);
        }
    }
    found
}

/// Unpadded base64 after `lead` zero bytes: the full run, and the characters holding only secret bits.
fn shifted(secret: &[u8], lead: usize) -> (String, String) {
    let mut padded = vec![0u8; lead];
    padded.extend_from_slice(secret);
    let encoded = base64_encode(&padded);
    let text = encoded.trim_end_matches('=');
    let from = (lead * 8).div_ceil(6);
    let to = (lead + secret.len()) * 8 / 6;
    let full = text.get(from..).unwrap_or("");
    let core = text.get(from..to.max(from)).unwrap_or("");
    (full.to_owned(), core.to_owned())
}

/// Plain, path-encoded in either hex case, and base64 at each byte alignment.
pub fn secret_spellings(secret: &str) -> Vec<Spelling> {
    if secret.is_empty() {
        return Vec::new();
    }
    let encoded = encode_path_segment(secret);
    let mut found = vec![
        Spelling {
            text: secret.to_owned(),
            folded: Vec::new(),
        },
        Spelling {
            folded: hex_positions(&encoded),
            text: encoded,
        },
    ];
    for lead in 0..3 {
        let (full, core) = shifted(secret.as_bytes(), lead);
        for text in [full, core] {
            found.push(Spelling {
                text,
                folded: Vec::new(),
            });
        }
    }
    let mut kept: Vec<Spelling> = Vec::new();
    for spelling in found {
        if !spelling.text.is_empty() && !kept.contains(&spelling) {
            kept.push(spelling);
        }
    }
    kept
}

/// A spelling by characters, to compare at a position.
struct Prepared {
    chars: Vec<char>,
    folded: Vec<usize>,
}

fn prepared(spellings: &[Spelling]) -> Vec<Prepared> {
    spellings
        .iter()
        .map(|spelling| Prepared {
            chars: spelling.text.chars().collect(),
            folded: spelling.folded.clone(),
        })
        .collect()
}

fn same(held: char, wanted: char) -> bool {
    held == wanted || (held.is_ascii() && wanted.is_ascii() && held.eq_ignore_ascii_case(&wanted))
}

fn matches(text: &[char], at: usize, spelling: &Prepared) -> bool {
    if at + spelling.chars.len() > text.len() {
        return false;
    }
    spelling.chars.iter().enumerate().all(|(place, wanted)| {
        let held = text[at + place];
        held == *wanted || (spelling.folded.contains(&place) && same(held, *wanted))
    })
}

/// Length of the longest spelling at `at`; zero when none.
fn longest_at(text: &[char], at: usize, spellings: &[Prepared]) -> usize {
    spellings
        .iter()
        .filter(|spelling| matches(text, at, spelling))
        .map(|spelling| spelling.chars.len())
        .max()
        .unwrap_or(0)
}

/// Whether any spelling occurs in `text`.
pub fn leaks_spelling(text: &str, spellings: &[Spelling]) -> bool {
    let chars: Vec<char> = text.chars().collect();
    let held = prepared(spellings);
    (0..chars.len()).any(|at| longest_at(&chars, at, &held) > 0)
}

/// One pass: longest matches become `[key]`; replacements are never rescanned.
pub fn scrub_spellings(text: &str, spellings: &[Spelling]) -> String {
    let chars: Vec<char> = text.chars().collect();
    let held = prepared(spellings);
    let mut out = String::new();
    let mut at = 0;
    while at < chars.len() {
        let length = longest_at(&chars, at, &held);
        if length == 0 {
            out.push(chars[at]);
            at += 1;
        } else {
            out.push_str(REDACTED);
            at += length;
        }
    }
    out
}

/// `text` with every spelling of `secret` replaced.
pub fn scrub_secret(text: &str, secret: &str) -> String {
    scrub_spellings(text, &secret_spellings(secret))
}
