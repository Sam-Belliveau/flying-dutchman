use chess::{Board, Color, Piece::*};

use self::{gamephase::GamePhase, phased_score::PhasedScore, psq_table::PieceSquareTable};

use super::Score;

pub mod gamephase;
pub mod phased_score;
pub mod psq_table;

pub fn evaluate(board: &Board) -> Score {
    let gamephase = GamePhase::new(board);
    let mut score = PhasedScore::new();

    for piece in &[Pawn, Knight, Bishop, Rook, Queen, King] {
        let table = PieceSquareTable::from_piece(*piece);

        for color in &[Color::White, Color::Black] {
            let mask = board.color_combined(*color);

            for square in mask & board.pieces(*piece) {
                score += PhasedScore::from_piece(*piece, *color);
                score += table.from_square(square, *color);
            }
        }
    }

    score.collapse(gamephase)
}
