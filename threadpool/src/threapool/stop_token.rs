use std::{
    sync::{
        atomic::{AtomicBool, AtomicUsize},
        Arc,
    },
    thread,
};

#[derive(Clone, Debug)]
pub struct StopToken {
    id: usize,
    flag: Arc<AtomicBool>,
}

impl Default for StopToken {
    fn default() -> Self {
        static ID: AtomicUsize = AtomicUsize::new(0);

        Self {
            id: ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst),
            flag: Default::default(),
        }
    }
}

impl StopToken {
    pub(super) fn cancel(self) {
        eprintln!(
            "[{:?}][token: {}] canceling",
            thread::current().id(),
            self.id
        );
        self.flag.store(true, std::sync::atomic::Ordering::Release);
    }

    pub fn should_cancel(&self) -> bool {
        let flag = self.flag.load(std::sync::atomic::Ordering::Acquire);
        eprintln!(
            "[{:?}][token: {}] should_cancel = {flag}",
            thread::current().id(),
            self.id
        );
        flag
    }

    pub fn should_continue(&self) -> bool {
        let flag = !self.flag.load(std::sync::atomic::Ordering::Acquire);
        eprintln!(
            "[{:?}][token: {}] should_continue = {flag}",
            thread::current().id(),
            self.id
        );
        flag
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub enum ExecutionResult<T> {
    Canceled,
    Returned(T),
}

#[derive(Debug)]
pub(super) enum RunningCancelable {
    Yes(StopToken),
    No,
    Canceled,
}

impl RunningCancelable {
    pub fn is_canceled(&self) -> bool {
        matches!(self, Self::Canceled)
    }

    pub fn is_not_canceled(&self) -> bool {
        !matches!(self, Self::Canceled)
    }

    pub fn set(&mut self, token: Option<StopToken>) -> Result<(), ()> {
        match self {
            RunningCancelable::Canceled => Err(()),
            _ => {
                *self = match token {
                    Some(token) => Self::Yes(token),
                    None => Self::No,
                };
                Ok(())
            }
        }
    }

    pub fn clear(&mut self) -> Result<(), ()> {
        match self {
            RunningCancelable::Canceled => Err(()),
            _ => {
                *self = RunningCancelable::No;
                Ok(())
            }
        }
    }

    pub fn cancel(&mut self) {
        if let Self::Yes(token) = std::mem::replace(self, Self::Canceled) {
            token.cancel();
        }
    }
}
