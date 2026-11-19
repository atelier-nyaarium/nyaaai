//! Retention cases.

use nyaaai_agentic::kinds::opencode::retention::{is_zdr as zdr, retention_of};
use nyaaai_testkit::cases::Outcome;

use super::shapes::*;

pub fn of(input: RetentionOfInput) -> Outcome<RetentionOfResult> {
    Ok(RetentionOfResult {
        rows: input
            .rows
            .iter()
            .map(|row| {
                retention_of(row.plan, &row.id).map(|held| RetentionOfResultRowsItem {
                    zdr: held.zdr,
                    until: held.until.map(str::to_owned),
                })
            })
            .collect(),
    })
}

pub fn is_zdr(input: RetentionIsZdrInput) -> Outcome<RetentionIsZdrResult> {
    Ok(RetentionIsZdrResult {
        values: input
            .rows
            .iter()
            .map(|row| zdr(row.plan, &row.id, row.now))
            .collect(),
    })
}
