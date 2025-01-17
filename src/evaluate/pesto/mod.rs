pub mod gamephase;
pub mod phased_score;
pub mod psq_table;

use chess::Board;
use chess::Color::{Black, White};

use crate::evaluate::pesto::gamephase::GamePhase;
use crate::evaluate::pesto::phased_score::PhasedScore;
use crate::evaluate::pesto::psq_table::PieceSquareTable;

use super::{Score, CENTIPAWN};

pub fn evaluate(board: &Board) -> Score {
    let gamephase = GamePhase::new(board);
    let mut score = PhasedScore::new();

    let white_mask = board.color_combined(White);
    let black_mask = board.color_combined(Black);

    for (piece, table) in PieceSquareTable::TABLES {
        let pieces = board.pieces(piece);
        let white_pieces = pieces & white_mask;
        let black_pieces = pieces & black_mask;

        score += PhasedScore::from_piece(piece, White) * white_pieces.popcnt() as Score;
        for square in white_pieces {
            score += table.get_square(square, White);
        }

        score += PhasedScore::from_piece(piece, Black) * black_pieces.popcnt() as Score;
        for square in black_pieces {
            score += table.get_square(square, Black);
        }
    }

    (score * CENTIPAWN).collapse(gamephase)
}
