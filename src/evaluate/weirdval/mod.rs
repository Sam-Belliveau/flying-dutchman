use chess::{Board, Color, Piece};

use crate::evaluate::{Score, CENTIPAWN};

const PAWN_VALUE: Score = 1000 * CENTIPAWN;
const KNIGHT_VALUE: Score = 3 * CENTIPAWN;
const BISHOP_VALUE: Score = 3 * CENTIPAWN;
const ROOK_VALUE: Score = 5 * CENTIPAWN;
const QUEEN_VALUE: Score = 9 * CENTIPAWN;

pub fn evaluate_for_white(board: &Board) -> Score {
    let mut score = 0;

    for square in *board.color_combined(Color::White) {
        score += match board.piece_on(square) {
            Some(Piece::Pawn) => PAWN_VALUE,
            Some(Piece::Knight) => KNIGHT_VALUE,
            Some(Piece::Bishop) => BISHOP_VALUE,
            Some(Piece::Rook) => ROOK_VALUE,
            Some(Piece::Queen) => QUEEN_VALUE,
            Some(Piece::King) => 0,
            None => 0,
        };
    }

    for square in *board.color_combined(Color::Black) {
        score -= match board.piece_on(square) {
            Some(Piece::Pawn) => PAWN_VALUE,
            Some(Piece::Knight) => KNIGHT_VALUE,
            Some(Piece::Bishop) => BISHOP_VALUE,
            Some(Piece::Rook) => ROOK_VALUE,
            Some(Piece::Queen) => QUEEN_VALUE,
            Some(Piece::King) => 0,
            None => 0,
        };
    }

    score
}
