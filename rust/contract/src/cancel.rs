//! Cancel token.

use std::future::{Future, poll_fn};
use std::pin::pin;
use std::sync::Arc;
use std::task::Poll;

use tokio::sync::watch;

/// Clones share one flag.
#[derive(Clone)]
pub struct CancelToken {
    state: Arc<watch::Sender<bool>>,
}

impl CancelToken {
    pub fn new() -> Self {
        Self {
            state: Arc::new(watch::channel(false).0),
        }
    }

    pub fn cancel(&self) {
        self.state.send_replace(true);
    }

    pub fn is_cancelled(&self) -> bool {
        *self.state.borrow()
    }

    /// Ends once cancelled.
    pub async fn cancelled(&self) {
        let mut watcher = self.state.subscribe();
        let _ = watcher.wait_for(|held| *held).await;
    }

    /// `None` when cancelled first; a cancelled token never polls the work.
    pub async fn run_until<F: Future>(&self, work: F) -> Option<F::Output> {
        let mut work = pin!(work);
        let mut cancelled = pin!(self.cancelled());
        poll_fn(|context| {
            if cancelled.as_mut().poll(context).is_ready() {
                return Poll::Ready(None);
            }
            work.as_mut().poll(context).map(Some)
        })
        .await
    }
}

impl Default for CancelToken {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cancels_every_clone_once() {
        let token = CancelToken::new();
        let other = token.clone();
        assert!(!other.is_cancelled());
        token.cancel();
        token.cancel();
        assert!(other.is_cancelled());
    }

    #[test]
    fn ends_the_work_a_cancel_beats() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        runtime.block_on(async {
            let token = CancelToken::new();
            assert_eq!(token.run_until(async { 7 }).await, Some(7));
            token.cancel();
            token.cancelled().await;
            assert_eq!(token.run_until(async { 7 }).await, None);
            assert_eq!(token.run_until(std::future::pending::<()>()).await, None);
        });
    }
}
