use std::io::{self, Write};

pub mod display;
pub mod go_options;
pub mod interpret;
pub mod thread;
pub mod tokens;

pub fn stdout_sync() {
    let _ = io::stdout().flush();
}
