use std::{
    cell::{OnceCell, RefCell},
    sync::{atomic::AtomicBool, Arc},
};

#[derive(Default, Clone, Debug)]
pub struct StopToken {
    flag: Arc<AtomicBool>,
}

impl StopToken {
    pub(super) fn cancel(&self) {
        self.flag.store(true, std::sync::atomic::Ordering::Release);
    }

    pub fn should_cancel(&self) -> bool {
        self.flag.load(std::sync::atomic::Ordering::Acquire)
    }

    pub fn should_continue(&self) -> bool {
        !self.should_cancel()
    }
}

thread_local! {
    static POOL_STOP_TOKEN: OnceCell<StopToken> = const { OnceCell::new() };
    static JOB_STOP_TOKEN: RefCell<Option<StopToken>> = const { RefCell::new(None) };
}

pub(super) fn init_worker_token(token: StopToken) {
    POOL_STOP_TOKEN.with(|t| {
        t.get_or_init(move || token);
    });
}

pub(super) fn worker_should_cancel() -> bool {
    POOL_STOP_TOKEN.with(|pool_token| pool_token.get().unwrap().should_cancel())
}

pub(super) fn set_job_token(token: StopToken) -> impl Drop {
    JOB_STOP_TOKEN.with_borrow_mut(|t| *t = Some(token));
    struct ClearGuard;
    impl Drop for ClearGuard {
        fn drop(&mut self) {
            JOB_STOP_TOKEN.with_borrow_mut(|t| *t = None)
        }
    }
    ClearGuard
}

pub fn job_should_continue() -> bool {
    !job_should_cancel()
}

pub fn job_should_cancel() -> bool {
    POOL_STOP_TOKEN.with(|pool_token| {
        if pool_token.get().unwrap().should_cancel() {
            return true;
        }
        JOB_STOP_TOKEN.with_borrow(|job_token| {
            if let Some(token) = job_token {
                if token.should_cancel() {
                    return true;
                }
            }
            false
        })
    })
}
