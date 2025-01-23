use chess::{Board, ChessMove};
use circular_buffer::CircularBuffer;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BoardHistory {
    history: CircularBuffer<9, Board>,
}

impl BoardHistory {
    pub fn new(board: Board) -> Self {
        BoardHistory {
            history: CircularBuffer::from([board]),
        }
    }

    pub fn with_board(&self, board: Board) -> BoardHistory {
        let mut new = self.clone();
        new.history.push_front(board);
        new
    }

    pub fn with_move(&self, movement: ChessMove) -> BoardHistory {
        let new_board = self.last().make_move_new(movement);
        self.with_board(new_board)
    }

    pub fn with_null_move(&self) -> Option<BoardHistory> {
        if let Some(null_board) = self.last().null_move() {
            Some(self.with_board(null_board))
        } else {
            None
        }
    }

    pub fn last(&self) -> &Board {
        unsafe {
            // The buffer is never empty because it starts with
            // a board and we never remove elements from it.
            self.history.front().unwrap_unchecked()
        }
    }

    pub fn is_draw(&self) -> bool {
        let mut matches: i32 = 0;
        let mut boards = self.history.iter();

        if let Some(last_board) = boards.next() {
            for board in boards {
                if last_board == board {
                    matches += 1;
                }
            }
        }

        matches >= 2
    }
}
