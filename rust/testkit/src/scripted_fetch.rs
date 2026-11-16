//! A Fetch that plays a script on virtual time.

use std::sync::{Arc, Mutex, MutexGuard};

use nyaaai_contract::cancel::CancelToken;
use nyaaai_contract::fetch::{Chunks, Fetch, FetchError, FetchRequest, FetchResponse};
use nyaaai_contract::future::BoxFuture;
use nyaaai_contract::headers::response_headers;
use nyaaai_contract::shapes::step::HeaderList;

use crate::scheduler::TimerWheel;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptedChunk {
    pub bytes: Vec<u8>,
    /// Wheel time spent first.
    pub delay_ms: i64,
}

/// How an exchange ends.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ScriptedEnd {
    #[default]
    Closed,
    /// The last read fails.
    Dropped,
    /// No head arrives.
    Refused,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptedExchange {
    pub status: u16,
    /// Raw pairs; answered in `response_headers` form.
    pub headers: HeaderList,
    pub chunks: Vec<ScriptedChunk>,
    /// Wheel time spent first.
    pub head_ms: i64,
    pub end: ScriptedEnd,
    /// Message of a drop or refusal.
    pub error: Option<String>,
}

struct State {
    requests: Vec<FetchRequest>,
    served: Vec<usize>,
    next: usize,
}

/// Plays exchanges in order, spending wheel time.
pub struct ScriptedFetch {
    exchanges: Vec<ScriptedExchange>,
    wheel: Arc<TimerWheel>,
    state: Arc<Mutex<State>>,
}

fn locked(state: &Mutex<State>) -> MutexGuard<'_, State> {
    state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

impl ScriptedFetch {
    pub fn new(exchanges: Vec<ScriptedExchange>, wheel: Arc<TimerWheel>) -> Self {
        let served = vec![0; exchanges.len()];
        Self {
            exchanges,
            wheel,
            state: Arc::new(Mutex::new(State {
                requests: Vec::new(),
                served,
                next: 0,
            })),
        }
    }

    /// Every request given, in order.
    pub fn requests(&self) -> Vec<FetchRequest> {
        locked(&self.state).requests.clone()
    }

    /// Chunks handed out, per exchange of the script.
    pub fn read(&self) -> Vec<usize> {
        locked(&self.state).served.clone()
    }
}

struct Body {
    chunks: Vec<ScriptedChunk>,
    at: usize,
    exchange: usize,
    end: ScriptedEnd,
    error: Option<String>,
    wheel: Arc<TimerWheel>,
    cancel: CancelToken,
    state: Arc<Mutex<State>>,
    over: bool,
}

impl Chunks for Body {
    fn next(&mut self) -> BoxFuture<'_, Result<Option<Vec<u8>>, FetchError>> {
        Box::pin(async move {
            if self.over {
                return Ok(None);
            }
            if let Some(chunk) = self.chunks.get(self.at) {
                self.wheel.advance(chunk.delay_ms);
                if self.cancel.is_cancelled() {
                    self.over = true;
                    return Err(FetchError::Cancelled);
                }
                locked(&self.state).served[self.exchange] += 1;
                self.at += 1;
                return Ok(Some(chunk.bytes.clone()));
            }
            self.over = true;
            if self.cancel.is_cancelled() {
                return Err(FetchError::Cancelled);
            }
            if self.end == ScriptedEnd::Dropped {
                return Err(FetchError::Failed(
                    self.error
                        .clone()
                        .unwrap_or_else(|| "connection dropped".to_owned()),
                ));
            }
            Ok(None)
        })
    }
}

impl Fetch for ScriptedFetch {
    fn fetch(
        &self,
        request: FetchRequest,
        cancel: CancelToken,
    ) -> BoxFuture<'_, Result<FetchResponse, FetchError>> {
        Box::pin(async move {
            locked(&self.state).requests.push(request);
            if cancel.is_cancelled() {
                return Err(FetchError::Cancelled);
            }
            let at = {
                let mut state = locked(&self.state);
                let at = state.next;
                if at >= self.exchanges.len() {
                    return Err(FetchError::Failed(
                        "the script holds no further exchange".to_owned(),
                    ));
                }
                state.next += 1;
                at
            };
            let exchange = &self.exchanges[at];
            self.wheel.advance(exchange.head_ms);
            if cancel.is_cancelled() {
                return Err(FetchError::Cancelled);
            }
            if exchange.end == ScriptedEnd::Refused {
                return Err(FetchError::Failed(
                    exchange
                        .error
                        .clone()
                        .unwrap_or_else(|| "connection refused".to_owned()),
                ));
            }
            Ok(FetchResponse {
                status: exchange.status,
                headers: response_headers(&exchange.headers),
                chunks: Box::new(Body {
                    chunks: exchange.chunks.clone(),
                    at: 0,
                    exchange: at,
                    end: exchange.end,
                    error: exchange.error.clone(),
                    wheel: Arc::clone(&self.wheel),
                    cancel,
                    state: Arc::clone(&self.state),
                    over: false,
                }),
            })
        })
    }
}
