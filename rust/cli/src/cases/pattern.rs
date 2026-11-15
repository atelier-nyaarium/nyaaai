//! Pattern cases.

use nyaaai_contract::pattern::{Pattern, PatternMatch, pattern_problem};
use nyaaai_testkit::cases::Outcome;

use super::shapes::*;

fn pattern_match_shape(matched: PatternMatch) -> PatternCaseMatch {
    PatternCaseMatch {
        start: matched.start as u64,
        end: matched.end as u64,
        text: matched.text,
        captures: matched.captures,
    }
}

pub fn problems(input: PatternProblemsInput) -> Outcome<PatternProblemsResult> {
    Ok(PatternProblemsResult {
        refused: input
            .patterns
            .iter()
            .map(|held| pattern_problem(&held.source, &held.flags).is_some())
            .collect(),
    })
}

pub fn matches(input: PatternMatchesInput) -> Outcome<PatternMatchesResult> {
    let pattern =
        Pattern::compile(&input.source, &input.flags).map_err(|error| error.to_string())?;
    let results = input
        .texts
        .iter()
        .map(|text| PatternMatchesResultResultsItem {
            test: pattern.test(text),
            all: pattern
                .find_all(text)
                .into_iter()
                .map(pattern_match_shape)
                .collect(),
        })
        .collect();
    Ok(PatternMatchesResult { results })
}

pub fn find(input: PatternFindInput) -> Outcome<PatternFindResult> {
    let pattern =
        Pattern::compile(&input.source, &input.flags).map_err(|error| error.to_string())?;
    let found = input
        .froms
        .iter()
        .map(|from| {
            pattern
                .find(&input.text, *from as usize)
                .map(pattern_match_shape)
        })
        .collect();
    Ok(PatternFindResult { found })
}

pub fn edits(input: PatternEditsInput) -> Outcome<PatternEditsResult> {
    let pattern =
        Pattern::compile(&input.source, &input.flags).map_err(|error| error.to_string())?;
    let results = input
        .texts
        .iter()
        .map(|text| PatternEditsResultResultsItem {
            replaced: pattern.replace_all(text, &input.template),
            pieces: pattern.split(text),
        })
        .collect();
    Ok(PatternEditsResult { results })
}
