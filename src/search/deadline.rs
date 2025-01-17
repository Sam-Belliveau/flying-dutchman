use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use super::Depth;

pub struct Deadline {
    deadline: Option<Instant>,
    trigger: AtomicBool,
    max_depth: Option<Depth>,
}

impl Deadline {
    pub fn none() -> Deadline {
        Deadline {
            deadline: None,
            trigger: AtomicBool::new(false),
            max_depth: None,
        }
    }

    pub fn depth(depth: Depth) -> Deadline {
        Deadline {
            deadline: None,
            trigger: AtomicBool::new(false),
            max_depth: Some(depth),
        }
    }

    pub fn timeout(duration: std::time::Duration) -> Deadline {
        Deadline {
            deadline: Some(Instant::now() + duration),
            trigger: AtomicBool::new(false),
            max_depth: None,
        }
    }

    pub fn check_depth(&self, depth: Depth) -> bool {
        if let Some(max_depth) = self.max_depth {
            depth < max_depth
        } else {
            true
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
