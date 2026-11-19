//! The one way a catalog joins reads that run at once.

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

/// Starts every read at once, waits for every one to settle, ends none early. Answers the values
/// in list order, or the first failure in list order.
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
}
