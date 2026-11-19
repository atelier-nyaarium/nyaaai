//! One entry's key status; a newer read drops the last.

use std::sync::{Arc, Mutex, MutexGuard};

use nyaaai_contract::cancel::CancelToken;
use nyaaai_contract::future::BoxFuture;
use nyaaai_contract::shapes::provider::{KeyState, KeyStateChecking, KeyStateUnreachable};
use nyaaai_contract::time::{Deadline, Scheduler, deadline};
use tokio::runtime::Handle;
use tokio::sync::{oneshot, watch};

pub const KEY_CHECK_MS: i64 = 30_000;

/// A check that ended without a state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Failed;

/// One check: it ends with the deadline's token.
pub type Check =
    Arc<dyn Fn(Arc<Deadline>) -> BoxFuture<'static, Result<KeyState, Failed>> + Send + Sync>;

/// Told each state the reader shows.
pub type Shown = Arc<dyn Fn(KeyState) + Send + Sync>;

struct Inner {
    scheduler: Arc<dyn Scheduler>,
    check: Check,
    shown: Shown,
    last: Mutex<Option<CancelToken>>,
    running: watch::Sender<usize>,
}

fn locked<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Ends a check's count when its task ends, by a panic too.
struct Count(Arc<Inner>);

impl Drop for Count {
    fn drop(&mut self) {
        self.0.running.send_modify(|count| *count -= 1);
    }
}

/// One entry's key status.
#[derive(Clone)]
pub struct KeyReader {
    inner: Arc<Inner>,
}

impl KeyReader {
    /// The deadline of each check comes from `scheduler`.
    pub fn new(scheduler: Arc<dyn Scheduler>, check: Check, shown: Shown) -> Self {
        Self {
            inner: Arc::new(Inner {
                scheduler,
                check,
                shown,
                last: Mutex::new(None),
                running: watch::channel(0).0,
            }),
        }
    }

    /// Shows `checking`, then the check's state unless a newer read or a dispose came first; a
    /// check that fails or outlives its deadline shows `unreachable`. The answer settles when the
    /// check has ended and been shown or dropped. The check runs on a `tokio` runtime; with none,
    /// the read shows `unreachable`.
    pub fn read(&self) -> BoxFuture<'static, ()> {
        let abort = CancelToken::new();
        let previous = locked(&self.inner.last).replace(abort.clone());
        if let Some(previous) = previous {
            previous.cancel();
        }
        (self.inner.shown)(KeyState::Checking(KeyStateChecking {}));
        let limit = Arc::new(deadline(&*self.inner.scheduler, KEY_CHECK_MS, &abort));
        let work = (self.inner.check)(Arc::clone(&limit));
        self.inner.running.send_modify(|count| *count += 1);
        let count = Count(Arc::clone(&self.inner));
        let (done, ended) = oneshot::channel::<()>();
        let held = abort.clone();
        let inner = Arc::clone(&self.inner);
        let finish = async move {
            let _count = count;
            let result = work.await;
            if !abort.is_cancelled() {
                (inner.shown)(match result {
                    Ok(state) => state,
                    Err(Failed) => KeyState::Unreachable(KeyStateUnreachable {}),
                });
            }
            limit.clear();
            drop(_count);
            let _ = done.send(());
        };
        match Handle::try_current() {
            Ok(runtime) => {
                runtime.spawn(finish);
            }
            Err(_) => {
                // No runtime to run the check on: the read cannot reach the server.
                drop(finish);
                if !held.is_cancelled() {
                    (self.inner.shown)(KeyState::Unreachable(KeyStateUnreachable {}));
                }
            }
        }
        Box::pin(async move {
            let _ = ended.await;
        })
    }

    /// A check is running: one a newer read or a dispose dropped counts until it ends.
    pub fn busy(&self) -> bool {
        *self.inner.running.borrow() > 0
    }

    /// Answers when no check is running, having waited for those that were.
    pub fn idle(&self) -> BoxFuture<'static, ()> {
        let mut running = self.inner.running.subscribe();
        Box::pin(async move {
            let _ = running.wait_for(|count| *count == 0).await;
        })
    }

    pub fn dispose(&self) {
        if let Some(last) = locked(&self.inner.last).as_ref() {
            last.cancel();
        }
    }
}

#[cfg(test)]
mod tests {
    use nyaaai_testkit::scheduler::{FakeScheduler, TimerWheel};

    use super::*;

    #[test]
    fn a_read_with_no_runtime_shows_unreachable() {
        let wheel = Arc::new(TimerWheel::new(0));
        let shown: Arc<Mutex<Vec<KeyState>>> = Arc::default();
        let reader = KeyReader::new(
            Arc::new(FakeScheduler::new(Arc::clone(&wheel))),
            Arc::new(|_| Box::pin(async { Ok(KeyState::Checking(KeyStateChecking {})) })),
            {
                let shown = Arc::clone(&shown);
                Arc::new(move |state| locked(&shown).push(state))
            },
        );
        drop(reader.read());
        assert_eq!(
            *locked(&shown),
            vec![
                KeyState::Checking(KeyStateChecking {}),
                KeyState::Unreachable(KeyStateUnreachable {})
            ]
        );
        assert!(!reader.busy());
        assert!(wheel.waiting().is_empty());
    }
}
