use chess::{Board, EMPTY};

use crate::evaluate::{Score, evaluate};

use super::{
    alpha_beta::{AlphaBeta, NegaMaxResult::*},
    movegen::OrderedMoveGen,
};

pub fn ab_qsearch(board: Board, mut window: AlphaBeta) -> Score {
    let movegen = {
        if *board.checkers() == EMPTY {
            let score = evaluate(&board);
            if let Pruned { .. } = window.negamax(score) {
                return score;
            }

            OrderedMoveGen::quiescence_search(&board)
        } else {
            OrderedMoveGen::full_search(&board, None)
        }
    };

    for movement in movegen {
        let new_board = board.make_move_new(movement);
        let score = -ab_qsearch(new_board, -window);

        if let Pruned { .. } = window.negamax(score) {
            return score;
        }
    }

    window.alpha()
}

pub fn qsearch(board: Board) -> Score {
    ab_qsearch(board, AlphaBeta::new())
}
