#![allow(dead_code)]

mod stop_token;

use std::{
    collections::BinaryHeap,
    fmt,
    sync::Arc,
    thread::{self, available_parallelism, JoinHandle},
};

use parking_lot::{Condvar, Mutex, MutexGuard};
pub use stop_token::{ExecutionResult, StopToken};

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

pub struct Promise<R, S> {
    cancelable: S,
    response: oneshot::Receiver<R>,
}

impl<R, S> Promise<R, S> {
    pub fn wait(self) -> Option<R> {
        self.response.recv().ok()
    }
}

impl<R> Promise<R, Cancelable> {
    pub fn cancel(self) -> Option<R> {
        self.cancelable.0.cancel();
        self.response.recv().ok()
    }
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
}

impl<'t> JobBuilder<'t, Uncacelable> {
    pub fn submit<F, R>(self, f: F) -> Promise<R, Uncacelable>
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        let (tx, rx) = oneshot::channel();
        self.pool.submit(Job {
            priority: self.priority,
            cancelable: None,
            fun: Box::new(move |_stop_token| {
                let r = f();
                let _ = tx.send(r);
            }),
        });
        Promise {
            cancelable: self.cancelable,
            response: rx,
        }
    }
}

impl<'t> JobBuilder<'t, Cancelable> {
    pub fn submit<F, R>(self, f: F) -> Promise<ExecutionResult<R>, Cancelable>
    where
        F: FnOnce(StopToken) -> ExecutionResult<R> + Send + 'static,
        R: Send + 'static,
    {
        let (tx, rx) = oneshot::channel();
        self.pool.submit(Job {
            priority: self.priority,
            cancelable: Some(self.cancelable.0.clone()),
            fun: Box::new(move |stop_token| {
                let r = f(stop_token.expect("should always be some"));
                let _ = tx.send(r);
            }),
        });
        Promise {
            cancelable: self.cancelable,
            response: rx,
        }
    }
}

#[derive(Debug)]
pub struct ThreadPoolInner {
    queue: Mutex<BinaryHeap<Job>>,
    running: Box<[Mutex<stop_token::RunningCancelable>]>,
    has_jobs: Condvar,
    neet: Condvar, // no jobs
}

#[derive(Debug)]
pub struct ThreadPool {
    threads: Box<[thread::JoinHandle<()>]>,
    inner: Arc<ThreadPoolInner>,
}

impl Default for ThreadPool {
    fn default() -> Self {
        Self::new()
    }
}

fn start_worker_thread(id: usize, inner: Arc<ThreadPoolInner>) -> JoinHandle<()> {
    thread::spawn(move || {
        let running = &inner.running[id];
        eprintln!("[{:?}] starting", thread::current().id());
        while running.lock().is_not_canceled() {
            let job = {
                let mut queue = inner.queue.lock();
                if running.lock().is_canceled() {
                    return;
                }
                while queue.is_empty() {
                    eprintln!("[{:?}] falling asleep", thread::current().id());
                    inner.has_jobs.wait(&mut queue);
                    eprintln!("[{:?}] woke up", thread::current().id());
                    if running.lock().is_canceled() {
                        return;
                    }
                }
                let job = queue.pop().expect("queue can't possibly be empty");
                if queue.is_empty() {
                    inner.neet.notify_one();
                }
                let mut running = running.lock();
                if running.set(job.cancelable.clone()).is_err() {
                    return;
                }
                job
            };

            eprintln!("[{:?}] running job", thread::current().id());
            job.run();
            if running.lock().clear().is_err() {
                return;
            }
        }
    })
}

impl ThreadPool {
    pub fn new() -> Self {
        Self::new_with_size(available_parallelism().unwrap().get())
    }

    pub fn new_with_size(size: usize) -> Self {
        let inner = Arc::new(ThreadPoolInner {
            queue: Mutex::new(BinaryHeap::new()),
            running: (0..size)
                .map(|_| Mutex::new(stop_token::RunningCancelable::No))
                .collect(),
            has_jobs: Condvar::new(),
            neet: Condvar::new(),
        });

        let threads = (0..size)
            .map(|i| start_worker_thread(i, inner.clone()))
            .collect();

        Self { inner, threads }
    }

    pub fn new_job(&self) -> JobBuilder<Uncacelable> {
        JobBuilder {
            priority: Priority::default(),
            cancelable: Uncacelable,
            pool: self,
        }
    }

    fn submit(&self, job: Job) {
        self.inner.queue.lock().push(job);
        self.inner.has_jobs.notify_one();
    }

    pub fn stop_all(self) {
        for c in &self.inner.running {
            c.lock().cancel();
        }
        self.inner.has_jobs.notify_all();
        let mut queue = self.inner.queue.lock();
        queue.clear();
        Self::wait_impl(self.threads, &mut queue, &self.inner.neet)
    }

    pub fn wait(self) {
        Self::wait_impl(self.threads, &mut self.inner.queue.lock(), &self.inner.neet)
    }

    fn wait_impl(
        threads: Box<[JoinHandle<()>]>,
        queue: &mut MutexGuard<BinaryHeap<Job>>,
        neet: &Condvar,
    ) {
        neet.wait_while(queue, |q| !q.is_empty());
        for t in threads {
            let _ = t.join();
        }
    }
}

struct Job {
    priority: Priority,
    cancelable: Option<StopToken>,
    fun: Box<dyn FnOnce(Option<StopToken>) + Send>,
}

impl Job {
    fn run(mut self) {
        (self.fun)(self.cancelable.take())
    }
}

impl fmt::Debug for Job {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            priority,
            cancelable,
            fun: _,
        } = self;
        f.debug_struct("Job")
            .field("priority", priority)
            .field("cancelable", cancelable)
            .finish()
    }
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
