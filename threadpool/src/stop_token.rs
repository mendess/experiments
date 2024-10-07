use std::sync::{atomic::AtomicBool, Arc};

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

pub struct JointStopToken(pub StopToken, pub StopToken);

impl JointStopToken {
    pub fn should_cancel(&self) -> bool {
        self.0.should_cancel() || self.1.should_cancel()
    }

    pub fn should_continue(&self) -> bool {
        self.0.should_continue() && self.1.should_continue()
    }
}
