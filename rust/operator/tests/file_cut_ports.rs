//! File-backed session cut tests.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use nyaaai_contract::id::Ids;
use nyaaai_contract::time::Clock;
use nyaaai_operator::claude::session_cut::{
    CutError, CutMode, CutOutcome, CutPorts, CutRequest, Lines, Summarizer, Summary, cut_session,
    request_for, verify_cut,
};
use nyaaai_operator::claude::session_file::{FileCutPorts, session_lines};
use nyaaai_testkit::cases::block_on;
use nyaaai_testkit::claude_session::{CLI_VERSION, compacted_session};

fn cut<Ports: CutPorts + ?Sized>(
    ports: &Ports,
    request: &CutRequest,
) -> Result<CutOutcome, CutError> {
    block_on(cut_session(ports, request)).unwrap()
}

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
    let done = cut(&ports, &request_for(CutMode::Trim)).unwrap().unwrap();
    let built = ports.built(&done.session_id);
    assert_eq!(built.parent(), Some(ports.out.as_path()));
    let problems = verify_cut(session_lines(&built).unwrap(), &done.session_id, false).unwrap();
    assert_eq!(problems, Vec::<String>::new());
    assert_eq!(fs::read(&source).unwrap(), bytes);
    assert_eq!(files_under(&folder.join("sessions")), 1);
}

/// Summarizes after a yield.
struct Summarizing(FileCutPorts);

impl CutPorts for Summarizing {
    fn read(&self) -> io::Result<Lines<'_>> {
        self.0.read()
    }
    fn source_path(&self) -> String {
        self.0.source_path()
    }
    fn create(&self, session_id: &str, lines: &[String]) -> io::Result<()> {
        self.0.create(session_id, lines)
    }
    fn read_built(&self, session_id: &str) -> io::Result<Lines<'_>> {
        self.0.read_built(session_id)
    }
    fn discard(&self, session_id: &str) -> io::Result<()> {
        self.0.discard(session_id)
    }
    fn ids(&self) -> &dyn Ids {
        self.0.ids()
    }
    fn clock(&self) -> &dyn Clock {
        self.0.clock()
    }
    fn cli_version(&self) -> io::Result<String> {
        self.0.cli_version()
    }
    fn summarizer(&self) -> Option<&dyn Summarizer> {
        Some(self)
    }
}

impl Summarizer for Summarizing {
    fn summarize<'a>(&'a self, _: Option<&'a str>, rows: &'a [String]) -> Summary<'a> {
        Box::pin(async move {
            tokio::task::yield_now().await;
            Ok(format!("{} rows folded", rows.len()))
        })
    }
}

fn is_send<T: Send>(held: T) -> T {
    held
}

#[test]
fn awaits_the_summary_a_port_answers_and_stays_send_over_sync_ports() {
    let (folder, source, _) = scratch("file-ports-summary");
    let ports = Summarizing(FileCutPorts {
        source,
        out: folder.join("out"),
        cli_version: CLI_VERSION.to_owned(),
        keep_out_of: None,
    });
    let mut request = request_for(CutMode::Compact);
    request.target_uuid = Some(compacted_session().second);
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let done = runtime
        .block_on(is_send(cut_session(&ports, &request)))
        .unwrap()
        .unwrap();
    let built = ports.0.built(&done.session_id);
    let text = fs::read_to_string(built).unwrap();
    assert!(text.contains(" rows folded"));
}

/// Read-back always fails.
struct Unreadable(FileCutPorts);

impl CutPorts for Unreadable {
    fn read(&self) -> io::Result<Lines<'_>> {
        self.0.read()
    }
    fn source_path(&self) -> String {
        self.0.source_path()
    }
    fn create(&self, session_id: &str, lines: &[String]) -> io::Result<()> {
        self.0.create(session_id, lines)
    }
    fn read_built(&self, _: &str) -> io::Result<Lines<'_>> {
        Err(io::Error::other("denied"))
    }
    fn discard(&self, session_id: &str) -> io::Result<()> {
        self.0.discard(session_id)
    }
    fn ids(&self) -> &dyn Ids {
        self.0.ids()
    }
    fn clock(&self) -> &dyn Clock {
        self.0.clock()
    }
    fn cli_version(&self) -> io::Result<String> {
        self.0.cli_version()
    }
}

#[test]
fn discards_the_built_file_when_its_read_back_fails() {
    let (folder, source, _) = scratch("file-ports-unreadable");
    let ports = Unreadable(FileCutPorts {
        source,
        out: folder.join("out"),
        cli_version: CLI_VERSION.to_owned(),
        keep_out_of: None,
    });
    assert!(cut(&ports, &request_for(CutMode::Trim)).is_err());
    assert_eq!(files_under(&folder.join("out")), 0);
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
        assert!(cut(&ports, &request_for(CutMode::Trim)).is_err());
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
