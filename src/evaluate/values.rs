use chess::Piece;

use super::{Score, SCORE_BASE};

// Value of having a piece on the board
pub const POSSES: Score = SCORE_BASE;

// Value of attacking an enemy piece
pub const ATTACK: Score = SCORE_BASE / 3;

// Value of being able to move to a vacant square
pub const HOLD: Score = SCORE_BASE / 9;

// Value of being able to move to a vacant square
pub const NEAR_KING: Score = SCORE_BASE / 36;

// Value of being the side evaluated, helps with tempo
pub const TEMPO: Score = 0;

pub fn piece(piece: Piece, scale: Score) -> Score {
    scale
        * match piece {
            Piece::Pawn => 100,
            Piece::Knight => 320,
            Piece::Bishop => 330,
            Piece::Rook => 500,
            Piece::Queen => 900,
            Piece::King => 20000,
        }
}
