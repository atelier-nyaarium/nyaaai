//! Claude Code session rows.

use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use nyaaai_contract::id::random_uuid;

use super::session_cut::{CutPorts, Lines};

const READ_CHUNK_BYTES: usize = 1 << 20;

/// Replaces invalid bytes; preserves BOM.
fn decoded(row: &[u8]) -> String {
    String::from_utf8_lossy(row).into_owned()
}

/// Splits LF rows; drops incomplete tails.
pub fn lines_of(bytes: &[u8]) -> Vec<String> {
    let mut rows = Vec::new();
    let mut from = 0;
    while let Some(at) = bytes[from..].iter().position(|byte| *byte == b'\n') {
        rows.push(decoded(&bytes[from..from + at]));
        from += at + 1;
    }
    rows
}

/// Streams LF-terminated rows from a read-only file.
pub fn session_lines(file: &Path) -> io::Result<impl Iterator<Item = io::Result<String>> + use<>> {
    let mut reader = BufReader::with_capacity(READ_CHUNK_BYTES, File::open(file)?);
    Ok(std::iter::from_fn(move || {
        let mut row = Vec::new();
        match reader.read_until(b'\n', &mut row) {
            Ok(_) if row.last() == Some(&b'\n') => {
                row.pop();
                Some(Ok(decoded(&row)))
            }
            Ok(_) => None,
            Err(error) => Some(Err(error)),
        }
    }))
}

/// Canonicalizes paths with missing tails.
fn resolved(path: &Path) -> io::Result<PathBuf> {
    let mut held = std::path::absolute(path)?;
    let mut tail: Vec<OsString> = Vec::new();
    loop {
        match fs::canonicalize(&held) {
            Ok(mut found) => {
                found.extend(tail.iter().rev());
                return Ok(found);
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                match held.components().next_back() {
                    Some(Component::Normal(last)) => tail.push(last.to_owned()),
                    _ => return Err(error),
                }
                held.pop();
            }
            Err(error) => return Err(error),
        }
    }
}

/// File-backed session cut ports.
#[derive(Debug, Clone)]
pub struct FileCutPorts {
    pub source: PathBuf,
    pub out: PathBuf,
    pub cli_version: String,
    /// Forbidden output folder.
    pub keep_out_of: Option<PathBuf>,
}

impl FileCutPorts {
    pub fn built(&self, session_id: &str) -> PathBuf {
        self.out.join(format!("{session_id}.jsonl"))
    }

    /// The built file, for an id that is a file name.
    fn target(&self, session_id: &str) -> io::Result<PathBuf> {
        let named = !session_id.is_empty()
            && !session_id.contains(['/', '\\'])
            && session_id != "."
            && session_id != "..";
        if !named {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "the session id is not a file name",
            ));
        }
        Ok(self.built(session_id))
    }
}

impl CutPorts for FileCutPorts {
    fn read(&self) -> io::Result<Lines<'_>> {
        Ok(Box::new(session_lines(&self.source)?))
    }

    fn source_path(&self) -> String {
        self.source.to_string_lossy().into_owned()
    }

    fn create(&self, session_id: &str, lines: &[String]) -> io::Result<()> {
        if let Some(kept) = &self.keep_out_of
            && resolved(&self.out)?.starts_with(resolved(kept)?)
        {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                format!("{} is inside {}", self.out.display(), kept.display()),
            ));
        }
        fs::create_dir_all(&self.out)?;
        let target = self.target(session_id)?;
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&target)?;
        let mut out = BufWriter::new(file);
        let written = lines
            .iter()
            .try_for_each(|line| {
                out.write_all(line.as_bytes())?;
                out.write_all(b"\n")
            })
            .and_then(|()| out.flush());
        if written.is_err() {
            // Remove partial output.
            let _ = fs::remove_file(&target);
        }
        written
    }

    fn read_built(&self, session_id: &str) -> io::Result<Lines<'_>> {
        Ok(Box::new(session_lines(&self.target(session_id)?)?))
    }

    fn discard(&self, session_id: &str) -> io::Result<()> {
        fs::remove_file(self.target(session_id)?)
    }

    fn new_id(&self) -> String {
        random_uuid()
    }

    fn now_ms(&self) -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |since| since.as_millis() as i64)
    }

    fn cli_version(&self) -> io::Result<String> {
        Ok(self.cli_version.clone())
    }
}
