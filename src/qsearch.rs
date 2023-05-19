use chess::{Board, MoveGen, get_bishop_moves, get_rook_moves, get_knight_moves, EMPTY};

use crate::{
    evaluate::evaluate,
    types::{Score, MAX_SCORE, MIN_SCORE},
};

fn alpha_beta_search(board: Board, mut alpha: Score, beta: Score) -> Score {
    let t1 = *board.pieces(chess::Piece::Queen);
    let t2 = *board.pieces(chess::Piece::Rook);
    let t3 = board.pieces(chess::Piece::Bishop) | board.pieces(chess::Piece::Knight);
    let t4 = *board.pieces(chess::Piece::Pawn);

    let eval = evaluate(&board);
    alpha = alpha.max(eval);
    if eval >= beta {
        return beta;
    }

    let mut moves = MoveGen::new_legal(&board);
    for mask in [t1, t2, t3, t4] {
        moves.set_iterator_mask(mask);

        for movement in &mut moves {
            let eval = -alpha_beta_search(board.make_move_new(movement), -beta, -alpha);

            alpha = alpha.max(eval);
            if alpha >= beta {
                return beta;
            }
        }
    }

    let pieces = *board.combined();
    let king = board.king_square(!board.side_to_move());
    let checkers = get_bishop_moves(king, pieces) | get_rook_moves(king, pieces) | get_knight_moves(king);

    for mask in [pieces & checkers, checkers] {
        moves.set_iterator_mask(mask);

        for movement in &mut moves {
            let result = board.make_move_new(movement);
            if *result.checkers() != EMPTY {
                let eval = -alpha_beta_search(result, -beta, -alpha);

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
