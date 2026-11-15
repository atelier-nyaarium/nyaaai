//! Case tree planting.

use std::fs;
use std::io;
use std::path::{Component, Path};

use nyaaai_contract::base64::base64_decode;

use crate::cases::TMP_MARK;
use crate::tree_shapes::TreeEntry;

#[cfg(unix)]
fn link(target: &str, at: &Path) -> io::Result<()> {
    std::os::unix::fs::symlink(target, at)
}

#[cfg(not(unix))]
fn link(_target: &str, _at: &Path) -> io::Result<()> {
    Err(io::Error::other("links need a Unix host"))
}

/// Detects path or symlink escapes.
fn leaves(folder: &Path, entry: &str) -> bool {
    let mut held = folder.to_owned();
    for part in Path::new(entry).components() {
        match part {
            Component::CurDir => continue,
            Component::Normal(name) => held.push(name),
            _ => return true,
        }
        let linked = fs::symlink_metadata(&held).is_ok_and(|found| found.file_type().is_symlink());
        if linked {
            return true;
        }
    }
    false
}

/// Parents first; paths stay contained.
pub fn plant(folder: &Path, entries: &[TreeEntry]) -> io::Result<()> {
    let root = folder.to_string_lossy();
    for entry in entries {
        if leaves(folder, &entry.path) {
            return Err(io::Error::other("a tree entry leaves its folder"));
        }
        let at = folder.join(&entry.path);
        let file = entry.text.is_some() || entry.base64.is_some();
        let parent = if file || entry.link.is_some() {
            at.parent().unwrap_or(folder)
        } else {
            &at
        };
        fs::create_dir_all(parent)?;
        if let Some(target) = &entry.link {
            link(&target.replace(TMP_MARK, &root), &at)?;
        } else if let Some(text) = &entry.base64 {
            let bytes = base64_decode(text).ok_or_else(|| io::Error::other("not base64"))?;
            fs::write(&at, bytes)?;
        } else if let Some(text) = &entry.text {
            fs::write(&at, text)?;
        }
    }
    Ok(())
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    fn entry(path: &str, text: Option<&str>, link: Option<&str>) -> TreeEntry {
        TreeEntry {
            path: path.to_owned(),
            text: text.map(str::to_owned),
            base64: None,
            link: link.map(str::to_owned),
        }
    }

    #[test]
    fn keeps_every_entry_inside_the_tree() {
        let outer = std::env::temp_dir().join(format!("nyaaai-tree-{}", std::process::id()));
        let folder = outer.join("tree");
        let planted = plant(
            &folder,
            &[
                entry("a/f.txt", Some("x"), None),
                entry("out", None, Some("$TMP/..")),
            ],
        );
        let absolute = outer.join("absolute.txt");
        let escapes = [
            "../beside.txt",
            "a/../../beside.txt",
            &absolute.to_string_lossy(),
            "out/through.txt",
            "out",
        ]
        .map(|path| plant(&folder, &[entry(path, Some("x"), None)]).is_err());
        let beside = fs::read_dir(&outer).map(Iterator::count).unwrap_or(0);
        let _ = fs::remove_dir_all(&outer);
        assert!(planted.is_ok());
        assert_eq!(escapes, [true; 5]);
        assert_eq!(beside, 1);
    }
}
