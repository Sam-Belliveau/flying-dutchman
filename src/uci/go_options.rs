use std::time::Duration;

use crate::search::Depth;
use chess::{Board, Piece};
use logos::Logos;

use crate::{
    search::deadline::Deadline,
    uci::tokens::UCIGoToken::{self, *},
};

const BUFFER: Duration = Duration::from_millis(250);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GoOptions {
    Infinite,
    ToDepth(Depth),

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
                Ok(Depth) => {
                    if let Some(Ok(Number(depth))) = lexer.next() {
                        return GoOptions::ToDepth(depth.try_into().unwrap_or(0));
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
            GoOptions::ToDepth(depth) => Deadline::depth(depth),
            GoOptions::MoveTime(time) => Deadline::timeout(time),
            GoOptions::TimeLimit {
                white_time,
                black_time,
                white_inc,
                black_inc,
            } => {
                let side = board.side_to_move();

                let (time, inc) = match side {
                    chess::Color::White => (white_time, white_inc),
                    chess::Color::Black => (black_time, black_inc),
                };

                let moves_left = (48
                    + board.pieces(Piece::Pawn).popcnt()
                    + 4 * board.pieces(Piece::Knight).popcnt()
                    + 4 * board.pieces(Piece::Bishop).popcnt()
                    + 8 * board.pieces(Piece::Rook).popcnt()
                    + 16 * board.pieces(Piece::Queen).popcnt())
                    / 4;

                let adjusted_inc = inc.checked_div(4).unwrap_or_default();

                let time_for_move = adjusted_inc
                    + time
                        .checked_div(moves_left as u32)
                        .unwrap_or_default()
                        .saturating_sub(BUFFER);

                Deadline::timeout(time_for_move)
            }
        }
    }
}
