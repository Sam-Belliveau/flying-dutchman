use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    io::{BufRead, BufReader, Write},
    num::NonZeroUsize,
    process::{Child, ChildStdout, Command, Stdio},
    str::FromStr,
};

use chess::{Board, ChessMove};
use lru::LruCache;

use crate::search::deadline::Deadline;

const OPPONENT_CACHE_SIZE: usize = 1000 * 1000;

/// Code adapted from UCI
/// uci-0.2.3 uci::Engine

/// Custom error type for our opponent engine failures.
#[derive(Debug)]
pub enum OpponentEngineError {
    /// Failure to create or start the UCI engine process.
    EngineCreationError(String),
    /// Failure when sending commands to the engine or waiting for a response.
    EngineCommandError(String),
    /// Could not parse the move token from the engine's "bestmove" line.
    NoBestMoveFound,
    /// Parsing the returned best move into a `ChessMove` failed.
    InvalidMoveFormat(String),
    /// Cache size was invalid (e.g., zero).
    InvalidCacheSize,
    /// Deadline was exceeded.
    DeadlineExceeded,
}

impl Display for OpponentEngineError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            OpponentEngineError::EngineCreationError(e) => {
                write!(f, "Failed to create engine: {}", e)
            }
            OpponentEngineError::EngineCommandError(e) => {
                write!(f, "Engine command error: {}", e)
            }
            OpponentEngineError::NoBestMoveFound => {
                write!(f, "Could not parse best move from engine output line")
            }
            OpponentEngineError::InvalidMoveFormat(m) => {
                write!(f, "Engine returned invalid move format: '{}'", m)
            }
            OpponentEngineError::InvalidCacheSize => {
                write!(f, "Invalid cache size specified for LRU cache")
            }
            OpponentEngineError::DeadlineExceeded => {
                write!(f, "Deadline exceeded while waiting for engine response")
            }
        }
    }
}

impl Error for OpponentEngineError {}

pub struct OpponentEngine {
    /// A reference-counted, internally mutable child process.
    child: Child,
    /// A buffered reader for the child process's stdout.
    stdout_reader: BufReader<ChildStdout>,
    /// Cache of boards and their best moves.
    cached: LruCache<Board, ChessMove>,
}

impl OpponentEngine {
    /// Constructs a new `OpponentEngine`.
    /// Spawns the LC0 process, stores it in a RefCell, and initializes our LRU cache.
    pub fn new() -> Result<Self, OpponentEngineError> {
        // Prepare the child process with no spam to stdout/stderr.
        // - We read from stdout manually below.
        // - We redirect stderr to null, so it never appears.
        let mut command = Command::new("/opt/homebrew/bin/lc0");
        command
            .args(&["--weights=/Users/samb/Programming/Personal/flying-dutchman/maia-1100.pb.gz"]);
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null()); // No stderr spam

        let mut spawned_child = command
            .spawn()
            .map_err(|e| OpponentEngineError::EngineCreationError(e.to_string()))?;

        let stdout = spawned_child.stdout.take().ok_or_else(|| {
            OpponentEngineError::EngineCreationError("Engine stdout was unavailable".to_string())
        })?;

        // Validate cache size
        let cache_size =
            NonZeroUsize::new(OPPONENT_CACHE_SIZE).ok_or(OpponentEngineError::InvalidCacheSize)?;

        // Build our OpponentEngine
        let mut engine = OpponentEngine {
            child: spawned_child,
            stdout_reader: BufReader::new(stdout),
            cached: LruCache::new(cache_size),
        };

        // 1) Put the engine in UCI mode
        engine.send_command("uci\n")?;
        engine.read_until_prefix("uciok", &Deadline::none())?;

        // 2) Make sure it’s ready
        engine.send_command("isready\n")?;
        engine.read_until_prefix("readyok", &Deadline::none())?;

        Ok(engine)
    }

    /// Get a move from the engine, checking our cache first.
    pub fn get_move(
        &mut self,
        board: &Board,
        deadline: &Deadline,
    ) -> Result<ChessMove, OpponentEngineError> {
        // Check the cache
        if let Some(&movement) = self.cached.get(board) {
            return Ok(movement);
        }

        // Not in cache, get it from the engine
        let movement = self.uncached_get_move(board, deadline)?;

        // Store it
        self.cached.put(board.clone(), movement);

        Ok(movement)
    }

    /// Returns a move by querying the engine (no cache).
    fn uncached_get_move(
        &mut self,
        board: &Board,
        deadline: &Deadline,
    ) -> Result<ChessMove, OpponentEngineError> {
        // Send position command and then "go nodes 1"
        let command = format!("position fen {}\ngo nodes 1\n", board);
        let last_line = self.command_and_wait_for(&command, "bestmove", deadline)?;

        // Typically "bestmove e2e4"
        let move_str = last_line
            .trim()
            .split_whitespace()
            .nth(1) // skip "bestmove"
            .ok_or(OpponentEngineError::NoBestMoveFound)?;

        let movement = ChessMove::from_str(move_str)
            .map_err(|_| OpponentEngineError::InvalidMoveFormat(move_str.to_string()))?;

        if board.legal(movement) {
            Ok(movement)
        } else {
            Err(OpponentEngineError::InvalidMoveFormat(move_str.to_string()))
        }
    }

    // --- Internals below ---

    /// Write a command to the engine's stdin.
    fn send_command(&mut self, cmd: &str) -> Result<(), OpponentEngineError> {
        if let Some(stdin) = &mut self.child.stdin {
            stdin
                .write_all(cmd.as_bytes())
                .map_err(|e| OpponentEngineError::EngineCommandError(e.to_string()))?;
        }
        Ok(())
    }

    /// Write a command and read lines until we see a line starting with the `prefix`.
    fn command_and_wait_for(
        &mut self,
        cmd: &str,
        prefix: &str,
        deadline: &Deadline,
    ) -> Result<String, OpponentEngineError> {
        self.send_command(cmd)?;
        self.read_until_prefix(prefix, deadline)
    }

    /// Reads lines from engine stdout until a line starts with `prefix`.
    fn read_until_prefix(
        &mut self,
        prefix: &str,
        deadline: &Deadline,
    ) -> Result<String, OpponentEngineError> {
        loop {
            if deadline.passed() {
                return Err(OpponentEngineError::DeadlineExceeded);
            }

            let line = self.read_line(deadline)?;
            if line.starts_with(prefix) {
                return Ok(line);
            } else if line.starts_with("error") {
                return Err(OpponentEngineError::EngineCommandError(line));
            }
        }
    }

    /// Read a single line from engine stdout.
    fn read_line(&mut self, deadline: &Deadline) -> Result<String, OpponentEngineError> {
        let reader = &mut self.stdout_reader;
        let mut line = String::new();

        loop {
            if deadline.passed() {
                return Err(OpponentEngineError::DeadlineExceeded);
            }
            let bytes_read = reader
                .read_line(&mut line)
                .map_err(|e| OpponentEngineError::EngineCommandError(e.to_string()))?;
            if bytes_read == 0 {
                // End-of-stream
                break;
            }
            if line.ends_with('\n') {
                return Ok(line);
            }
        }
        Ok(line)
    }
}

// If you want to gracefully stop the engine on drop:
impl Drop for OpponentEngine {
    fn drop(&mut self) {
        let _ = self.send_command("quit\n"); // Tell engine to quit
        let _ = self.child.wait(); // Wait for child to exit
    }
}
