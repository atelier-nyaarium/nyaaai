//! OS path resolution.

use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

/// Resolves missing tails and rejects their `..`.
pub fn resolved_path(target: &Path) -> io::Result<PathBuf> {
    // Follow links before `..`.
    let mut held = std::path::absolute(target)?;
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

/// Tests resolved path containment.
pub fn is_inside(child: &Path, parent: &Path) -> io::Result<bool> {
    Ok(resolved_path(child)?.starts_with(resolved_path(parent)?))
}
