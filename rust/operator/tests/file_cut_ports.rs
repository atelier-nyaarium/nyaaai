//! File-backed session cut tests.

use std::fs;
use std::path::{Path, PathBuf};

use nyaaai_operator::claude::session_cut::{
    CutMode, CutPorts, CutRequest, cut_session, verify_cut,
};
use nyaaai_operator::claude::session_file::{FileCutPorts, session_lines};
use nyaaai_testkit::claude_session::{CLI_VERSION, compacted_session};

fn scratch(name: &str) -> (PathBuf, PathBuf, Vec<u8>) {
    let folder = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = fs::remove_dir_all(&folder);
    fs::create_dir_all(folder.join("sessions")).unwrap();
    let source = folder.join("sessions").join("source.jsonl");
    let bytes = compacted_session().session.bytes();
    fs::write(&source, &bytes).unwrap();
    (folder, source, bytes)
}

fn files_under(folder: &Path) -> usize {
    fs::read_dir(folder).map_or(0, Iterator::count)
}

#[test]
fn builds_a_sound_file_in_a_folder_it_makes_and_leaves_the_source_alone() {
    let (folder, source, bytes) = scratch("file-ports-build");
    let ports = FileCutPorts {
        source: source.clone(),
        out: folder.join("out").join("nested"),
        cli_version: CLI_VERSION.to_owned(),
        keep_out_of: Some(folder.join("sessions")),
    };
    let done = cut_session(&ports, &CutRequest::new(CutMode::Trim))
        .unwrap()
        .unwrap();
    let built = ports.built(&done.session_id);
    assert_eq!(built.parent(), Some(ports.out.as_path()));
    let problems = verify_cut(session_lines(&built).unwrap(), &done.session_id, false).unwrap();
    assert_eq!(problems, Vec::<String>::new());
    assert_eq!(fs::read(&source).unwrap(), bytes);
    assert_eq!(files_under(&folder.join("sessions")), 1);
}

#[test]
fn refuses_to_build_inside_the_folder_it_is_kept_out_of() {
    let (folder, source, bytes) = scratch("file-ports-kept-out");
    for out in [
        folder.join("sessions"),
        folder.join("sessions").join("not").join("made"),
    ] {
        let ports = FileCutPorts {
            source: source.clone(),
            out,
            cli_version: CLI_VERSION.to_owned(),
            keep_out_of: Some(folder.join("sessions")),
        };
        assert!(cut_session(&ports, &CutRequest::new(CutMode::Trim)).is_err());
    }
    assert_eq!(files_under(&folder.join("sessions")), 1);
    assert_eq!(fs::read(&source).unwrap(), bytes);
}

#[test]
fn never_writes_over_a_file_that_is_there() {
    let (folder, source, _) = scratch("file-ports-exclusive");
    let ports = FileCutPorts {
        source,
        out: folder.join("out"),
        cli_version: CLI_VERSION.to_owned(),
        keep_out_of: None,
    };
    for id in ["../outside", "a/b", "..", "", "a\\b"] {
        assert!(ports.create(id, &["{}".to_owned()]).is_err(), "{id:?}");
        assert!(ports.discard(id).is_err(), "{id:?}");
    }
    assert!(!folder.join("outside.jsonl").exists());
    ports.create("taken", &["{}".to_owned()]).unwrap();
    assert!(ports.create("taken", &["other".to_owned()]).is_err());
    assert_eq!(fs::read_to_string(ports.built("taken")).unwrap(), "{}\n");
}
