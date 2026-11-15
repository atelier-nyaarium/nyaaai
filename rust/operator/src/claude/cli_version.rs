//! Installed Claude Code version.

use std::io::{self, Read};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const VERSION_WAIT: Duration = Duration::from_secs(15);
const POLL: Duration = Duration::from_millis(20);

/// First version triplet found.
pub fn version_in(text: &str) -> Option<&str> {
    let digit = |at: usize| text.as_bytes().get(at).is_some_and(u8::is_ascii_digit);
    for from in 0..text.len() {
        if !digit(from) {
            continue;
        }
        let mut at = from;
        let mut groups = 0;
        loop {
            while digit(at) {
                at += 1;
            }
            groups += 1;
            if groups == 3 {
                return Some(&text[from..at]);
            }
            if text.as_bytes().get(at) != Some(&b'.') || !digit(at + 1) {
                break;
            }
            at += 1;
        }
    }
    None
}

/// Parsed version, or full answer.
pub fn installed_version() -> io::Result<String> {
    let mut child = Command::new("claude")
        .arg("--version")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    let started = Instant::now();
    while child.try_wait()?.is_none() {
        if started.elapsed() > VERSION_WAIT {
            child.kill()?;
            child.wait()?;
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "claude --version did not answer",
            ));
        }
        thread::sleep(POLL);
    }
    let mut said = Vec::new();
    if let Some(mut out) = child.stdout.take() {
        out.read_to_end(&mut said)?;
    }
    let said = String::from_utf8_lossy(&said);
    Ok(version_in(&said).unwrap_or(said.trim()).to_owned())
}
