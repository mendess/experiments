mod promise;
mod stop_token;

use std::{
    any::Any,
    collections::BinaryHeap,
    panic::{self, AssertUnwindSafe, UnwindSafe},
    sync::Arc,
    thread,
};

use parking_lot::{Condvar, Mutex};
use promise::Promise;
pub use stop_token::{JointStopToken, StopToken};

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

pub struct JobBuilder<'t, S> {
    priority: Priority,
    cancelable: S,
    pool: &'t ThreadPool,
}

impl<'t, S> JobBuilder<'t, S> {
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

    fn submit_impl<R>(
        self,
        rx: oneshot::Receiver<Result<R, Box<dyn Any + Send>>>,
        job: Job,
    ) -> Promise<R, S> {
        self.pool.submit(job);
        Promise {
            cancelable: self.cancelable,
            response: rx,
        }
    }
}

impl<'t> JobBuilder<'t, Uncacelable> {
    pub fn submit_and_forget<F>(self, f: F)
    where
        F: FnOnce() + Send + UnwindSafe + 'static,
    {
        self.pool.submit(Job {
            priority: self.priority,
            fun: Box::new(|| {
                let _ = panic::catch_unwind(f);
            }),
        })
    }

    pub fn submit<F, R>(self, f: F) -> Promise<R, Uncacelable>
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        let (tx, rx) = oneshot::channel();
        let job = Job {
            priority: self.priority,
            fun: Box::new(move || {
                let _ = tx.send(panic::catch_unwind(AssertUnwindSafe(f)));
            }),
        };
        self.submit_impl(rx, job)
    }
}

impl<'t> JobBuilder<'t, Cancelable> {
    pub fn submit_and_forget<F>(self, f: F)
    where
        F: FnOnce(StopToken) + Send + UnwindSafe + 'static,
    {
        let stop_token = self.pool.pool_stop_token.clone();
        self.pool.submit(Job {
            priority: self.priority,
            fun: Box::new(move || {
                let _ = panic::catch_unwind(|| f(stop_token));
            }),
        })
    }

    pub fn submit<F, R>(self, f: F) -> Promise<R, Cancelable>
    where
        F: FnOnce(JointStopToken) -> R + Send + 'static,
        R: Send + 'static,
    {
        let (tx, rx) = oneshot::channel();
        let stop_token =
            JointStopToken(self.pool.pool_stop_token.clone(), self.cancelable.0.clone());
        let job = Job {
            priority: self.priority,
            fun: Box::new(move || {
                let _ = tx.send(panic::catch_unwind(AssertUnwindSafe(|| f(stop_token))));
            }),
        };
        self.submit_impl(rx, job)
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
                    let ThreadPoolInner { state, has_jobs } = &*inner;

                    loop {
                        let mut guard = state.lock();
                        let job = loop {
                            if pool_stop_token.should_cancel() {
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
