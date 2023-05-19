use crate::types::{Score, MATE, SCORE_BASE};
use chess::{get_king_moves, BitBoard, Board, BoardStatus, Color, MoveGen, Piece, EMPTY};

// Value of having a piece on the board
const POSSES_VALUE: Score = SCORE_BASE;

// Value of attacking an enemy piece
const ATTACK_VALUE: Score = SCORE_BASE / 4;

// Value of being able to move to a vacant square
const HOLD_VALUE: Score = SCORE_BASE / 10;

// Value of being able to move to a vacant square
const KING_HOLD_VALUE: Score = SCORE_BASE / 40;

// Value of being the side evaluated, helps with tempo
const TEMPO_BONUS: Score = 1 * SCORE_BASE;

pub fn piece_value(piece: Piece, scale: Score) -> Score {
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
    piece_value(Piece::Pawn, TEMPO_BONUS)
        + match board.side_to_move() {
            Color::White => evaluate_for_white(board),
            Color::Black => -evaluate_for_white(board),
        }
}

pub fn evaluate_for_white(board: &Board) -> Score {
    match board.status() {
        BoardStatus::Checkmate => match board.side_to_move() {
            Color::White => -MATE,
            Color::Black => MATE,
        },
        BoardStatus::Stalemate => -MATE,
        BoardStatus::Ongoing => evaluate_ongoing(board),
    }
}

fn evaluate_ongoing(board: &Board) -> Score {
    let mut score = 0;

    score += evaluate_pieces(board);
    score += evaluate_moves(board);

    score
}

fn evaluate_pieces(board: &Board) -> Score {
    let mut score = 0;

    for square in *board.color_combined(Color::White) {
        if let Some(piece) = board.piece_on(square) {
            score += piece_value(piece, POSSES_VALUE);
        }
    }

    for square in *board.color_combined(Color::Black) {
        if let Some(piece) = board.piece_on(square) {
            score -= piece_value(piece, POSSES_VALUE)
        }
    }

    score
}

fn split_board(board: Board) -> (Option<Board>, Option<Board>) {
    match board.side_to_move() {
        Color::White => (Some(board), board.null_move()),
        Color::Black => (board.null_move(), Some(board)),
    }
}

fn evaluate_moves(board: &Board) -> Score {
    fn evaluate_moves_side(board: Option<Board>) -> Score {
        let mut score = 0;

        if let Some(moves) = board {
            let targets = get_king_moves(moves.king_square(!moves.side_to_move()));
            for movement in MoveGen::new_legal(&moves) {
                let dest = movement.get_dest();

                let king = targets & BitBoard::from_square(dest) != EMPTY;
                match moves.piece_on(dest) {
                    Some(piece) => {
                        score += piece_value(piece, ATTACK_VALUE);
                        if king {
                            score += piece_value(piece, KING_HOLD_VALUE);
                        }
                    }

                    None => {
                        score += piece_value(Piece::Pawn, HOLD_VALUE);
                        if king {
                            score += piece_value(Piece::Pawn, KING_HOLD_VALUE);
                        }
                    }
                }
            }
        }

        score
    }

    let (white_board, black_board) = split_board(board.clone());

    let white_score = evaluate_moves_side(white_board);
    let black_score = evaluate_moves_side(black_board);

    white_score - black_score
}
