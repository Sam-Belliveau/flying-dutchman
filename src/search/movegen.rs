use chess::{BitBoard, Board, ChessMove, MoveGen, EMPTY};

use crate::transposition::best_moves::BestMoves;

pub struct OrderedMoveGen {
    pv_iter: BestMoves,
    masks: std::array::IntoIter<BitBoard, 6>,
    move_gen: MoveGen,
}

impl OrderedMoveGen {
    fn initialize(mut self) -> Self {
        self.move_gen.set_iterator_mask(self.masks.next().unwrap());
        self
    }

    pub fn full_search(board: &Board, pv: BestMoves) -> OrderedMoveGen {
        OrderedMoveGen {
            pv_iter: pv,
            masks: [
                *board.pieces(chess::Piece::Queen),
                *board.pieces(chess::Piece::Rook),
                *board.pieces(chess::Piece::Bishop),
                *board.pieces(chess::Piece::Knight),
                *board.pieces(chess::Piece::Pawn),
                !EMPTY,
            ]
            .into_iter(),
            move_gen: MoveGen::new_legal(board),
        }
        .initialize()
    }

    pub fn quiescence_search(board: &Board, pv: BestMoves) -> OrderedMoveGen {
        OrderedMoveGen {
            pv_iter: pv,
            masks: [
                *board.pieces(chess::Piece::Queen),
                *board.pieces(chess::Piece::Rook),
                *board.pieces(chess::Piece::Bishop),
                *board.pieces(chess::Piece::Knight),
                *board.pieces(chess::Piece::Pawn),
                EMPTY,
            ]
            .into_iter(),
            move_gen: MoveGen::new_legal(board),
        }
        .initialize()
    }
}

impl Iterator for OrderedMoveGen {
    type Item = ChessMove;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(pv) = self.pv_iter.pop() {
            self.move_gen.remove_move(pv.mv);
            Some(pv.mv)
        } else if let Some(mv) = self.move_gen.next() {
            Some(mv)
        } else if let Some(mask) = self.masks.next() {
            self.move_gen.set_iterator_mask(mask);
            self.next()
        } else {
            None
        }
    }
}
