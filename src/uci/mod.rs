use std::io::{self, Write};

pub mod go_options;
pub mod interpret;
pub mod thread;
pub mod tokens;

pub fn sync() {
    io::stdout().flush().unwrap();
}
