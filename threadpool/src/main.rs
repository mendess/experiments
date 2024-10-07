use std::{thread::sleep, time::Duration};

use assert_matches::assert_matches;
use threadpool::ThreadPool;

fn main() {
    let pool = ThreadPool::new_with_size(3);
    let promise = pool.new_job().with_priority(1).submit(|| {
        println!("hello world");
        42
    });

    let c_promise = pool.new_job().cancelable().submit(|token| {
        sleep(Duration::from_secs(2));
        if token.should_cancel() {
            return None;
        }
        println!("cancelable hello world");
        Some(())
    });

    sleep(Duration::from_secs(1));
    eprintln!("stopping");
    pool.stop_all();
    let forty_two = promise.wait();

    let wait = c_promise.wait();
    assert_eq!(forty_two, Some(42));
    assert_matches!(wait, Some(None) | None);
}

#[cfg(test)]
mod test {
    use std::{thread::yield_now, time::Duration};

    use assert_matches::assert_matches;

    use threadpool::{JointStopToken, ThreadPool};

    #[must_use]
    fn cancelable_sleep(token: &JointStopToken, duration: Duration) -> bool {
        for _ in 0..duration.as_millis() {
            if token.should_cancel() {
                return true;
            }
            std::thread::sleep(Duration::from_millis(1))
        }
        false
    }

    #[test]
    fn simple() {
        let pool = ThreadPool::new_with_size(1);
        let promise = pool.new_job().submit(|| 42);

        assert_eq!(promise.wait(), Some(42));
    }

    #[test]
    fn cancelable() {
        let pool = ThreadPool::new_with_size(1);
        let promise = pool.new_job().cancelable().submit(|token| {
            while token.should_continue() {
                yield_now();
            }
            None::<()>
        });

        assert_eq!(promise.cancel(), Some(None));
    }

    #[test]
    fn canceling_pool_stops_jobs() {
        let pool = ThreadPool::new_with_size(3);
        let promise = pool.new_job().submit(|| 42);

        let (tx, rx) = oneshot::channel();
        let c_promise = pool.new_job().cancelable().submit(|token| {
            let _ = tx.send(());
            while token.should_continue() {
                yield_now()
            }
            None::<()>
        });

        let _ = rx.recv();
        pool.stop_all();
        let forty_two = promise.wait();

        let wait = c_promise.wait();
        assert_eq!(forty_two, Some(42));
        assert_matches!(wait, Some(None));
    }

    #[test]
    fn canceling_before_job_runs_returns_none() {
        let pool = ThreadPool::new_with_size(2);
        pool.new_job().cancelable().submit(|token| {
            let _ = cancelable_sleep(&token, Duration::from_secs(1));
        });
        pool.new_job().cancelable().submit(|token| {
            let _ = cancelable_sleep(&token, Duration::from_secs(1));
        });

        let c_promise = pool.new_job().cancelable().submit(|_| 1);

        pool.stop_all();

        let wait = c_promise.wait();
        assert_matches!(wait, None);
    }

    #[test]
    fn canceling_a_job_does_not_impact_others() {
        let pool = ThreadPool::new_with_size(4);
        let (tx1, rx1) = oneshot::channel::<()>();
        let promise1 = pool.new_job().cancelable().submit(|_| {
            let _ = rx1.recv();
            Some(42)
        });
        let (tx2, rx2) = oneshot::channel::<()>();
        let promise2 = pool.new_job().cancelable().submit(|token| {
            let _ = rx2.recv();
            while token.should_continue() {
                yield_now();
            }
            None::<()>
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

        assert_eq!(promise1.wait(), Some(Some(42)));
        assert_eq!(result, Some(None));
        assert_eq!(promise3.wait(), Some(42));
    }

    #[test]
    fn stop_all_works_for_empty_jobs() {
        let pool = ThreadPool::new_with_size(2);
        pool.stop_all();
    }

    #[test]
    fn wait_all_works_for_empty_jobs() {
        let pool = ThreadPool::new_with_size(2);
        pool.wait();
    }

    #[test]
    fn wait_all_waits_for_long_running_jobs() {
        let pool = ThreadPool::new_with_size(1);
        let promise1 = pool.new_job().submit(|| {
            std::thread::sleep(Duration::from_millis(100));
            1
        });
        let promise2 =
            pool.new_job()
                .cancelable()
                .submit(|token| if token.should_cancel() { 0 } else { 2 });
        let promise3 = pool.new_job().submit(|| 3);

        pool.wait();
        assert_eq!(promise1.wait(), Some(1));
        assert_eq!(promise2.wait(), Some(2));
        assert_eq!(promise3.wait(), Some(3));
    }

    #[test]
    fn stop_all_does_not_wait_for_long_running_jobs() {
        let pool = ThreadPool::new_with_size(1);
        let promise1 = pool.new_job().cancelable().submit(|token| {
            if cancelable_sleep(&token, Duration::from_secs(1)) {
                return None;
            }
            Some(1)
        });
        let promise2 = pool.new_job().submit(|| 2);
        let promise3 = pool.new_job().submit(|| 3);

        pool.stop_all();
        assert_matches!(promise1.wait(), None | Some(None));
        assert_matches!(promise2.wait(), None);
        assert_matches!(promise3.wait(), None);
    }

    #[test]
    #[should_panic]
    fn panics_are_propagated() {
        let pool = ThreadPool::new_with_size(1);

        let fut = pool.new_job().submit(|| panic!("lol"));

        let _ = fut.wait();
    }
}
