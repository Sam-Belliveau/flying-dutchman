use chess::{BitBoard, Board, ChessMove, EMPTY};

const REPETITION_SEARCH_DEPTH: usize = 9;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum BoardChain<'a> {
    First(Board, usize),
    Linked(&'a BoardChain<'a>, Board, usize),
    OwnedLink(Box<BoardChain<'a>>, Board, usize),
}

impl BoardChain<'static> {
    pub fn new(board: Board) -> Self {
        BoardChain::First(board, 1)
    }

    pub fn take_board(self, board: Board) -> BoardChain<'static> {
        let repetitions = self.get_repetitions(&board, REPETITION_SEARCH_DEPTH);
        match self {
            BoardChain::First(_, _) => BoardChain::OwnedLink(Box::new(self), board, repetitions),
            BoardChain::Linked(_, _, _) => panic!("Cannot take board from linked chain"),
            BoardChain::OwnedLink(_, _, _) => {
                BoardChain::OwnedLink(Box::new(self), board, repetitions)
            }
        }
    }

    pub fn take_move(self, movement: ChessMove) -> BoardChain<'static> {
        let new_board = self.last().make_move_new(movement);
        let capture = self.last().combined() & BitBoard::from_square(movement.get_dest());
        if capture != EMPTY {
            BoardChain::new(new_board)
        } else {
            self.take_board(new_board)
        }
    }
}

impl<'a> BoardChain<'a> {
    pub fn get_repetitions(&self, test_board: &Board, depth: usize) -> usize {
        if depth <= 0 {
            return 1;
        } else {
            match self {
                BoardChain::First(board, repetitions) => {
                    if board == test_board {
                        *repetitions + 1
                    } else {
                        1
                    }
                }
                BoardChain::Linked(previous, board, repetitions) => {
                    if board == test_board {
                        *repetitions + 1
                    } else {
                        previous.get_repetitions(test_board, depth - 1)
                    }
                }
                BoardChain::OwnedLink(previous, board, repetitions) => {
                    if board == test_board {
                        *repetitions + 1
                    } else {
                        previous.get_repetitions(test_board, depth - 1)
                    }
                }
            }
        }
    }

    pub fn with_board(&self, board: Board) -> BoardChain {
        let repetitions = self.get_repetitions(&board, REPETITION_SEARCH_DEPTH);
        BoardChain::Linked(self, board, repetitions)
    }

    pub fn with_move(&self, movement: ChessMove) -> BoardChain {
        let new_board = self.last().make_move_new(movement);
        let capture = self.last().combined() & BitBoard::from_square(movement.get_dest());
        if capture != EMPTY {
            BoardChain::new(new_board)
        } else {
            self.with_board(new_board)
        }
    }

    pub fn with_null_move(&self) -> Option<BoardChain> {
        if let Some(null_board) = self.last().null_move() {
            Some(self.with_board(null_board))
        } else {
            None
        }
    }

    pub fn last(&self) -> &Board {
        match self {
            BoardChain::First(board, _) => board,
            BoardChain::Linked(_, board, _) => board,
            BoardChain::OwnedLink(_, board, _) => board,
        }
    }

    pub fn repetitions(&self) -> usize {
        match self {
            BoardChain::First(_, repetitions) => *repetitions,
            BoardChain::Linked(_, _, repetitions) => *repetitions,
            BoardChain::OwnedLink(_, _, repetitions) => *repetitions,
        }
    }

    pub fn is_draw(&self) -> bool {
        self.repetitions() >= 3
    }
}
