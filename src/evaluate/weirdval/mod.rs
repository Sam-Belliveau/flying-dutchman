use once_cell::sync::Lazy;

use chess::Color::{Black, White};
use chess::{Board, Color};

use crate::evaluate::pesto::gamephase::GamePhase;
use crate::evaluate::pesto::phased_score::PhasedScore;
use crate::evaluate::pesto::psq_table::PieceSquareTable;

use super::{pesto, Score, CENTIPAWN};

const ENGINE_CENTIPAWN: Score = CENTIPAWN;
const OPPONENT_CENTIPAWN: Score = CENTIPAWN * 936 / 512;

pub fn unbalanced_evaluate_raw(board: &Board) -> Score {
    let gamephase = GamePhase::new(board);
    let mut white_score = PhasedScore::new();
    let mut black_score = PhasedScore::new();

    let white_mask = board.color_combined(White);
    let black_mask = board.color_combined(Black);

    for (piece, table) in PieceSquareTable::TABLES {
        let pieces = board.pieces(piece);
        let white_pieces = pieces & white_mask;
        let black_pieces = pieces & black_mask;

        white_score += PhasedScore::from_piece(piece, White) * white_pieces.popcnt() as Score;
        for square in white_pieces {
            white_score += table.get_square(square, White);
        }

        black_score -= PhasedScore::from_piece(piece, Black) * black_pieces.popcnt() as Score;
        for square in black_pieces {
            black_score -= table.get_square(square, Black);
        }
    }

    match board.side_to_move() {
        Color::White => {
            (white_score * ENGINE_CENTIPAWN).collapse(gamephase)
                - (black_score * OPPONENT_CENTIPAWN).collapse(gamephase)
        }
        Color::Black => {
            (black_score * ENGINE_CENTIPAWN).collapse(gamephase)
                - (white_score * OPPONENT_CENTIPAWN).collapse(gamephase)
        }
    }
}

pub fn evaluate_raw(board: &Board) -> Score {
    let mut white_score = 0;
    let mut black_score = 0;

    board.pieces(chess::Piece::Pawn).for_each(|square| {
        match board.color_on(square) {
            Some(Color::White) => white_score += 100 * CENTIPAWN,
            Some(Color::Black) => black_score += 100 * CENTIPAWN,
            None => (),
        }
    });

    let base = pesto::evaluate_for_white(board);

    match board.side_to_move() {
        Color::White => white_score - black_score + base,
        Color::Black => black_score - white_score - base,
    }
}

static DEFAULT_SCORE: Lazy<Score> = Lazy::new(|| evaluate_raw(&Board::default()));

pub fn evaluate(board: &Board) -> Score {
    evaluate_raw(board) - *DEFAULT_SCORE
}
