use std::collections::HashSet;

use chess::{Board, ChessMove};

use crate::transposition::table::{TTable, TTableType::*};

pub struct PVLine<'a> {
    table: &'a mut TTable,
    passed: HashSet<Board>,
    board: Board,
}

impl<'a> PVLine<'a> {
    pub fn new(table: &'a mut TTable, board: Board) -> PVLine<'a> {
        PVLine {
            table,
            passed: HashSet::new(),
            board,
        }
    }
}

impl Iterator for PVLine<'_> {
    type Item = ChessMove;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(entry) = (self.table.get::<true>(Exact, &self.board))
            .or_else(|| self.table.get::<true>(Lower, &self.board))
            .or_else(|| self.table.get::<true>(Upper, &self.board))
        {
            if let Some(best_move) = entry.peek() {
                if self.passed.insert(self.board) {
                    self.board = self.board.make_move_new(best_move);

                    return Some(best_move);
                }
            }
        }

        None
    }
}
