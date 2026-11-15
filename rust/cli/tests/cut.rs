//! Cut command tests.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use nyaaai_contract::json::{Value, parse_json};
use nyaaai_operator::claude::session_cut::verify_cut;
use nyaaai_operator::claude::session_file::session_lines;
use nyaaai_testkit::claude_session::{CLI_VERSION, compacted_session};

fn scratch(name: &str) -> (PathBuf, PathBuf, Vec<u8>) {
    let folder = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = fs::remove_dir_all(&folder);
    fs::create_dir_all(folder.join("out")).unwrap();
    let source = folder.join("source.jsonl");
    let bytes = compacted_session().session.bytes();
    fs::write(&source, &bytes).unwrap();
    (folder, source, bytes)
}

fn cut(source: &Path, out: &Path, more: &[&str]) -> (Option<i32>, Value) {
    let ran = Command::new(env!("CARGO_BIN_EXE_nyaaai"))
        .arg("cut")
        .args(more)
        .args([
            "--source",
            source.to_str().unwrap(),
            "--out",
            out.to_str().unwrap(),
            "--cli-version",
            CLI_VERSION,
        ])
        .output()
        .unwrap();
    let said = String::from_utf8(ran.stdout).unwrap();
    (
        ran.status.code(),
        parse_json(said.trim()).unwrap_or(Value::Null),
    )
}

#[test]
fn writes_a_sound_cut_beside_an_untouched_source() {
    let (folder, source, bytes) = scratch("cut-smoke");
    let out = folder.join("out");
    let third = compacted_session().third;

    for (more, paired, gone, kept) in [
        (vec!["trim"], false, "ancient answer", "second turn"),
        (
            vec!["clear", "--target", &third],
            true,
            "second turn",
            "third turn",
        ),
    ] {
        let (code, said) = cut(&source, &out, &more);
        assert_eq!(code, Some(0), "{more:?}");
        let session_id = said
            .get("sessionId")
            .and_then(Value::as_str)
            .unwrap()
            .to_owned();
        let file = PathBuf::from(said.get("file").and_then(Value::as_str).unwrap());
        assert_eq!(file, out.join(format!("{session_id}.jsonl")));
        assert_eq!(
            verify_cut(session_lines(&file).unwrap(), &session_id, paired).unwrap(),
            Vec::<String>::new()
        );
        let built = fs::read_to_string(&file).unwrap();
        assert!(!built.contains(gone), "{more:?} kept {gone}");
        assert!(built.contains(kept), "{more:?} lost {kept}");
    }
    assert_eq!(fs::read(&source).unwrap(), bytes);
    assert_eq!(fs::read_dir(&folder).unwrap().count(), 2);
}

#[test]
fn refuses_with_its_own_exit_code_and_writes_nothing() {
    let (folder, source, bytes) = scratch("cut-refusal");
    let out = folder.join("out");
    let (code, said) = cut(&source, &out, &["clear", "--target", "nowhere"]);
    assert_eq!(code, Some(2));
    assert_eq!(
        said.get("reason").and_then(Value::as_str),
        Some("target_compacted")
    );
    assert_eq!(fs::read_dir(&out).unwrap().count(), 0);
    assert_eq!(fs::read(&source).unwrap(), bytes);
}

#[test]
fn compacts_with_a_handed_summary_and_fails_on_a_source_it_cannot_read() {
    let (folder, source, _) = scratch("cut-compact");
    let out = folder.join("out");
    let third = compacted_session().third;
    let (code, said) = cut(
        &source,
        &out,
        &["compact", "--target", &third, "--summary", "handed over"],
    );
    assert_eq!(code, Some(0));
    let file = PathBuf::from(said.get("file").and_then(Value::as_str).unwrap());
    assert!(fs::read_to_string(&file).unwrap().contains("handed over"));

    let (code, said) = cut(&folder.join("absent.jsonl"), &out, &["trim"]);
    assert_eq!(code, Some(1));
    assert!(said.is_null());
    assert_eq!(fs::read_dir(&out).unwrap().count(), 1);
}
