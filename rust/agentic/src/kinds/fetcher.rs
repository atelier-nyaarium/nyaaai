//! One shared, cached refresh with a wait after a failure.

use std::sync::{Arc, Mutex, MutexGuard};

use nyaaai_contract::cancel::CancelToken;
use nyaaai_contract::future::BoxFuture;
use tokio::sync::watch;

/// The twelve hours a good listing holds.
pub const TTL_MS: i64 = 12 * 60 * 60 * 1000;
/// The wait after a failed refresh.
pub const RETRY_MS: i64 = 60_000;
/// The time one refresh may take.
pub const FETCH_MS: i64 = 30_000;

/// How a refresh ended.
pub enum Refreshed<T> {
    Good { value: T, at: i64 },
    Failed { at: i64 },
}

/// The caller's cancel ended its own wait.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Aborted;

type Answer<T> = watch::Receiver<Option<Option<T>>>;

struct State<T> {
    good: Option<(T, i64)>,
    failed_at: Option<i64>,
    inflight: Option<Answer<T>>,
}

/// What a call waits for.
pub enum Waiter<T> {
    Ready(Option<T>),
    Pending(Answer<T>),
}

impl<T: Clone> Waiter<T> {
    pub async fn wait(self) -> Option<T> {
        match self {
            Self::Ready(held) => held,
            Self::Pending(mut answer) => match answer.wait_for(Option::is_some).await {
                Ok(held) => held.clone().flatten(),
                Err(_) => None,
            },
        }
    }

    /// The answer, or `Aborted` once the caller's token is cancelled (at once when it already is).
    pub async fn raced(self, cancel: Option<&CancelToken>) -> Result<Option<T>, Aborted> {
        match cancel {
            None => Ok(self.wait().await),
            Some(token) if token.is_cancelled() => Err(Aborted),
            Some(token) => token.run_until(self.wait()).await.ok_or(Aborted),
        }
    }
}

/// The last good data and the one refresh in flight.
pub struct Fetcher<T> {
    state: Arc<Mutex<State<T>>>,
}

impl<T: Clone + Send + Sync + 'static> Fetcher<T> {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(State {
                good: None,
                failed_at: None,
                inflight: None,
            })),
        }
    }

    fn locked(&self) -> MutexGuard<'_, State<T>> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// The last good data, whatever its age.
    pub fn good(&self) -> Option<T> {
        self.locked().good.as_ref().map(|(value, _)| value.clone())
    }

    /// A failure is forgotten, so the next call may refresh.
    pub fn retry(&self) {
        self.locked().failed_at = None;
    }

    /// Serves the good data while it is fresh, or while a failure is recent; else one shared refresh.
    /// `start` runs now and answers the refresh, which runs on its own.
    pub fn fetched(
        &self,
        now: i64,
        ttl: i64,
        start: impl FnOnce() -> BoxFuture<'static, Refreshed<T>>,
    ) -> Waiter<T> {
        let state = self.locked();
        if let Some((value, at)) = &state.good
            && now - at < ttl
        {
            return Waiter::Ready(Some(value.clone()));
        }
        if state
            .failed_at
            .is_some_and(|failed| now - failed < RETRY_MS)
        {
            return Waiter::Ready(state.good.as_ref().map(|(value, _)| value.clone()));
        }
        self.join_or_start(state, start)
    }

    /// Refreshes whatever the age of the data, or joins the refresh in flight.
    pub fn refresh_now(
        &self,
        start: impl FnOnce() -> BoxFuture<'static, Refreshed<T>>,
    ) -> Waiter<T> {
        let state = self.locked();
        self.join_or_start(state, start)
    }

    fn join_or_start(
        &self,
        mut state: MutexGuard<'_, State<T>>,
        start: impl FnOnce() -> BoxFuture<'static, Refreshed<T>>,
    ) -> Waiter<T> {
        if let Some(answer) = &state.inflight {
            return Waiter::Pending(answer.clone());
        }
        let (send, answer) = watch::channel(None);
        state.inflight = Some(answer.clone());
        let refresh = start();
        let shared = Arc::clone(&self.state);
        tokio::spawn(async move {
            let outcome = refresh.await;
            let result = {
                let mut state = shared
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                state.inflight = None;
                match outcome {
                    Refreshed::Good { value, at } => {
                        state.good = Some((value.clone(), at));
                        state.failed_at = None;
                        Some(value)
                    }
                    Refreshed::Failed { at } => {
                        state.failed_at = Some(at);
                        state.good.as_ref().map(|(value, _)| value.clone())
                    }
                }
            };
            let _ = send.send(Some(result));
        });
        Waiter::Pending(answer)
    }
}

impl<T: Clone + Send + Sync + 'static> Default for Fetcher<T> {
    fn default() -> Self {
        Self::new()
    }
}
