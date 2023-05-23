use std::{time::Instant, sync::atomic::{AtomicBool, Ordering}};

pub enum Deadline {
    None,
    Timeout(Instant),
    Async(AtomicBool),
}

impl Deadline {

    pub fn none() -> Deadline {
        Deadline::None
    }

    pub fn timeout(duration: std::time::Duration) -> Deadline {
        Deadline::Timeout(Instant::now() + duration)
    }

    pub fn asyncronous() -> Deadline {
        Deadline::Async(AtomicBool::new(false))
    }

    pub fn passed(&self) -> bool {
        match self {
            Deadline::None => false,
            Deadline::Timeout(t) => Instant::now() >= *t,
            Deadline::Async(pass) => pass.load(Ordering::Relaxed),
        }
    }

    pub fn trigger(&self) {
        match self {
            Deadline::None => {},
            Deadline::Timeout(_) => {},
            Deadline::Async(pass) => pass.store(true, Ordering::Relaxed),
        }
    }

    pub fn reset(&self) {
        match self {
            Deadline::None => {},
            Deadline::Timeout(_) => {},
            Deadline::Async(pass) => pass.store(false, Ordering::Relaxed),
        }
    }

}

unsafe impl Send for Deadline {}
unsafe impl Sync for Deadline {}