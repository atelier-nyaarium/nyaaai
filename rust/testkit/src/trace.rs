//! Byte-form trace replay.

use std::collections::VecDeque;
use std::fs;
use std::io;
use std::path::Path;

use nyaaai_contract::base64::base64_decode;
use nyaaai_contract::json::Value;
use nyaaai_contract::shape::Shape;
use nyaaai_contract::text::ascii_lower;

use crate::trace_shapes::{Trace, TraceExchange};

pub fn read_trace(file: &Path) -> io::Result<Trace> {
    let text = fs::read_to_string(file)?;
    Trace::decode_text(&text)
        .map_err(|refused| io::Error::other(format!("{} is no trace: {refused}", file.display())))
}

pub struct Replayed {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    /// Ordered body chunks.
    pub chunks: Vec<Vec<u8>>,
}

/// One exchange per fetch.
pub struct ReplayTransport {
    left: VecDeque<TraceExchange>,
    /// Parsed request bodies.
    pub requests: Vec<Value>,
}

impl ReplayTransport {
    pub fn new(trace: Trace) -> Self {
        Self {
            left: trace.exchanges.into(),
            requests: Vec::new(),
        }
    }

    /// Returns the next exchange.
    pub fn fetch(&mut self, request: Value) -> io::Result<Replayed> {
        let exchange = self
            .left
            .pop_front()
            .ok_or_else(|| io::Error::other("the trace holds no further exchange"))?;
        self.requests.push(request);
        let chunks = exchange
            .chunks
            .iter()
            .map(|chunk| {
                base64_decode(chunk).ok_or_else(|| io::Error::other("a trace chunk is not base64"))
            })
            .collect::<io::Result<Vec<_>>>()?;
        Ok(Replayed {
            status: exchange.status as u16,
            headers: exchange
                .headers
                .iter()
                .map(|(name, value)| (ascii_lower(name), value.clone()))
                .collect(),
            chunks,
        })
    }
}
