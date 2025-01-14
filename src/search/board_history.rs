use chess::{Board, ChessMove};

#[derive(Clone, Copy, Debug)]
pub enum BoardHistory {
    Last1(Board),
    Last2(Board, Board),
    Last3(Board, Board, Board),
    Last4(Board, Board, Board, Board),
    Last5(Board, Board, Board, Board, Board),
    Last6(Board, Board, Board, Board, Board, Board),
    Last7(Board, Board, Board, Board, Board, Board, Board),
    Last8(Board, Board, Board, Board, Board, Board, Board, Board),
    Last9(
        Board,
        Board,
        Board,
        Board,
        Board,
        Board,
        Board,
        Board,
        Board,
    ),
}

impl BoardHistory {
    pub fn new(board: Board) -> Self {
        Self::Last1(board)
    }

    pub fn with_board(&self, board: Board) -> BoardHistory {
        match *self {
            Self::Last1(b1) => Self::Last2(board, b1),
            Self::Last2(b1, b2) => Self::Last3(board, b1, b2),
            Self::Last3(b1, b2, b3) => Self::Last4(board, b1, b2, b3),
            Self::Last4(b1, b2, b3, b4) => Self::Last5(board, b1, b2, b3, b4),
            Self::Last5(b1, b2, b3, b4, b5) => Self::Last6(board, b1, b2, b3, b4, b5),
            Self::Last6(b1, b2, b3, b4, b5, b6) => Self::Last7(board, b1, b2, b3, b4, b5, b6),
            Self::Last7(b1, b2, b3, b4, b5, b6, b7) => {
                Self::Last8(board, b1, b2, b3, b4, b5, b6, b7)
            }
            Self::Last8(b1, b2, b3, b4, b5, b6, b7, b8) => {
                Self::Last9(board, b1, b2, b3, b4, b5, b6, b7, b8)
            }
            Self::Last9(b1, b2, b3, b4, b5, b6, b7, b8, ..) => {
                Self::Last9(board, b1, b2, b3, b4, b5, b6, b7, b8)
            }
        }
    }

    pub fn with_move(&self, movement: ChessMove) -> BoardHistory {
        self.with_board(self.last().make_move_new(movement))
    }

    pub fn last(&self) -> Board {
        match *self {
            Self::Last1(b1, ..) => b1,
            Self::Last2(b1, ..) => b1,
            Self::Last3(b1, ..) => b1,
            Self::Last4(b1, ..) => b1,
            Self::Last5(b1, ..) => b1,
            Self::Last6(b1, ..) => b1,
            Self::Last7(b1, ..) => b1,
            Self::Last8(b1, ..) => b1,
            Self::Last9(b1, ..) => b1,
        }
    }

    pub fn is_draw(&self) -> bool {
        if let Self::Last9(b1, b2, b3, b4, b5, b6, b7, b8, b9) = *self {
            let a2 = (b1 == b2) as i32;
            let a3 = (b1 == b3) as i32;
            let a4 = (b1 == b4) as i32;
            let a5 = (b1 == b5) as i32;
            let a6 = (b1 == b6) as i32;
            let a7 = (b1 == b7) as i32;
            let a8 = (b1 == b8) as i32;
            let a9 = (b1 == b9) as i32;

            (a2 + a3 + a4 + a5 + a6 + a7 + a8 + a9) >= 2
        } else {
            false
        }
    }
}
