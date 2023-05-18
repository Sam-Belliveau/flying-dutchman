use crate::types::{Score, DRAW, MATE, SCORE_BASE};
use chess::{Board, BoardStatus, Color, MoveGen, Piece, get_rook_moves, get_bishop_moves};

// Value of having a piece on the board
const POSSES_VALUE: Score = SCORE_BASE;

// The cost of a piece when it is pinned
const PINNED_PENALTY: Score = -SCORE_BASE;

// Value of attacking an enemy piece
const ATTACK_VALUE: Score = SCORE_BASE / 4;

// Value of being able to move to a vacant square
const HOLD_VALUE: Score = SCORE_BASE / 8;

// Value of having a square be visible to the king
const KING_VIS_PENALTY: Score = -SCORE_BASE / 4;

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
    match board.side_to_move() {
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
        BoardStatus::Stalemate => DRAW,
        BoardStatus::Ongoing => evaluate_ongoing(board),
    }
}

fn evaluate_ongoing(board: &Board) -> Score {
    let mut score = 0;

    score += evaluate_pieces(board);
    score += evaluate_moves(board);
    score += evaluate_pins(board);
    score += evaluate_king_vis(board);

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
            for movement in MoveGen::new_legal(&moves) {
                let dest = movement.get_dest();

                match moves.piece_on(dest) {
                    Some(piece) => {
                        score += piece_value(piece, ATTACK_VALUE);
                    }

                    None => {
                        score += piece_value(Piece::Pawn, HOLD_VALUE);
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

pub fn evaluate_pins(board: &Board) -> Score {
    fn evaluate_pins_side(board_opt: Option<Board>) -> Score {
        let mut score = 0;

        if let Some(board) = board_opt {
            for square in *board.pinned() {
                if let Some(piece) = board.piece_on(square) {
                    match board.color_on(square) {
                        Some(Color::White) => score += piece_value(piece, PINNED_PENALTY),
                        Some(Color::Black) => score -= piece_value(piece, PINNED_PENALTY),
                        None => {}
                    }
                }
            }
        }

        score
    }
    let (white_board, black_board) = split_board(board.clone());

    evaluate_pins_side(white_board) + evaluate_pins_side(black_board)
}

pub fn evaluate_king_vis(board: &Board) -> Score {
    let white_king = board.king_square(Color::White);
    let black_king = board.king_square(Color::Black);

    let pieces = *board.combined();
    let white_king = get_rook_moves(white_king, pieces) | get_bishop_moves(white_king, pieces);
    let black_king = get_rook_moves(black_king, pieces) | get_bishop_moves(black_king, pieces);

    let mut score = 0;
    score += white_king.popcnt() as Score * piece_value(Piece::Pawn, KING_VIS_PENALTY);
    score -= black_king.popcnt() as Score * piece_value(Piece::Pawn, KING_VIS_PENALTY);

    score
}
