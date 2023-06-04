use chess::{get_king_moves, BitBoard, Board, Color, MoveGen, Piece, EMPTY};

use super::{Score, SCORE_BASE};

// Value of having a piece on the board
const POSSES: Score = SCORE_BASE;

// Value of attacking an enemy piece
const ATTACK: Score = SCORE_BASE / 3;

// Value of being able to move to a vacant square
const HOLD: Score = SCORE_BASE / 9;

// Value of being able to move to a vacant square
const NEAR_KING: Score = SCORE_BASE / 27;

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
    fn evaluate_moves_side(board: &Board) -> Score {
        let mut score = 0;

        let king_area = get_king_moves(board.king_square(!board.side_to_move()));
        for movement in MoveGen::new_legal(board) {
            let dest = movement.get_dest();
            let near_king = (king_area & BitBoard::from_square(dest)) != EMPTY;

            match board.piece_on(dest) {
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

        score
    }

    if let Some(null_move) = board.null_move() {
        let (white_board, black_board) = match board.side_to_move() {
            Color::White => (board, &null_move),
            Color::Black => (&null_move, board),
        };

        let white_score = evaluate_moves_side(white_board);
        let black_score = evaluate_moves_side(black_board);

        white_score - black_score
    } else {
        // This is typically unreachable due to QSearch
        // not running standpat on moves with check.
        0
    }
}
