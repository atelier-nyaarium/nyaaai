//! One shared, cached refresh with a wait after a failure. A refresh runs on a `tokio` runtime;
//! with none, a call answers that it has no data.

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{Arc, Mutex, MutexGuard};

use nyaaai_contract::cancel::CancelToken;
use nyaaai_contract::future::BoxFuture;
use tokio::runtime::Handle;
use tokio::sync::watch;

/// The twelve hours a good listing holds.
pub const TTL_MS: i64 = 12 * 60 * 60 * 1000;
/// The wait after a failed refresh.
pub const RETRY_MS: i64 = 60_000;
/// The time one refresh may take.
pub const FETCH_MS: i64 = 30_000;

/// Tells the owner a refresh stored its data; what it does changes nothing.
pub type Told = Box<dyn FnOnce() + Send>;

/// How a refresh ended.
pub enum Refreshed<T> {
    /// The data is stored first, then `told` runs outside any lock.
    Good {
        value: T,
        at: i64,
        told: Option<Told>,
    },
    Failed {
        at: i64,
    },
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

fn locked<T>(state: &Mutex<State<T>>) -> MutexGuard<'_, State<T>> {
    state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Ends a refresh's mark when the task ends, by a panic too.
struct Finish<T> {
    state: Arc<Mutex<State<T>>>,
    send: Option<watch::Sender<Option<Option<T>>>>,
}

impl<T> Finish<T> {
    fn end(&mut self, result: Option<T>) {
        if let Some(send) = self.send.take() {
            locked(&self.state).inflight = None;
            let _ = send.send(Some(result));
        }
    }
}

impl<T> Drop for Finish<T> {
    fn drop(&mut self) {
        self.end(None);
    }
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

    /// The last good data, whatever its age.
    pub fn good(&self) -> Option<T> {
        locked(&self.state)
            .good
            .as_ref()
            .map(|(value, _)| value.clone())
    }

    /// A failure is forgotten, so the next call may refresh.
    pub fn retry(&self) {
        locked(&self.state).failed_at = None;
    }

    /// Whether a refresh is running.
    pub fn running(&self) -> bool {
        locked(&self.state).inflight.is_some()
    }

    /// Answers when no refresh is running, having waited for the one that was.
    pub fn idle(&self) -> impl Future<Output = ()> + Send + use<T> {
        let state = Arc::clone(&self.state);
        async move {
            loop {
                let running = locked(&state).inflight.clone();
                let Some(mut answer) = running else {
                    return;
                };
                let _ = answer.wait_for(Option::is_some).await;
            }
        }
    }

    /// Serves the good data while it is fresh, or while a failure is recent; else one shared refresh.
    /// `start` runs at the call and answers the refresh, which runs on its own.
    pub fn fetched(
        &self,
        now: i64,
        ttl: i64,
        start: impl FnOnce() -> BoxFuture<'static, Refreshed<T>>,
    ) -> Waiter<T> {
        let state = locked(&self.state);
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
        let state = locked(&self.state);
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
        let Ok(runtime) = Handle::try_current() else {
            return Waiter::Ready(None);
        };
        let refresh = start();
        let (send, answer) = watch::channel(None);
        let mut finish = Finish {
            state: Arc::clone(&self.state),
            send: Some(send),
        };
        let shared = Arc::clone(&self.state);
        // The mark below is set only once the task exists; the task waits for this lock.
        runtime.spawn(async move {
            let outcome = refresh.await;
            let (result, told) = {
                let mut state = locked(&shared);
                match outcome {
                    Refreshed::Good { value, at, told } => {
                        state.good = Some((value.clone(), at));
                        state.failed_at = None;
                        (Some(value), told)
                    }
                    Refreshed::Failed { at } => {
                        state.failed_at = Some(at);
                        (state.good.as_ref().map(|(value, _)| value.clone()), None)
                    }
                }
            };
            if let Some(told) = told {
                let _ = catch_unwind(AssertUnwindSafe(told));
            }
            finish.end(result);
        });
        state.inflight = Some(answer.clone());
        Waiter::Pending(answer)
    }
}

impl<T: Clone + Send + Sync + 'static> Default for Fetcher<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn runtime() -> tokio::runtime::Runtime {
        tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("a runtime")
    }

    fn good(value: i32) -> BoxFuture<'static, Refreshed<i32>> {
        Box::pin(async move {
            Refreshed::Good {
                value,
                at: 0,
                told: None,
            }
        })
    }

    #[test]
    fn no_runtime_answers_no_data_and_a_later_call_refreshes() {
        let fetcher = Fetcher::new();
        let mut started = false;
        let waiter = fetcher.fetched(0, 10, || {
            started = true;
            good(1)
        });
        assert!(matches!(waiter, Waiter::Ready(None)));
        assert!(!started);
        let answer = runtime().block_on(async { fetcher.fetched(0, 10, || good(2)).wait().await });
        assert_eq!(answer, Some(2));
    }

    #[test]
    fn a_refresh_that_panics_tells_no_data_and_the_next_call_tries_again() {
        runtime().block_on(async {
            let fetcher = Fetcher::new();
            let waiter = fetcher.fetched(0, 10, || Box::pin(async { panic!("a decoder bug") }));
            assert_eq!(waiter.wait().await, None);
            fetcher.idle().await;
            let answer = fetcher.fetched(0, 10, || good(3)).wait().await;
            assert_eq!(answer, Some(3));
        });
    }

    #[test]
    fn a_start_that_panics_leaves_no_mark() {
        runtime().block_on(async {
            let fetcher = Fetcher::new();
            let failed = catch_unwind(AssertUnwindSafe(|| {
                fetcher.fetched(0, 10, || panic!("no deadline"))
            }));
            assert!(failed.is_err());
            let answer = fetcher.fetched(0, 10, || good(4)).wait().await;
            assert_eq!(answer, Some(4));
        });
    }

    #[test]
    fn a_told_that_panics_changes_nothing() {
        runtime().block_on(async {
            let fetcher = Fetcher::new();
            let waiter = fetcher.fetched(0, 10, || {
                Box::pin(async {
                    Refreshed::Good {
                        value: 5,
                        at: 0,
                        told: Some(Box::new(|| panic!("a listener"))),
                    }
                })
            });
            assert_eq!(waiter.wait().await, Some(5));
            assert_eq!(fetcher.good(), Some(5));
            fetcher.idle().await;
            assert_eq!(fetcher.fetched(1, 10, || good(6)).wait().await, Some(5));
        });
    }

    #[test]
    fn the_data_is_stored_before_told_runs() {
        runtime().block_on(async {
            let fetcher = Arc::new(Fetcher::new());
            let seen = Arc::new(Mutex::new(None));
            let reader = Arc::clone(&fetcher);
            let told_seen = Arc::clone(&seen);
            let waiter = fetcher.fetched(0, 10, move || {
                Box::pin(async move {
                    Refreshed::Good {
                        value: 7,
                        at: 0,
                        told: Some(Box::new(move || {
                            *told_seen.lock().expect("held") = reader.good();
                        })),
                    }
                })
            });
            waiter.wait().await;
            assert_eq!(*seen.lock().expect("held"), Some(7));
        });
    }
}
