use chess::{BitBoard, Board, ChessMove, MoveGen, EMPTY};

pub struct OrderedMoveGen {
    pv: Option<ChessMove>,
    masks: std::array::IntoIter<BitBoard, 6>,
    move_gen: MoveGen,
}

impl<'a> OrderedMoveGen {
    fn initialize(mut self) -> Self {
        if let Some(pv) = self.pv {
            assert!(self.move_gen.remove_move(pv));
        }

        self.move_gen.set_iterator_mask(self.masks.next().unwrap());

        self
    }

    pub fn new(board: &Board, pv: Option<ChessMove>) -> OrderedMoveGen {
        OrderedMoveGen {
            pv,
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

    pub fn new_qsearch(board: &Board) -> OrderedMoveGen {
        OrderedMoveGen {
            pv: None,
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
        if let Some(pv) = self.pv.take() {
            return Some(pv);
        }

        if let Some(movement) = self.move_gen.next() {
            return Some(movement);
        }

        if let Some(mask) = self.masks.next() {
            self.move_gen.set_iterator_mask(mask);
            return self.next();
        }

        None
    }
}
