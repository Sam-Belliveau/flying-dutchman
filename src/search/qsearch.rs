use chess::{Board, EMPTY};

use crate::evaluate::{evaluate, Score};

use super::{
    alpha_beta::{AlphaBeta, NegaMaxResult::*},
    movegen::OrderedMoveGen,
};

pub fn ab_qsearch(board: Board, mut window: AlphaBeta) -> Score {
    let movegen = if *board.checkers() == EMPTY {
        let eval = evaluate(&board);
        if let Pruned { .. } = window.negamax(eval) {
            return eval;
        }

        OrderedMoveGen::new_qsearch(&board)
    } else {
        OrderedMoveGen::new(&board, None)
    };

    for movement in movegen {
        let new_board = board.make_move_new(movement);
        let eval = -ab_qsearch(new_board, -window);

        if let Pruned { .. } = window.negamax(eval) {
            return eval;
        }
    }

    window.alpha()
}

pub fn qsearch(board: Board) -> Score {
    ab_qsearch(board, AlphaBeta::new())
}
