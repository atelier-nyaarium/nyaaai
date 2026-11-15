//! Streamed row chunk tests.

use std::fs;
use std::path::Path;

use nyaaai_operator::claude::session_file::{lines_of, session_lines};

#[test]
fn streams_the_rows_a_held_file_splits_into_across_chunk_edges() {
    // Split inside multibyte characters.
    let row = |at: usize| format!("{at} \u{1F600}\u{e9}\u{2028}{}\r", "x".repeat(at % 97));
    let rows: Vec<String> = (0..40_000).map(row).collect();
    let bytes = format!("{}\nunterminated tail", rows.join("\n")).into_bytes();
    let file = Path::new(env!("CARGO_TARGET_TMPDIR")).join("session-file-rows.jsonl");
    fs::write(&file, &bytes).unwrap();

    let streamed: Vec<String> = session_lines(&file).unwrap().map(Result::unwrap).collect();
    assert!(bytes.len() > 2 * (1 << 20));
    assert_eq!(streamed, lines_of(&bytes));
    assert_eq!(streamed.len(), 40_000);
    assert_eq!(streamed[7], row(7));
    fs::remove_file(&file).unwrap();
}
