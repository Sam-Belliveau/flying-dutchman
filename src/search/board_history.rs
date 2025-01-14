use chess::{Board, ChessMove};
use circular_buffer::CircularBuffer;

#[derive(Clone, Debug)]
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
        new.history.push_back(board);
        new
    }

    pub fn with_move(&self, movement: ChessMove) -> BoardHistory {
        let new_board = self.last().make_move_new(movement);
        self.with_board(new_board)
    }

    pub fn last(&self) -> &Board {
        unsafe {
            // The buffer is never empty because it starts with
            // a board and we never remove elements from it.
            self.history.back().unwrap_unchecked()
        }
    }

    pub fn is_draw(&self) -> bool {
        let mut matches: i32 = 0;
        let last = *self.last();
        for board in self.history.iter().skip(1) {
            matches += (last == *board) as i32;
        }

        matches >= 2
    }
}
