use chess::{Board, ChessMove};
use circular_buffer::CircularBuffer;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BoardHistory {
    history: CircularBuffer<12, (Board, Option<ChessMove>)>,
}

impl BoardHistory {
    pub fn new(board: Board) -> Self {
        BoardHistory {
            history: CircularBuffer::from([(board, None)]),
        }
    }

    pub fn with_board(&self, new_board: Board) -> BoardHistory {
        let mut new = self.clone();
        new.history.push_front((new_board, None));
        new
    }

    pub fn with_move(&self, movement: ChessMove) -> BoardHistory {
        let mut new = self.clone();
        let new_board = self.last().make_move_new(movement);
        new.history.push_front((new_board, Some(movement)));
        new
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
            &self.history.front().unwrap_unchecked().0
        }
    }

    pub fn is_draw(&self) -> bool {
        let mut matches: i32 = 0;
        let mut boards = self.history.iter();

        if let Some(last_board) = boards.next() {
            for board in boards {
                if last_board.0 == board.0 {
                    matches += 1;
                }
            }
        }

        matches >= 2
    }

    pub fn to_uci_position(&self) -> String {
        let mut boards = self.history.iter().rev();

        if let Some((mut start_pos, _)) = boards.next() {
            let mut moves: Vec<ChessMove> = Vec::new();

            for (board, movement) in boards {
                if let Some(movement) = movement {
                    moves.push(*movement);
                } else {
                    start_pos = *board;
                    moves.clear();
                }
            }

            if moves.is_empty() {
                format!("position fen {}", start_pos)
            } else {
                let mut uci = format!("position fen {} moves", start_pos);
                for movement in moves {
                    uci.push_str(format!(" {}", movement).as_str());
                }
                uci
            }
        } else {
            return "position startpos".to_string();
        }
    }
}
