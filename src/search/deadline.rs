use std::{
    sync::atomic::{AtomicBool, Ordering},
    time::Instant,
};

pub struct Deadline {
    deadline: Option<Instant>,
    trigger: AtomicBool,
}

impl Deadline {
    pub fn none() -> Deadline {
        Deadline {
            deadline: None,
            trigger: AtomicBool::new(false),
        }
    }

    pub fn timeout(duration: std::time::Duration) -> Deadline {
        Deadline {
            deadline: Some(Instant::now() + duration),
            trigger: AtomicBool::new(false),
        }
    }

    pub fn passed(&self) -> bool {
        if let Some(deadline) = self.deadline {
            if Instant::now() >= deadline {
                return true;
            }
        }

        self.trigger.load(Ordering::Relaxed)
    }

    pub fn trigger(&self) {
        self.trigger.store(true, Ordering::Relaxed)
    }
}

unsafe impl Send for Deadline {}
unsafe impl Sync for Deadline {}
