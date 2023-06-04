use chess::{get_king_moves, BitBoard, Board, Color, MoveGen, Piece, EMPTY};

use crate::evaluate::pesto::{phased_score::PhasedScore, gamephase::GamePhase};

use super::{Score, SCORE_BASE, pesto};

// Value of attacking an enemy piece
const ATTACK: Score = SCORE_BASE / 5;

// Value of being able to move to a vacant square
const HOLD: Score = SCORE_BASE / 10;

// Value of being able to move to a vacant square
const NEAR_KING: Score = SCORE_BASE / 40;

pub fn evaluate(board: &Board) -> Score {
    let mut score = 0;

    score += pesto::evaluate(board);
    score += evaluate_moves(board);

    score
}

fn evaluate_moves(board: &Board) -> Score {
    fn evaluate_moves_side(board: &Board) -> PhasedScore {
        let mut score = PhasedScore::new();
        let sidemove = board.side_to_move();
        let opponent = !sidemove;
        let king_area = get_king_moves(board.king_square(opponent));
        for movement in MoveGen::new_legal(board) {
            let dest = movement.get_dest();
            let near_king = (king_area & BitBoard::from_square(dest)) != EMPTY;

            match board.piece_on(dest) {
                Some(piece) => {
                    score += PhasedScore::from_piece(piece, sidemove) * ATTACK;
                    if near_king {
                        score += PhasedScore::from_piece(piece, sidemove) * NEAR_KING;
                    }
                }

                None => {
                    score += PhasedScore::from_piece(Piece::Pawn, sidemove) * HOLD;
                    if near_king {
                        score += PhasedScore::from_piece(Piece::Pawn, sidemove) * NEAR_KING;
                    }
                }
            }
        }

        score
    }

    if let Some(null_move) = board.null_move() {
        let (white_board, black_board) = match board.side_to_move() {
            Color::White => (board, &null_move),
            Color::Black => (&null_move, board),
        };

        let mut score = PhasedScore::new();

        score += evaluate_moves_side(white_board);
        score += evaluate_moves_side(black_board);

        score.collapse(GamePhase::new(board))
    } else {
        // This is typically unreachable due to QSearch
        // not running standpat on moves with check.
        0
    }
}
