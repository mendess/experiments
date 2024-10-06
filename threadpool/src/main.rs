use std::{thread::sleep, time::Duration};

use threapool::ExecutionResult;

mod threapool;

fn main() {
    let pool = threapool::ThreadPool::new_with_size(3);
    let promise = pool.new_job().with_priority(1).submit(|| {
        println!("hello world");
        42
    });

    let c_promise = pool.new_job().cancelable().submit(|token| {
        sleep(Duration::from_secs(2));
        if token.should_cancel() {
            return ExecutionResult::Canceled;
        }
        println!("cancelable hello world");
        ExecutionResult::Returned(())
    });

    sleep(Duration::from_secs(1));
    pool.stop_all();
    let _nothing_at_all = promise.wait();

    let wait = c_promise.wait();
    assert!(
        matches!(wait, Some(ExecutionResult::Canceled) | None),
        "{wait:?}"
    );
}

#[cfg(test)]
mod test {
    use std::{
        thread::{sleep, yield_now},
        time::Duration,
    };

    use assert_matches::assert_matches;

    use crate::threapool::{ExecutionResult, ThreadPool};

    #[test]
    fn simple() {
        let pool = ThreadPool::new_with_size(1);
        let promise = pool.new_job().submit(|| 42);

        assert_eq!(promise.wait(), Some(42));
    }

    #[test]
    fn cancelable() {
        let pool = ThreadPool::new_with_size(1);
        let promise = pool.new_job().cancelable().submit::<_, ()>(|token| {
            while token.should_continue() {
                yield_now();
            }
            ExecutionResult::Canceled
        });

        assert_eq!(promise.cancel(), Some(ExecutionResult::Canceled));
    }

    #[test]
    fn canceling_pool_stops_jobs() {
        let pool = ThreadPool::new_with_size(1);
        let promise = pool.new_job().cancelable().submit::<_, ()>(|token| {
            while token.should_continue() {
                yield_now();
            }
            ExecutionResult::Canceled
        });

        sleep(Duration::from_secs(1));
        pool.stop_all();

        assert_matches!(promise.wait(), None | Some(ExecutionResult::Canceled));
    }

    #[test]
    fn canceling_pool_stops_jobs_no_sleep() {
        let pool = ThreadPool::new_with_size(1);
        let promise = pool.new_job().cancelable().submit::<_, ()>(|token| {
            while token.should_continue() {
                yield_now();
            }
            ExecutionResult::Canceled
        });

        pool.stop_all();

        assert_matches!(promise.wait(), None | Some(ExecutionResult::Canceled));
    }

    #[test]
    fn canceling_a_job_does_not_impact_others() {
        let pool = ThreadPool::new_with_size(4);
        let (tx1, rx1) = oneshot::channel::<()>();
        let promise1 = pool.new_job().cancelable().submit(|_| {
            let _ = rx1.recv();
            ExecutionResult::Returned(42)
        });
        let (tx2, rx2) = oneshot::channel::<()>();
        let promise2 = pool.new_job().cancelable().submit::<_, ()>(|token| {
            let _ = rx2.recv();
            while token.should_continue() {
                yield_now();
            }
            ExecutionResult::Canceled
        });
        let (tx3, rx3) = oneshot::channel::<()>();
        let promise3 = pool.new_job().submit(|| {
            let _ = rx3.recv();
            42
        });

        let _ = tx2.send(());
        let result = promise2.cancel();
        let _ = tx1.send(());
        let _ = tx3.send(());

        assert_eq!(promise1.wait(), Some(ExecutionResult::Returned(42)));
        assert_eq!(result, Some(ExecutionResult::Canceled));
        assert_eq!(promise3.wait(), Some(42));
    }
}
