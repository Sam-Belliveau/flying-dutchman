use chess::{get_king_moves, BitBoard, Board, BoardStatus, Color, MoveGen, Piece, EMPTY};

use super::{values, Score, MATE};

pub fn evaluate(board: &Board) -> Score {
    match board.status() {
        BoardStatus::Checkmate => -MATE,
        BoardStatus::Stalemate => MATE,
        BoardStatus::Ongoing => match board.side_to_move() {
            Color::White => values::piece(Piece::Pawn, values::TEMPO) + evaluate_for_white(board),
            Color::Black => values::piece(Piece::Pawn, values::TEMPO) - evaluate_for_white(board),
        },
    }
}

fn evaluate_for_white(board: &Board) -> Score {
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
            score += values::piece(piece, values::POSSES);
        }
    }

    for square in *board.color_combined(Color::Black) {
        if let Some(piece) = board.piece_on(square) {
            score -= values::piece(piece, values::POSSES)
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
                        score += values::piece(piece, values::ATTACK);
                        if near_king {
                            score += values::piece(piece, values::NEAR_KING);
                        }
                    }

                    None => {
                        score += values::piece(Piece::Pawn, values::HOLD);
                        if near_king {
                            score += values::piece(Piece::Pawn, values::NEAR_KING);
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
