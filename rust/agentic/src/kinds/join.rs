//! The one way a catalog joins reads that run at once.

use std::collections::HashSet;
use std::future::Future;
use std::pin::Pin;
use std::task::Poll;

/// Starts every read at once, in list order, and answers in list order.
pub async fn join_all<F: Future>(reads: impl IntoIterator<Item = F>) -> Vec<F::Output> {
    let mut futures: Vec<Pin<Box<F>>> = reads.into_iter().map(Box::pin).collect();
    let mut done: Vec<Option<F::Output>> = futures.iter().map(|_| None).collect();
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

/// A listing's rows read at most this many.
pub const MAX_LISTED: usize = 2000;

/// Reads in flight at once, for a read per listed row.
pub const READS_AT_ONCE: usize = 8;

/// Starts reads in list order, at most `limit` at once. A failure starts no more and waits for
/// those started; the earliest failure in list order is answered.
pub async fn join_limited<T, E, F: Future<Output = Result<T, E>>>(
    reads: impl IntoIterator<Item = F>,
    limit: usize,
) -> Result<Vec<T>, E> {
    let limit = limit.max(1);
    let mut upcoming = reads.into_iter().enumerate();
    let mut flying: Vec<(usize, Pin<Box<F>>)> = Vec::new();
    let mut done: Vec<Option<T>> = Vec::new();
    let mut failure: Option<(usize, E)> = None;
    std::future::poll_fn(|context| {
        loop {
            while failure.is_none() && flying.len() < limit {
                let Some((at, read)) = upcoming.next() else {
                    break;
                };
                done.push(None);
                flying.push((at, Box::pin(read)));
            }
            if flying.is_empty() {
                return Poll::Ready(());
            }
            let mut finished = false;
            let mut at = 0;
            while at < flying.len() {
                match flying[at].1.as_mut().poll(context) {
                    Poll::Ready(result) => {
                        let (place, _) = flying.remove(at);
                        finished = true;
                        match result {
                            Ok(held) => done[place] = Some(held),
                            Err(error) => {
                                if failure.as_ref().is_none_or(|(held, _)| place < *held) {
                                    failure = Some((place, error));
                                }
                            }
                        }
                    }
                    Poll::Pending => at += 1,
                }
            }
            if !finished {
                return Poll::Pending;
            }
        }
    })
    .await;
    match failure {
        Some((_, error)) => Err(error),
        None => Ok(done.into_iter().flatten().collect()),
    }
}

/// A read that cannot fail, as one a limited join takes.
pub async fn ok<T>(read: impl Future<Output = T>) -> Result<T, std::convert::Infallible> {
    Ok(read.await)
}

/// The first `max` items with distinct keys, in order.
pub fn first_distinct<T>(items: Vec<T>, key_of: impl Fn(&T) -> String, max: usize) -> Vec<T> {
    let mut seen = HashSet::new();
    let mut kept = Vec::new();
    for item in items {
        if kept.len() >= max {
            break;
        }
        if seen.insert(key_of(&item)) {
            kept.push(item);
        }
    }
    kept
}

/// Waits for every read; answers values in list order, or the first failure.
pub async fn join_settled<T, E, F: Future<Output = Result<T, E>>>(
    reads: impl IntoIterator<Item = F>,
) -> Result<Vec<T>, E> {
    join_all(reads).await.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use super::*;

    fn runtime() -> tokio::runtime::Runtime {
        tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("a runtime")
    }

    async fn read(log: Arc<Mutex<Vec<i32>>>, name: i32, steps: u32) -> i32 {
        log.lock().expect("held").push(name);
        for _ in 0..steps {
            tokio::task::yield_now().await;
        }
        name
    }

    #[test]
    fn reads_start_in_list_order_and_answer_in_list_order() {
        runtime().block_on(async {
            let log = Arc::new(Mutex::new(Vec::new()));
            let answers = join_all([
                read(Arc::clone(&log), 1, 3),
                read(Arc::clone(&log), 2, 0),
                read(Arc::clone(&log), 3, 1),
            ])
            .await;
            assert_eq!(answers, [1, 2, 3]);
            assert_eq!(*log.lock().expect("held"), [1, 2, 3]);
        });
    }

    #[test]
    fn a_settled_join_waits_for_every_read_and_answers_the_first_failure_in_list_order() {
        runtime().block_on(async {
            let finished = Arc::new(Mutex::new(Vec::new()));
            let each = |name: i32, steps: u32, fails: bool| {
                let finished = Arc::clone(&finished);
                async move {
                    for _ in 0..steps {
                        tokio::task::yield_now().await;
                    }
                    finished.lock().expect("held").push(name);
                    if fails { Err(name) } else { Ok(name) }
                }
            };
            let answer =
                join_settled([each(1, 4, true), each(2, 0, true), each(3, 2, false)]).await;
            assert_eq!(answer, Err(1));
            assert_eq!(*finished.lock().expect("held"), [2, 3, 1]);
            let values = join_settled([each(4, 1, false), each(5, 0, false)]).await;
            assert_eq!(values, Ok(vec![4, 5]));
        });
    }

    #[test]
    fn a_limited_join_never_has_more_reads_in_flight_than_its_limit() {
        runtime().block_on(async {
            let flying = Arc::new(Mutex::new((0usize, 0usize)));
            let reads: Vec<_> = (0..20)
                .map(|name| {
                    let flying = Arc::clone(&flying);
                    async move {
                        {
                            let mut held = flying.lock().expect("held");
                            held.0 += 1;
                            held.1 = held.1.max(held.0);
                        }
                        for _ in 0..3 {
                            tokio::task::yield_now().await;
                        }
                        flying.lock().expect("held").0 -= 1;
                        Ok::<_, ()>(name)
                    }
                })
                .collect();
            let answers = join_limited(reads, 3).await.expect("no failure");
            assert_eq!(answers, (0..20).collect::<Vec<_>>());
            assert_eq!(flying.lock().expect("held").1, 3);
        });
    }

    #[test]
    fn a_limited_join_after_failures_starts_no_read_and_answers_the_earliest_in_list_order() {
        runtime().block_on(async {
            let started = Arc::new(Mutex::new(Vec::new()));
            let reads: Vec<_> = (0..4)
                .map(|name: i32| {
                    let started = Arc::clone(&started);
                    async move {
                        started.lock().expect("held").push(name);
                        for _ in 0..if name == 0 { 6 } else { 1 } {
                            tokio::task::yield_now().await;
                        }
                        if name < 2 { Err(name) } else { Ok(name) }
                    }
                })
                .collect();
            assert_eq!(join_limited(reads, 2).await, Err(0));
            assert_eq!(*started.lock().expect("held"), [0, 1]);
        });
    }
}
