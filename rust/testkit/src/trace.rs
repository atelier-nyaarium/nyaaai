//! Byte-form trace replay.

use std::fs;
use std::io;
use std::path::Path;
use std::sync::Arc;

use nyaaai_contract::base64::base64_decode;
use nyaaai_contract::shape::Shape;

use crate::scheduler::FakeScheduler;
use crate::scripted_fetch::{ScriptedChunk, ScriptedEnd, ScriptedExchange, ScriptedFetch};
use crate::trace_shapes::Trace;

pub fn read_trace(file: &Path) -> io::Result<Trace> {
    let text = fs::read_to_string(file)?;
    Trace::decode_text(&text)
        .map_err(|refused| io::Error::other(format!("{} is no trace: {refused}", file.display())))
}

/// Plays the exchanges in order on the scheduler's wheel, budgeted, keeping chunk edges. Nothing waits.
pub fn replay_fetch(trace: &Trace, scheduler: Arc<FakeScheduler>) -> io::Result<ScriptedFetch> {
    let exchanges = trace
        .exchanges
        .iter()
        .map(|exchange| {
            let chunks = exchange
                .chunks
                .iter()
                .map(|chunk| {
                    base64_decode(chunk)
                        .map(|bytes| ScriptedChunk { bytes, delay_ms: 0 })
                        .ok_or_else(|| io::Error::other("a trace chunk is not base64"))
                })
                .collect::<io::Result<Vec<_>>>()?;
            Ok(ScriptedExchange {
                status: exchange.status as u16,
                headers: exchange
                    .headers
                    .iter()
                    .map(|(name, value)| (name.to_owned(), value.clone()))
                    .collect(),
                chunks,
                head_ms: 0,
                end: ScriptedEnd::Closed,
                error: None,
            })
        })
        .collect::<io::Result<Vec<_>>>()?;
    Ok(ScriptedFetch::new(exchanges, scheduler))
}
