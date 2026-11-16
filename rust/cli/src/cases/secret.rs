//! Secret spelling cases.

use nyaaai_contract::secret::{leaks_spelling, scrub_spellings, secret_spellings};
use nyaaai_testkit::cases::Outcome;

use super::shapes::*;

pub fn spellings(input: SecretSpellingsInput) -> Outcome<SecretSpellingsResult> {
    Ok(SecretSpellingsResult {
        spellings: secret_spellings(&input.secret)
            .into_iter()
            .map(|spelling| SecretSpellingsResultSpellingsItem {
                text: spelling.text,
                folded: spelling.folded.into_iter().map(|at| at as i64).collect(),
            })
            .collect(),
    })
}

pub fn scrub(input: SecretScrubInput) -> Outcome<SecretScrubResult> {
    let spellings = secret_spellings(&input.secret);
    Ok(SecretScrubResult {
        scrubbed: scrub_spellings(&input.text, &spellings),
        leaks: leaks_spelling(&input.text, &spellings),
    })
}
