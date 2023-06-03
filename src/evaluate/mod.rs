pub mod pesto;
pub mod score;
pub mod simpleval;

pub use self::score::*;

use chess::{Board, BoardStatus, Color};

const SIMPLE_EVAL: bool = false;
const TEMPO: Score = SCORE_BASE * 0;

pub fn evaluate(board: &Board) -> Score {
    match board.status() {
        BoardStatus::Checkmate => -MATE,
        BoardStatus::Stalemate => MATE,
        BoardStatus::Ongoing => match board.side_to_move() {
            Color::White => TEMPO + evaluate_for_white(board),
            Color::Black => TEMPO - evaluate_for_white(board),
        },
    }
}

fn evaluate_for_white(board: &Board) -> Score {
    let mut score = 0;

    if SIMPLE_EVAL {
        score += simpleval::evaluate(board);
    } else {
        score += pesto::evaluate(board);
    }

    score
}
