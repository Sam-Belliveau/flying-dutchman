pub mod pesto;
pub mod score;

pub use self::score::*;

use chess::{Board, BoardStatus, Color};

pub fn evaluate(board: &Board) -> Score {
    match board.status() {
        BoardStatus::Checkmate => -MATE,
        BoardStatus::Stalemate => MATE,
        BoardStatus::Ongoing => match board.side_to_move() {
            Color::White => 0 + evaluate_for_white(board),
            Color::Black => 0 - evaluate_for_white(board),
        },
    }
}

fn evaluate_for_white(board: &Board) -> Score {
    let mut score = 0;

    score += SCORE_BASE * pesto::evaluate(board);

    score
}
