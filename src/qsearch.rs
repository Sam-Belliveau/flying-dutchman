use chess::{get_bishop_moves, get_knight_moves, get_rook_moves, Board, MoveGen, EMPTY};

use crate::{
    evaluate::evaluate,
    types::{Score, MAX_SCORE, MIN_SCORE},
};

fn alpha_beta_search(board: Board, mut alpha: Score, beta: Score) -> Score {
    let eval = evaluate(&board);
    alpha = alpha.max(eval);
    if eval >= beta {
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
                let eval = -alpha_beta_search(new_board, -beta, -alpha);

                alpha = alpha.max(eval);
                if alpha >= beta {
                    return beta;
                }
            }
        }
    }

    alpha
}

pub fn qsearch(board: Board) -> Score {
    alpha_beta_search(board, MIN_SCORE, MAX_SCORE)
}
