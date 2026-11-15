//! Generated from prompts/operator by scripts/contract.ts. Do not edit.

/// prompts/operator/cleared-note.md
pub fn cleared_note(rows: &str, source_path: &str) -> String {
    ["[The owner cleared ", rows, " earlier rows of this conversation without summarizing them. They remain in the transcript at: ", source_path, "]"].concat()
}

/// prompts/operator/compacted-summary.md
pub fn compacted_summary(summary: &str, source_path: &str) -> String {
    ["This session is being continued from an earlier part of this conversation, which the owner compacted above a message they chose. The summary below covers everything before that message.\u{a}\u{a}", summary, "\u{a}\u{a}If you need specific details from before the compaction, read the full transcript at: ", source_path, "\u{a}\u{a}The messages after this summary are the conversation from that message on, kept verbatim."].concat()
}
