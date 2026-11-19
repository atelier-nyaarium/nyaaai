//! One entry's key status; a newer read drops the last.

use std::future::Future;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::pin::Pin;
use std::sync::{Arc, Mutex, MutexGuard};
use std::task::{Context, Poll};

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

/// One check, ended by the deadline's token; `Ok(None)` ended without effect.
pub type Check = Arc<
    dyn Fn(Arc<Deadline>) -> BoxFuture<'static, Result<Option<KeyState>, Failed>> + Send + Sync,
>;

/// Told each state the reader shows.
pub type Shown = Arc<dyn Fn(KeyState) + Send + Sync>;

struct Inner {
    scheduler: Arc<dyn Scheduler>,
    runtime: Handle,
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

/// A future whose panic ends it with none.
struct Caught<F>(Pin<Box<F>>);

impl<F: Future> Future for Caught<F> {
    type Output = Option<F::Output>;

    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        match catch_unwind(AssertUnwindSafe(|| self.0.as_mut().poll(context))) {
            Ok(Poll::Ready(held)) => Poll::Ready(Some(held)),
            Ok(Poll::Pending) => Poll::Pending,
            Err(_) => Poll::Ready(None),
        }
    }
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
    /// The deadline of each check comes from `scheduler`; a check runs on `runtime`.
    pub fn new(scheduler: Arc<dyn Scheduler>, runtime: Handle, check: Check, shown: Shown) -> Self {
        Self {
            inner: Arc::new(Inner {
                scheduler,
                runtime,
                check,
                shown,
                last: Mutex::new(None),
                running: watch::channel(0).0,
            }),
        }
    }

    /// Shows `checking`, then the state unless a newer read or dispose won; failure or deadline shows `unreachable`.
    pub fn read(&self) -> BoxFuture<'static, ()> {
        let abort = CancelToken::new();
        let previous = locked(&self.inner.last).replace(abort.clone());
        if let Some(previous) = previous {
            previous.cancel();
        }
        (self.inner.shown)(KeyState::Checking(KeyStateChecking {}));
        let limit = Arc::new(deadline(&*self.inner.scheduler, KEY_CHECK_MS, &abort));
        // A check that throws at once is a failed check and leaves no timer.
        let Ok(work) = catch_unwind(AssertUnwindSafe(|| (self.inner.check)(Arc::clone(&limit))))
        else {
            if !abort.is_cancelled() {
                (self.inner.shown)(KeyState::Unreachable(KeyStateUnreachable {}));
            }
            limit.clear();
            return Box::pin(async {});
        };
        self.inner.running.send_modify(|count| *count += 1);
        let count = Count(Arc::clone(&self.inner));
        let (done, ended) = oneshot::channel::<()>();
        let inner = Arc::clone(&self.inner);
        let finish = async move {
            let _count = count;
            // Raced against the deadline: a miss shows `unreachable`, a later answer is dropped.
            let raced = limit.token().run_until(Caught(Box::pin(work))).await;
            if !abort.is_cancelled() {
                match raced.flatten() {
                    Some(Ok(Some(state))) => (inner.shown)(state),
                    Some(Ok(None)) => {}
                    Some(Err(Failed)) | None => {
                        (inner.shown)(KeyState::Unreachable(KeyStateUnreachable {}));
                    }
                }
            }
            limit.clear();
            drop(_count);
            let _ = done.send(());
        };
        self.inner.runtime.spawn(finish);
        Box::pin(async move {
            let _ = ended.await;
        })
    }

    /// Running, counting a dropped check until it ends.
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
    use nyaaai_contract::shapes::provider::KeyStateNone;
    use nyaaai_contract::time::SystemScheduler;

    use super::*;

    #[test]
    fn a_read_from_a_plain_thread_sets_its_deadline_and_reaches_its_state() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .unwrap();
        let shown: Arc<Mutex<Vec<KeyState>>> = Arc::default();
        let reader = KeyReader::new(
            Arc::new(SystemScheduler::new(runtime.handle().clone())),
            runtime.handle().clone(),
            Arc::new(|_| Box::pin(async { Ok(Some(KeyState::None(KeyStateNone {}))) })),
            {
                let shown = Arc::clone(&shown);
                Arc::new(move |state| locked(&shown).push(state))
            },
        );
        let read = std::thread::scope(|scope| scope.spawn(|| reader.read()).join().unwrap());
        runtime.block_on(read);
        assert_eq!(
            *locked(&shown),
            vec![
                KeyState::Checking(KeyStateChecking {}),
                KeyState::None(KeyStateNone {})
            ]
        );
        assert!(!reader.busy());
    }
}
