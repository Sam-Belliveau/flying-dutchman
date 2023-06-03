use chess::{Piece, Board, Color, get_king_moves, MoveGen, BitBoard, EMPTY};

use super::{Score, SCORE_BASE};

// Value of having a piece on the board
const POSSES: Score = SCORE_BASE;

// Value of attacking an enemy piece
const ATTACK: Score = SCORE_BASE / 4;

// Value of being able to move to a vacant square
const HOLD: Score = SCORE_BASE / 10;

// Value of being able to move to a vacant square
const NEAR_KING: Score = SCORE_BASE / 40;

fn piece_value(piece: Piece, scale: Score) -> Score {
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

pub fn evaluate(board: &Board) -> Score {
    let mut score = 0;

    score += evaluate_pieces(board);
    score += evaluate_moves(board);

    score
}

// This code evaluates the pieces on the board and returns a score for the board.
fn evaluate_pieces(board: &Board) -> Score {
    let mut score = 0;

    for square in *board.color_combined(Color::White) {
        if let Some(piece) = board.piece_on(square) {
            score += piece_value(piece, POSSES);
        }
    }

    for square in *board.color_combined(Color::Black) {
        if let Some(piece) = board.piece_on(square) {
            score -= piece_value(piece, POSSES);
        }
    }

    score
}

fn evaluate_moves(board: &Board) -> Score {
    fn evaluate_moves_side(board: Option<&Board>) -> Score {
        let mut score = 0;

        if let Some(moves) = board {
            let king_area = get_king_moves(moves.king_square(!moves.side_to_move()));
            for movement in MoveGen::new_legal(moves) {
                let dest = movement.get_dest();
                let near_king = (king_area & BitBoard::from_square(dest)) != EMPTY;

                match moves.piece_on(dest) {
                    Some(piece) => {
                        score += piece_value(piece, ATTACK);
                        if near_king {
                            score += piece_value(piece, NEAR_KING);
                        }
                    }

                    None => {
                        score += piece_value(Piece::Pawn, HOLD);
                        if near_king {
                            score += piece_value(Piece::Pawn, NEAR_KING);
                        }
                    }
                }
            }
        }

        score
    }

    let null_move = board.null_move();
    let (white_board, black_board) = match board.side_to_move() {
        Color::White => (Some(board), null_move.as_ref()),
        Color::Black => (null_move.as_ref(), Some(board)),
    };

    let white_score = evaluate_moves_side(white_board);
    let black_score = evaluate_moves_side(black_board);

    white_score - black_score
}