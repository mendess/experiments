use std::{any::Any, panic};

use crate::Cancelable;

pub struct Promise<R, S> {
    pub(super) cancelable: S,
    pub(super) response: oneshot::Receiver<Result<R, Box<dyn Any + Send>>>,
}

impl<R, S> Promise<R, S> {
    pub fn wait(self) -> Option<R> {
        match self.response.recv().ok()? {
            Ok(r) => Some(r),
            Err(e) => panic::resume_unwind(e),
        }
    }
}

impl<R> Promise<R, Cancelable> {
    pub fn cancel(self) -> Option<R> {
        self.cancelable.0.cancel();
        self.wait()
    }
}
