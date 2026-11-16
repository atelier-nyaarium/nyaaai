//! Notice sentence cases.

use nyaaai_agentic::notice_text;
use nyaaai_testkit::cases::Outcome;

use super::shapes::*;

pub fn texts(input: NoticeTextTextsInput) -> Outcome<NoticeTextTextsResult> {
    let overloaded = input
        .rows
        .iter()
        .map(|row| notice_text::overloaded_text(row.model.as_deref()))
        .collect();
    let unavailable = input
        .rows
        .iter()
        .map(|row| notice_text::unavailable_text(row.model.as_deref(), row.entry.as_deref()))
        .collect();
    Ok(NoticeTextTextsResult {
        overloaded,
        unavailable,
    })
}
