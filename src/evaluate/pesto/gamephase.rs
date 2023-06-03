use chess::{Board, Piece};

use crate::evaluate::Score;

pub struct GamePhase {
    mid_weight: Score,
    end_weight: Score,
}

impl GamePhase {
    pub fn new(board: &Board) -> Self {
        let mid_weight = (
            // Sum pieces to get phase
            0 * board.pieces(Piece::Pawn).popcnt()
                + 1 * board.pieces(Piece::Knight).popcnt()
                + 1 * board.pieces(Piece::Bishop).popcnt()
                + 2 * board.pieces(Piece::Rook).popcnt()
                + 4 * board.pieces(Piece::Queen).popcnt()
        )
        .clamp(0, 24);

        let end_weight = 24 - mid_weight;

        Self {
            mid_weight: mid_weight as Score,
            end_weight: end_weight as Score,
        }
    }

    pub fn weight(&self, mid_game: Score, end_game: Score) -> Score {
        let numerator = mid_game * self.mid_weight + end_game * self.end_weight;
        numerator / (self.mid_weight + self.end_weight)
    }
}
