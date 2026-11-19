//! Cancel token.

use std::future::{Future, poll_fn};
use std::pin::pin;
use std::sync::{Arc, Mutex, Weak};
use std::task::Poll;

use tokio::sync::watch;

struct State {
    flag: watch::Sender<bool>,
    children: Mutex<Vec<Weak<State>>>,
}

impl State {
    fn cancel(&self) {
        self.flag.send_replace(true);
        let children = std::mem::take(
            &mut *self
                .children
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()),
        );
        for child in children {
            if let Some(child) = child.upgrade() {
                child.cancel();
            }
        }
    }
}

/// Ends a subscription; call it once.
pub type Unsubscribe = Box<dyn FnOnce() + Send + Sync>;

/// Clones share one flag.
#[derive(Clone)]
pub struct CancelToken {
    state: Arc<State>,
}

impl CancelToken {
    pub fn new() -> Self {
        Self {
            state: Arc::new(State {
                flag: watch::channel(false).0,
                children: Mutex::new(Vec::new()),
            }),
        }
    }

    /// A child: cancelled by this token or itself, never cancelling this one.
    pub fn child(&self) -> Self {
        let child = Self::new();
        let mut children = self
            .state
            .children
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        children.retain(|held| held.strong_count() > 0);
        if self.is_cancelled() {
            child.cancel();
        } else {
            children.push(Arc::downgrade(&child.state));
        }
        child
    }

    pub fn cancel(&self) {
        self.state.cancel();
    }

    pub fn is_cancelled(&self) -> bool {
        *self.state.flag.borrow()
    }

    /// Ends once cancelled.
    pub async fn cancelled(&self) {
        let mut watcher = self.state.flag.subscribe();
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
    fn a_child_follows_its_parent_alone() {
        let parent = CancelToken::new();
        let child = parent.child();
        child.cancel();
        assert!(child.is_cancelled());
        assert!(!parent.is_cancelled());
        let other = parent.child();
        let grand = other.child();
        parent.cancel();
        assert!(other.is_cancelled() && grand.is_cancelled());
        assert!(parent.child().is_cancelled());
    }

    #[test]
    fn a_child_wakes_when_its_parent_cancels() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        runtime.block_on(async {
            let parent = CancelToken::new();
            let child = parent.child();
            let waiting = child.run_until(std::future::pending::<()>());
            parent.cancel();
            assert_eq!(waiting.await, None);
        });
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
