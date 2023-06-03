use chess::{Color, Square, Piece};

use crate::evaluate::Score;

use super::phased_score::{PhasedScore, RawPhasedScore};

#[derive(Clone, Debug)]
pub struct PieceSquareTable {
    mg: [Score; 64],
    eg: [Score; 64],
}

impl PieceSquareTable {
    const fn new(mg: [Score; 64], eg: [Score; 64]) -> Self {
        Self { mg, eg }
    }

    fn from_index(&self, idx: usize) -> RawPhasedScore {
        RawPhasedScore::new(self.mg[idx], self.eg[idx])
    }

    pub fn from_piece(piece: Piece) -> &'static PieceSquareTable {
        match piece {
            Piece::Pawn => &PAWN_TABLE,
            Piece::Knight => &KNIGHT_TABLE,
            Piece::Bishop => &BISHOP_TABLE,
            Piece::Rook => &ROOK_TABLE,
            Piece::Queen => &QUEEN_TABLE,
            Piece::King => &KING_TABLE,
        }
    }

    pub fn from_square(&self, square: Square, color: Color) -> PhasedScore {
        let index = match color {
            Color::White => 0b111000 ^ square.to_index(),
            Color::Black => 0b000000 ^ square.to_index(),
        };

        self.from_index(index).colorize(color)
    }
}

/* piece/sq tables */
/* values from Rofchade: http://www.talkchess.com/forum3/viewtopic.php?f=2&t=68311&start=19 */

const PAWN_TABLE: PieceSquareTable = PieceSquareTable::new(
    [
        0, 0, 0, 0, 0, 0, 0, 0, // Rank 1 or 8
        98, 134, 61, 95, 68, 126, 34, -11, // Rank 2 or 7
        -6, 7, 26, 31, 65, 56, 25, -20, // Rank 3 or 6
        -14, 13, 6, 21, 23, 12, 17, -23, // Rank 4 or 5
        -27, -2, -5, 12, 17, 6, 10, -25, // Rank 5 or 4
        -26, -4, -4, -10, 3, 3, 33, -12, // Rank 6 or 3
        -35, -1, -20, -23, -15, 24, 38, -22, // Rank 7 or 2
        0, 0, 0, 0, 0, 0, 0, 0, // Rank 8 or 1
    ],
    [
        0, 0, 0, 0, 0, 0, 0, 0, // Rank 1 or 8
        178, 173, 158, 134, 147, 132, 165, 187, // Rank 2 or 7
        94, 100, 85, 67, 56, 53, 82, 84, // Rank 3 or 6
        32, 24, 13, 5, -2, 4, 17, 17, // Rank 4 or 5
        13, 9, -3, -7, -7, -8, 3, -1, // Rank 5 or 4
        4, 7, -6, 1, 0, -5, -1, -8, // Rank 6 or 3
        13, 8, 8, 10, 13, 0, 2, -7, // Rank 7 or 2
        0, 0, 0, 0, 0, 0, 0, 0, // Rank 8 or 1
    ],
);

const KNIGHT_TABLE: PieceSquareTable = PieceSquareTable::new(
    [
        -167, -89, -34, -49, 61, -97, -15, -107, // Rank 1 or 8
        -73, -41, 72, 36, 23, 62, 7, -17, // Rank 2 or 7
        -47, 60, 37, 65, 84, 129, 73, 44, // Rank 3 or 6
        -9, 17, 19, 53, 37, 69, 18, 22, // Rank 4 or 5
        -13, 4, 16, 13, 28, 19, 21, -8, // Rank 5 or 4
        -23, -9, 12, 10, 19, 17, 25, -16, // Rank 6 or 3
        -29, -53, -12, -3, -1, 18, -14, -19, // Rank 7 or 2
        -105, -21, -58, -33, -17, -28, -19, -23, // Rank 8 or 1
    ],
    [
        -58, -38, -13, -28, -31, -27, -63, -99, // Rank 1 or 8
        -25, -8, -25, -2, -9, -25, -24, -52, // Rank 2 or 7
        -24, -20, 10, 9, -1, -9, -19, -41, // Rank 3 or 6
        -17, 3, 22, 22, 22, 11, 8, -18, // Rank 4 or 5
        -18, -6, 16, 25, 16, 17, 4, -18, // Rank 5 or 4
        -23, -3, -1, 15, 10, -3, -20, -22, // Rank 6 or 3
        -42, -20, -10, -5, -2, -20, -23, -44, // Rank 7 or 2
        -29, -51, -23, -15, -22, -18, -50, -64, // Rank 8 or 1
    ],
);

const BISHOP_TABLE: PieceSquareTable = PieceSquareTable::new(
    [
        -29, 4, -82, -37, -25, -42, 7, -8, // Rank 1 or 8
        -26, 16, -18, -13, 30, 59, 18, -47, // Rank 2 or 7
        -16, 37, 43, 40, 35, 50, 37, -2, // Rank 3 or 6
        -4, 5, 19, 50, 37, 37, 7, -2, // Rank 4 or 5
        -6, 13, 13, 26, 34, 12, 10, 4, // Rank 5 or 4
        0, 15, 15, 15, 14, 27, 18, 10, // Rank 6 or 3
        4, 15, 16, 0, 7, 21, 33, 1, // Rank 7 or 2
        -33, -3, -14, -21, -13, -12, -39, -21, // Rank 8 or 1
    ],
    [
        -14, -21, -11, -8, -7, -9, -17, -24, // Rank 1 or 8
        -8, -4, 7, -12, -3, -13, -4, -14, // Rank 2 or 7
        2, -8, 0, -1, -2, 6, 0, 4, // Rank 3 or 6
        -3, 9, 12, 9, 14, 10, 3, 2, // Rank 4 or 5
        -6, 3, 13, 19, 7, 10, -3, -9, // Rank 5 or 4
        -12, -3, 8, 10, 13, 3, -7, -15, // Rank 6 or 3
        -14, -18, -7, -1, 4, -9, -15, -27, // Rank 7 or 2
        -23, -9, -23, -5, -9, -16, -5, -17, // Rank 8 or 1
    ],
);
const ROOK_TABLE: PieceSquareTable = PieceSquareTable::new(
    [
        32, 42, 32, 51, 63, 9, 31, 43, // Rank 1 or 8
        27, 32, 58, 62, 80, 67, 26, 44, // Rank 2 or 7
        -5, 19, 26, 36, 17, 45, 61, 16, // Rank 3 or 6
        -24, -11, 7, 26, 24, 35, -8, -20, // Rank 4 or 5
        -36, -26, -12, -1, 9, -7, 6, -23, // Rank 5 or 4
        -45, -25, -16, -17, 3, 0, -5, -33, // Rank 6 or 3
        -44, -16, -20, -9, -1, 11, -6, -71, // Rank 7 or 2
        -19, -13, 1, 17, 16, 7, -37, -26, // Rank 8 or 1
    ],
    [
        13, 10, 18, 15, 12, 12, 8, 5, // Rank 1 or 8
        11, 13, 13, 11, -3, 3, 8, 3, // Rank 2 or 7
        7, 7, 7, 5, 4, -3, -5, -3, // Rank 3 or 6
        4, 3, 13, 1, 2, 1, -1, 2, // Rank 4 or 5
        3, 5, 8, 4, -5, -6, -8, -11, // Rank 5 or 4
        -4, 0, -5, -1, -7, -12, -8, -16, // Rank 6 or 3
        -6, -6, 0, 2, -9, -9, -11, -3, // Rank 7 or 2
        -9, 2, 3, -1, -5, -13, 4, -20, // Rank 8 or 1
    ],
);

const QUEEN_TABLE: PieceSquareTable = PieceSquareTable::new(
    [
        -28, 0, 29, 12, 59, 44, 43, 45, // Rank 1 or 8
        -24, -39, -5, 1, -16, 57, 28, 54, // Rank 2 or 7
        -13, -17, 7, 8, 29, 56, 47, 57, // Rank 3 or 6
        -27, -27, -16, -16, -1, 17, -2, 1, // Rank 4 or 5
        -9, -26, -9, -10, -2, -4, 3, -3, // Rank 5 or 4
        -14, 2, -11, -2, -5, 2, 14, 5, // Rank 6 or 3
        -35, -8, 11, 2, 8, 15, -3, 1, // Rank 7 or 2
        -1, -18, -9, 10, -15, -25, -31, -50, // Rank 8 or 1
    ],
    [
        -9, 22, 22, 27, 27, 19, 10, 20, // Rank 1 or 8
        -17, 20, 32, 41, 58, 25, 30, 0, // Rank 2 or 7
        -20, 6, 9, 49, 47, 35, 19, 9, // Rank 3 or 6
        3, 22, 24, 45, 57, 40, 57, 36, // Rank 4 or 5
        -18, 28, 19, 47, 31, 34, 39, 23, // Rank 5 or 4
        -16, -27, 15, 6, 9, 17, 10, 5, // Rank 6 or 3
        -22, -23, -30, -16, -16, -23, -36, -32, // Rank 7 or 2
        -33, -28, -22, -43, -5, -32, -20, -41, // Rank 8 or 1
    ],
);

const KING_TABLE: PieceSquareTable = PieceSquareTable::new(
    [
        -65, 23, 16, -15, -56, -34, 2, 13, // Rank 1 or 8
        29, -1, -20, -7, -8, -4, -38, -29, // Rank 2 or 7
        -9, 24, 2, -16, -20, 6, 22, -22, // Rank 3 or 6
        -17, -20, -12, -27, -30, -25, -14, -36, // Rank 4 or 5
        -49, -1, -27, -39, -46, -44, -33, -51, // Rank 5 or 4
        -14, -14, -22, -46, -44, -30, -15, -27, // Rank 6 or 3
        1, 7, -8, -64, -43, -16, 9, 8, // Rank 7 or 2
        -15, 36, 12, -54, 8, -28, 24, 14, // Rank 8 or 1
    ],
    [
        -74, -35, -18, -18, -11, 15, 4, -17, // Rank 1 or 8
        -12, 17, 14, 17, 17, 38, 23, 11, // Rank 2 or 7
        10, 17, 23, 15, 20, 45, 44, 13, // Rank 3 or 6
        -8, 22, 24, 27, 26, 33, 26, 3, // Rank 4 or 5
        -18, -4, 21, 24, 27, 23, 9, -11, // Rank 5 or 4
        -19, -3, 11, 21, 23, 16, 7, -9, // Rank 6 or 3
        -27, -11, 4, 13, 14, 4, -5, -17, // Rank 7 or 2
        -53, -34, -21, -11, -28, -14, -24, -43, // Rank 8 or 1
    ],
);
