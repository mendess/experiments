mod promise;
mod stop_token;

use std::{
    collections::BinaryHeap,
    panic::{self, AssertUnwindSafe},
    sync::Arc,
    thread,
};

use parking_lot::{Condvar, Mutex};
use promise::Promise;
use stop_token::StopToken;
pub use stop_token::{job_should_cancel, job_should_continue};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, Debug, Default)]
pub struct Priority(i8);

impl From<i8> for Priority {
    fn from(value: i8) -> Self {
        Self(value)
    }
}

#[derive(Default)]
pub struct Cancelable(StopToken);
pub struct Uncacelable;

pub trait CancelationPolicy {
    fn token(&self) -> Option<StopToken>;
}

impl CancelationPolicy for Cancelable {
    fn token(&self) -> Option<StopToken> {
        Some(self.0.clone())
    }
}

impl CancelationPolicy for Uncacelable {
    fn token(&self) -> Option<StopToken> {
        None
    }
}

pub struct JobBuilder<'t, S> {
    priority: Priority,
    cancelable: S,
    pool: &'t ThreadPool,
}

impl<'t, S: CancelationPolicy> JobBuilder<'t, S> {
    pub fn cancelable(self) -> JobBuilder<'t, Cancelable> {
        JobBuilder {
            priority: self.priority,
            cancelable: Cancelable::default(),
            pool: self.pool,
        }
    }

    pub fn with_priority(self, priority: impl Into<Priority>) -> Self {
        Self {
            priority: priority.into(),
            ..self
        }
    }

    pub fn submit<F>(self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.pool.submit(Job {
            priority: self.priority,
            stop_token: self.cancelable.token(),
            fun: Box::new(|| {
                let _ = panic::catch_unwind(AssertUnwindSafe(f));
            }),
        })
    }

    pub fn output<F, R>(self, f: F) -> Promise<R, S>
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        let (tx, rx) = oneshot::channel();
        self.pool.submit(Job {
            priority: self.priority,
            stop_token: self.cancelable.token(),
            fun: Box::new(move || {
                let _ = tx.send(panic::catch_unwind(AssertUnwindSafe(f)));
            }),
        });
        Promise {
            cancelable: self.cancelable,
            response: rx,
        }
    }
}

#[derive(Default)]
struct ThreadPoolState {
    queue: BinaryHeap<Job>,
    shutting_down: bool,
}

struct ThreadPoolInner {
    state: Mutex<ThreadPoolState>,
    has_jobs: Condvar,
}

pub struct ThreadPool {
    pool_stop_token: StopToken,
    threads: Box<[thread::JoinHandle<()>]>,
    inner: Arc<ThreadPoolInner>,
}

impl Default for ThreadPool {
    fn default() -> Self {
        Self::new()
    }
}

impl ThreadPool {
    pub fn new() -> Self {
        Self::new_with_size(std::thread::available_parallelism().unwrap().get())
    }

    pub fn new_with_size(size: usize) -> Self {
        let inner = Arc::new(ThreadPoolInner {
            state: Mutex::new(Default::default()),
            has_jobs: Condvar::new(),
        });

        let pool_stop_token = StopToken::default();
        let threads = (0..size)
            .map(|_| {
                let inner = inner.clone();
                let pool_stop_token = pool_stop_token.clone();
                thread::spawn(move || {
                    stop_token::init_worker_token(pool_stop_token);
                    let ThreadPoolInner { state, has_jobs } = &*inner;

                    loop {
                        let mut guard = state.lock();
                        let job = loop {
                            if stop_token::worker_should_cancel() {
                                return;
                            }
                            if let Some(job) = guard.queue.pop() {
                                break job;
                            }
                            if guard.shutting_down {
                                return;
                            }
                            has_jobs.wait(&mut guard);
                        };
                        drop(guard);
                        let _guard = job.stop_token.map(stop_token::set_job_token);
                        (job.fun)();
                    }
                })
            })
            .collect();

        Self {
            inner,
            pool_stop_token,
            threads,
        }
    }

    pub fn new_job(&self) -> JobBuilder<Uncacelable> {
        JobBuilder {
            priority: Priority::default(),
            cancelable: Uncacelable,
            pool: self,
        }
    }

    fn submit(&self, job: Job) {
        self.inner.state.lock().queue.push(job);
        self.inner.has_jobs.notify_one();
    }

    pub fn stop_all(self) {
        {
            let _g = self.inner.state.lock();
            self.pool_stop_token.cancel();
        }
        self.inner.has_jobs.notify_all();
        for t in self.threads {
            let _ = t.join();
        }
    }

    pub fn wait(self) {
        self.inner.state.lock().shutting_down = true;
        self.inner.has_jobs.notify_all();
        for t in self.threads {
            let _ = t.join();
        }
    }
}

struct Job {
    priority: Priority,
    stop_token: Option<StopToken>,
    fun: Box<dyn FnOnce() + Send>,
}

impl PartialEq for Job {
    fn eq(&self, other: &Self) -> bool {
        self.priority.eq(&other.priority)
    }
}

impl Eq for Job {}

impl PartialOrd for Job {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Job {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.priority.cmp(&other.priority)
    }
}
