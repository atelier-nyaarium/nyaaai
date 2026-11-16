//! The effort ladder.

use nyaaai_contract::shapes::step::Effort;

/// Ascending thresholds.
pub const EFFORTS: [Effort; 7] = Effort::ALL;

fn rank(effort: &str) -> Option<usize> {
    EFFORTS.iter().position(|held| held.as_str() == effort)
}

/// Requested, else highest below, else lowest; fallbacks skip `max`.
pub fn resolve_effort<'a, T: AsRef<str>>(offered: &'a [T], wanted: &str) -> Option<&'a T> {
    if let Some(found) = offered.iter().find(|effort| effort.as_ref() == wanted) {
        return Some(found);
    }
    let mut ranked: Vec<(usize, &T)> = offered
        .iter()
        .filter(|effort| effort.as_ref() != "max")
        .filter_map(|effort| rank(effort.as_ref()).map(|at| (at, effort)))
        .collect();
    ranked.sort_by_key(|(at, _)| *at);
    let wanted = rank(wanted);
    ranked
        .iter()
        .rev()
        .find(|(at, _)| wanted.is_some_and(|wanted| *at < wanted))
        .or(ranked.first())
        .map(|(_, effort)| *effort)
}
