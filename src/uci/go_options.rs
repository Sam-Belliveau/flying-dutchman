use std::time::Duration;

use chess::{Board, Piece};
use logos::Logos;

use crate::{
    search::deadline::Deadline,
    uci::tokens::UCIGoToken::{self, *},
};

const BUFFER: Duration = Duration::from_millis(100);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GoOptions {
    Infinite,

    MoveTime(Duration),

    TimeLimit {
        white_time: Duration,
        black_time: Duration,
        white_inc: Duration,
        black_inc: Duration,
    },
}

impl GoOptions {
    pub fn build(options: &str) -> GoOptions {
        let mut lexer = UCIGoToken::lexer(options);

        let mut white_time = None;
        let mut black_time = None;
        let mut white_inc = None;
        let mut black_inc = None;

        while let Some(token) = lexer.next() {
            match token {
                Ok(Infinite) => {
                    return GoOptions::Infinite;
                }
                Ok(MoveTime) => {
                    if let Some(Ok(Number(ms))) = lexer.next() {
                        let time = Duration::from_millis(ms);
                        return GoOptions::MoveTime(time);
                    }
                }
                Ok(WhiteTime) => {
                    if let Some(Ok(Number(ms))) = lexer.next() {
                        white_time = Some(Duration::from_millis(ms));
                    }
                }
                Ok(BlackTime) => {
                    if let Some(Ok(Number(ms))) = lexer.next() {
                        black_time = Some(Duration::from_millis(ms));
                    }
                }
                Ok(WhiteTimeInc) => {
                    if let Some(Ok(Number(ms))) = lexer.next() {
                        white_inc = Some(Duration::from_millis(ms));
                    }
                }
                Ok(BlackTimeInc) => {
                    if let Some(Ok(Number(ms))) = lexer.next() {
                        black_inc = Some(Duration::from_millis(ms));
                    }
                }
                Ok(_) => {}
                Err(()) => {}
            }
        }

        if white_time.is_some() || black_time.is_some() {
            GoOptions::TimeLimit {
                white_time: white_time.unwrap_or(Duration::from_millis(0)),
                black_time: black_time.unwrap_or(Duration::from_millis(0)),
                white_inc: white_inc.unwrap_or(Duration::from_millis(0)),
                black_inc: black_inc.unwrap_or(Duration::from_millis(0)),
            }
        } else {
            GoOptions::Infinite
        }
    }

    pub fn to_deadline(self, board: &Board) -> Deadline {
        match self {
            GoOptions::Infinite => Deadline::none(),
            GoOptions::MoveTime(time) => Deadline::timeout(time),
            GoOptions::TimeLimit {
                white_time,
                black_time,
                white_inc,
                black_inc,
            } => {
                let side = board.side_to_move();
                let moves_left = board.pieces(Piece::Pawn).popcnt()
                    + board.pieces(Piece::Knight).popcnt()
                    + board.pieces(Piece::Bishop).popcnt()
                    + 2 * board.pieces(Piece::Rook).popcnt()
                    + 4 * board.pieces(Piece::Queen).popcnt();

                match side {
                    chess::Color::White => Deadline::timeout(
                        white_time.max(Duration::from_secs(0)) / moves_left + white_inc - BUFFER,
                    ),
                    chess::Color::Black => Deadline::timeout(
                        black_time.max(Duration::from_secs(0)) / moves_left + black_inc - BUFFER,
                    ),
                }
            }
        }
    }
}
