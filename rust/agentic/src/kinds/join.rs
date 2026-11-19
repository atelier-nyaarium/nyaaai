//! Futures run at once, each started in its place.

use std::future::Future;
use std::pin::Pin;
use std::task::Poll;

/// The answers in the order of the futures, which are polled in order on every wake.
pub async fn join_all<'a, T>(
    mut futures: Vec<Pin<Box<dyn Future<Output = T> + Send + 'a>>>,
) -> Vec<T> {
    let mut done: Vec<Option<T>> = futures.iter().map(|_| None).collect();
    std::future::poll_fn(|context| {
        for (at, future) in futures.iter_mut().enumerate() {
            if done[at].is_none()
                && let Poll::Ready(held) = future.as_mut().poll(context)
            {
                done[at] = Some(held);
            }
        }
        if done.iter().all(Option::is_some) {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    })
    .await;
    done.into_iter().flatten().collect()
}
