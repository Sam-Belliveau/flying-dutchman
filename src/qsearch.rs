use chess::{Board, MoveGen};

use crate::{evaluate::evaluate, types::{Score, MAX_SCORE, MIN_SCORE}};

fn alpha_beta_search(board: &Board, mut alpha: Score, beta: Score) -> Score {
    let t1 = *board.pieces(chess::Piece::Queen);
    let t2 = *board.pieces(chess::Piece::Rook);
    let t3 = board.pieces(chess::Piece::Bishop) | board.pieces(chess::Piece::Knight);
    let t4 = *board.pieces(chess::Piece::Pawn);

    let eval = evaluate(board);
    alpha = alpha.max(eval);
    if eval >= beta {
        return beta;
    }

    let mut moves = MoveGen::new_legal(&board);
    for mask in [t1, t2, t3, t4] {
        moves.set_iterator_mask(mask);

        for movement in &mut moves {
            let eval = -alpha_beta_search(&board.make_move_new(movement), -beta, -alpha);

            alpha = alpha.max(eval);
            if eval >= beta {
                return beta;
            }
        }
    }

    alpha
}

pub fn qsearch(board: &Board) -> Score {
    alpha_beta_search(board, MIN_SCORE, MAX_SCORE)
}
