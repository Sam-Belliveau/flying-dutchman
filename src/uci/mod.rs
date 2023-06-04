use std::io::{self, Write};

pub mod go_options;
pub mod interpret;
pub mod thread;
pub mod tokens;
pub mod display;

pub fn stdout_sync() {
    io::stdout().flush().unwrap();
}
