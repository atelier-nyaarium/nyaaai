//! Streamed row chunk tests.

use std::fs;
use std::path::Path;

use nyaaai_operator::claude::session_file::{lines_of, session_lines};

#[test]
fn streams_the_rows_a_held_file_splits_into_across_chunk_edges() {
    // Split inside multibyte characters.
    let row = |at: usize| format!("{at} \u{1F600}\u{e9}\u{2028}{}\r", "x".repeat(at % 97));
    let rows: Vec<String> = (0..40_000).map(row).collect();
    let bad = [b'b', b'a', b'd', b' ', 0xff, 0xe2, 0x82, b'\n'];
    let mut bytes = bad.to_vec();
    bytes.extend(format!("{}\n", rows.join("\n")).into_bytes());
    bytes.extend(bad);
    bytes.extend(b"unterminated tail");
    let file = Path::new(env!("CARGO_TARGET_TMPDIR")).join("session-file-rows.jsonl");
    fs::write(&file, &bytes).unwrap();

    let streamed: Vec<String> = session_lines(&file).unwrap().map(Result::unwrap).collect();
    assert!(bytes.len() > 2 * (1 << 20));
    assert_eq!(streamed, lines_of(&bytes));
    assert_eq!(streamed.len(), 40_002);
    assert_eq!(streamed[8], row(7));
    assert_eq!(streamed[0], "bad \u{fffd}\u{fffd}");
    assert_eq!(streamed[40_001], "bad \u{fffd}\u{fffd}");
    fs::remove_file(&file).unwrap();
}
