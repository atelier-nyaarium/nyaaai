//! A Fetch that plays a script on virtual time.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex, MutexGuard};

use nyaaai_contract::cancel::CancelToken;
use nyaaai_contract::fetch::{Chunks, Fetch, FetchError, FetchRequest, FetchResponse};
use nyaaai_contract::future::BoxFuture;
use nyaaai_contract::headers::response_headers;
use nyaaai_contract::shapes::step::HeaderList;
use tokio::sync::oneshot;

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

/// A request in line for the lane.
struct Waiter {
    id: u64,
    cancel: CancelToken,
    /// `true` hands the lane over; `false` refuses the request unseen.
    go: oneshot::Sender<bool>,
}

struct LaneState {
    busy: bool,
    line: VecDeque<Waiter>,
    ids: u64,
}

/// The one lane the network serves on.
struct Lane {
    state: Mutex<LaneState>,
}

fn lane_locked(lane: &Lane) -> MutexGuard<'_, LaneState> {
    lane.state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

impl Lane {
    /// The lane goes to the first waiter whose token has not ended; the others are refused unseen.
    fn hand(&self) {
        let mut state = lane_locked(self);
        loop {
            let Some(waiter) = state.line.pop_front() else {
                state.busy = false;
                return;
            };
            if waiter.cancel.is_cancelled() {
                let _ = waiter.go.send(false);
                continue;
            }
            if waiter.go.send(true).is_ok() {
                return;
            }
        }
    }
}

/// The turn on the lane; dropping it hands the lane on.
struct Held {
    lane: Arc<Lane>,
}

impl Drop for Held {
    fn drop(&mut self) {
        self.lane.hand();
    }
}

/// A place in line; dropped unanswered, it leaves, or hands on a lane it was given.
struct Waiting {
    lane: Arc<Lane>,
    id: u64,
    answer: oneshot::Receiver<bool>,
    settled: bool,
}

impl Drop for Waiting {
    fn drop(&mut self) {
        if self.settled {
            return;
        }
        {
            let mut state = lane_locked(&self.lane);
            if let Some(at) = state.line.iter().position(|each| each.id == self.id) {
                state.line.remove(at);
                return;
            }
        }
        if let Ok(true) = self.answer.try_recv() {
            self.lane.hand();
        }
    }
}

/// Takes the lane now, or waits in arrival order; a token that ends while waiting leaves unseen.
async fn enter(lane: &Arc<Lane>, cancel: &CancelToken) -> Result<Held, FetchError> {
    let mut waiting = {
        let mut state = lane_locked(lane);
        if !state.busy {
            state.busy = true;
            return Ok(Held {
                lane: Arc::clone(lane),
            });
        }
        if cancel.is_cancelled() {
            return Err(FetchError::Cancelled);
        }
        let (go, answer) = oneshot::channel();
        state.ids += 1;
        let id = state.ids;
        state.line.push_back(Waiter {
            id,
            cancel: cancel.clone(),
            go,
        });
        Waiting {
            lane: Arc::clone(lane),
            id,
            answer,
            settled: false,
        }
    };
    match cancel.run_until(&mut waiting.answer).await {
        Some(Ok(true)) => {
            waiting.settled = true;
            Ok(Held {
                lane: Arc::clone(lane),
            })
        }
        Some(_) => {
            waiting.settled = true;
            Err(FetchError::Cancelled)
        }
        None => Err(FetchError::Cancelled),
    }
}

/// Plays exchanges in order, spending wheel time, on one lane: a request holds it from its turn
/// until its body ends, fails, is cancelled or is dropped. Later requests wait in arrival order and
/// are seen (recorded, timed) at their turn; a request whose token has ended when its turn comes,
/// or ends while it waits, is refused unrecorded with no time spent. A body neither read to its
/// end nor dropped holds the lane.
pub struct ScriptedFetch {
    exchanges: Vec<ScriptedExchange>,
    wheel: Arc<TimerWheel>,
    state: Arc<Mutex<State>>,
    lane: Arc<Lane>,
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
            lane: Arc::new(Lane {
                state: Mutex::new(LaneState {
                    busy: false,
                    line: VecDeque::new(),
                    ids: 0,
                }),
            }),
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
    /// Held until the body ends, fails, is cancelled or dropped.
    held: Option<Held>,
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
                    self.held = None;
                    return Err(FetchError::Cancelled);
                }
                locked(&self.state).served[self.exchange] += 1;
                self.at += 1;
                return Ok(Some(chunk.bytes.clone()));
            }
            self.over = true;
            self.held = None;
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
            // Any early return drops the turn, which hands the lane on.
            let held = enter(&self.lane, &cancel).await?;
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
                    held: Some(held),
                }),
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use nyaaai_contract::fetch::Method;
    use nyaaai_contract::time::Clock;

    use super::*;

    fn exchange(head_ms: i64, chunks: &[(&str, i64)]) -> ScriptedExchange {
        ScriptedExchange {
            status: 200,
            headers: Vec::new(),
            chunks: chunks
                .iter()
                .map(|(text, delay_ms)| ScriptedChunk {
                    bytes: text.as_bytes().to_vec(),
                    delay_ms: *delay_ms,
                })
                .collect(),
            head_ms,
            end: ScriptedEnd::Closed,
            error: None,
        }
    }

    fn request(url: &str) -> FetchRequest {
        FetchRequest {
            method: Method::Get,
            url: url.to_owned(),
            headers: Vec::new(),
            body: None,
        }
    }

    fn runtime() -> tokio::runtime::Runtime {
        tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("a runtime")
    }

    async fn drain(response: FetchResponse) -> usize {
        let mut chunks = response.chunks;
        let mut count = 0;
        while let Ok(Some(_)) = chunks.next().await {
            count += 1;
        }
        count
    }

    fn urls(played: &ScriptedFetch) -> Vec<String> {
        played
            .requests()
            .iter()
            .map(|sent| sent.url.clone())
            .collect()
    }

    #[test]
    fn two_requests_at_once_are_served_in_turn() {
        runtime().block_on(async {
            let wheel = Arc::new(TimerWheel::new(0));
            let played = Arc::new(ScriptedFetch::new(
                vec![exchange(10, &[("a", 5)]), exchange(3, &[("b", 2)])],
                Arc::clone(&wheel),
            ));
            let first = {
                let played = Arc::clone(&played);
                tokio::spawn(async move {
                    let response = played
                        .fetch(request("a"), CancelToken::new())
                        .await
                        .expect("the lane was free");
                    tokio::task::yield_now().await;
                    drain(response).await
                })
            };
            let second = {
                let played = Arc::clone(&played);
                let wheel = Arc::clone(&wheel);
                tokio::spawn(async move {
                    let response = played
                        .fetch(request("b"), CancelToken::new())
                        .await
                        .expect("handed the lane");
                    let seen = wheel.now_ms();
                    (seen, drain(response).await)
                })
            };
            assert_eq!(first.await.expect("joined"), 1);
            // The second head waits for the first body: 10 + 5, then its own 3.
            assert_eq!(second.await.expect("joined"), (18, 1));
            assert_eq!(urls(&played), ["a", "b"]);
            assert_eq!(wheel.now_ms(), 20);
            assert_eq!(played.read(), [1, 1]);
        });
    }

    #[test]
    fn a_token_that_ends_while_waiting_leaves_unseen() {
        runtime().block_on(async {
            let wheel = Arc::new(TimerWheel::new(0));
            let played = Arc::new(ScriptedFetch::new(
                vec![exchange(1, &[("a", 0)]), exchange(1, &[("c", 0)])],
                Arc::clone(&wheel),
            ));
            let holder = played
                .fetch(request("a"), CancelToken::new())
                .await
                .expect("the lane was free");
            let leaving = CancelToken::new();
            let waiter = {
                let played = Arc::clone(&played);
                let token = leaving.clone();
                tokio::spawn(async move { played.fetch(request("b"), token).await.map(|_| ()) })
            };
            tokio::task::yield_now().await;
            leaving.cancel();
            assert_eq!(waiter.await.expect("joined"), Err(FetchError::Cancelled));
            drop(holder);
            let third = played
                .fetch(request("c"), CancelToken::new())
                .await
                .expect("the lane was handed on");
            drop(third);
            assert_eq!(urls(&played), ["a", "c"]);
        });
    }

    #[test]
    fn one_abort_ends_holder_and_waiter_and_serves_the_next() {
        runtime().block_on(async {
            let wheel = Arc::new(TimerWheel::new(0));
            let played = Arc::new(ScriptedFetch::new(
                vec![exchange(0, &[("a", 0)]), exchange(0, &[("c", 0)])],
                Arc::clone(&wheel),
            ));
            let shared = CancelToken::new();
            let holder = played
                .fetch(request("a"), shared.clone())
                .await
                .expect("the lane was free");
            let waiter = {
                let played = Arc::clone(&played);
                let token = shared.clone();
                tokio::spawn(async move { played.fetch(request("b"), token).await.map(|_| ()) })
            };
            let next = {
                let played = Arc::clone(&played);
                tokio::spawn(async move {
                    played
                        .fetch(request("c"), CancelToken::new())
                        .await
                        .map(|_| ())
                })
            };
            tokio::task::yield_now().await;
            shared.cancel();
            let mut body = holder.chunks;
            assert_eq!(body.next().await, Err(FetchError::Cancelled));
            assert_eq!(waiter.await.expect("joined"), Err(FetchError::Cancelled));
            assert_eq!(next.await.expect("joined"), Ok(()));
            assert_eq!(urls(&played), ["a", "c"]);
        });
    }

    #[test]
    fn a_body_dropped_midway_frees_the_lane() {
        runtime().block_on(async {
            let wheel = Arc::new(TimerWheel::new(0));
            let played = Arc::new(ScriptedFetch::new(
                vec![exchange(0, &[("a", 0), ("b", 0)]), exchange(0, &[("c", 0)])],
                Arc::clone(&wheel),
            ));
            let mut holder = played
                .fetch(request("a"), CancelToken::new())
                .await
                .expect("the lane was free")
                .chunks;
            assert!(holder.next().await.expect("a chunk").is_some());
            let waiter = {
                let played = Arc::clone(&played);
                tokio::spawn(async move {
                    played
                        .fetch(request("b"), CancelToken::new())
                        .await
                        .map(|_| ())
                })
            };
            tokio::task::yield_now().await;
            assert_eq!(urls(&played), ["a"]);
            drop(holder);
            assert_eq!(waiter.await.expect("joined"), Ok(()));
            assert_eq!(urls(&played), ["a", "b"]);
        });
    }

    #[test]
    fn a_request_with_the_lane_free_is_seen_with_an_ended_token() {
        runtime().block_on(async {
            let wheel = Arc::new(TimerWheel::new(0));
            let played = ScriptedFetch::new(vec![exchange(0, &[])], wheel);
            let ended = CancelToken::new();
            ended.cancel();
            let answer = played.fetch(request("a"), ended).await.map(|_| ());
            assert_eq!(answer, Err(FetchError::Cancelled));
            assert_eq!(urls(&played), ["a"]);
        });
    }
}
