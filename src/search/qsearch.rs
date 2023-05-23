use chess::{get_bishop_moves, get_knight_moves, get_rook_moves, Board, MoveGen, EMPTY};

use crate::evaluate::{evaluate, Score};

use super::alpha_beta::{AlphaBeta, NegaMaxResult::*};

fn alpha_beta_search(board: Board, mut window: AlphaBeta) -> Score {
    let eval = evaluate(&board);

    if let BetaPrune { beta } = window.negamax(eval) {
        return beta;
    }

    let pieces = *board.combined();
    let king = board.king_square(!board.side_to_move());
    let checkers =
        get_bishop_moves(king, pieces) | get_rook_moves(king, pieces) | get_knight_moves(king);

    let t1 = (true, *board.pieces(chess::Piece::Queen));
    let t2 = (true, *board.pieces(chess::Piece::Rook));
    let t3 = (true, *board.pieces(chess::Piece::Bishop));
    let t4 = (true, *board.pieces(chess::Piece::Knight));
    let t5 = (true, *board.pieces(chess::Piece::Pawn));
    let t6 = (false, checkers);

    let mut moves = MoveGen::new_legal(&board);
    for (good, mask) in [t1, t2, t3, t4, t5, t6] {
        moves.set_iterator_mask(mask);

        for movement in &mut moves {
            let new_board = board.make_move_new(movement);

            if good || (*new_board.checkers() != EMPTY) {
                let eval = -alpha_beta_search(new_board, -window);

                if let BetaPrune { beta } = window.negamax(eval) {
                    return beta;
                }
            }
        }
    }

    window.alpha()
}

pub fn qsearch(board: Board) -> Score {
    alpha_beta_search(board, AlphaBeta::new())
}
