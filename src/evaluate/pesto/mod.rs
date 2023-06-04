pub mod gamephase;
pub mod phased_score;
pub mod psq_table;

use chess::{Board, Color::{White, Black}};

use self::{gamephase::GamePhase, phased_score::PhasedScore, psq_table::PieceSquareTable};

use super::{Score, SCORE_BASE};

pub fn evaluate(board: &Board) -> Score {
    let gamephase = GamePhase::new(board);
    let mut score = PhasedScore::new();

    for (piece, table) in PieceSquareTable::TABLES {
        for square in board.color_combined(White) & board.pieces(piece) {
            score += PhasedScore::from_piece(piece, White);
            score += table.from_square(square, White);
        }

        for square in board.color_combined(Black) & board.pieces(piece) {
            score += PhasedScore::from_piece(piece, Black);
            score += table.from_square(square, Black);
        }
    }

    (score * SCORE_BASE).collapse(gamephase)
}
