//! Path cases.

use std::path::PathBuf;
use std::sync::{Mutex, PoisonError};

use nyaaai_process::paths;
use nyaaai_testkit::cases::{CaseContext, Outcome, failed};
use nyaaai_testkit::tree::plant;

use super::shapes::*;

/// The working directory is shared.
static CWD: Mutex<()> = Mutex::new(());

pub fn is_inside(input: PathsIsInsideInput, context: &CaseContext) -> Outcome<PathsIsInsideResult> {
    let root = context.tmp()?;
    plant(root, &input.tree).map_err(failed)?;
    let relative = input.relative.is_some();
    let place = |text: &str| match text.strip_prefix('/') {
        None if !relative => root.join(text),
        _ => PathBuf::from(text),
    };
    let _held = relative.then(|| CWD.lock().unwrap_or_else(PoisonError::into_inner));
    let started = if relative {
        let started = std::env::current_dir().map_err(failed)?;
        std::env::set_current_dir(root).map_err(failed)?;
        Some(started)
    } else {
        None
    };
    let results = input
        .checks
        .iter()
        .map(
            |check| match paths::is_inside(&place(&check.child), &place(&check.parent)) {
                Ok(inside) => PathsIsInsideResultResultsItem::Flag(inside),
                Err(_) => PathsIsInsideResultResultsItem::Error,
            },
        )
        .collect();
    if let Some(started) = started {
        let _ = std::env::set_current_dir(started);
    }
    Ok(PathsIsInsideResult { results })
}
