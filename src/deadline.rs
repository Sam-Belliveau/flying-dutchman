use std::time::Instant;

pub enum Deadline {
    None,
    Timeout(Instant),
}

impl Deadline {

    pub fn none() -> Deadline {
        Deadline::None
    }

    pub fn timeout(duration: std::time::Duration) -> Deadline {
        Deadline::Timeout(Instant::now() + duration)
    }

    pub fn passed(&self) -> bool {
        match self {
            Deadline::None => false,
            Deadline::Timeout(t) => Instant::now() >= *t,
        }
    }

}