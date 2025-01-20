use std::collections::HashSet;

use chess::{Board, ChessMove};

use crate::transposition::table::TTable;

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
        let entry = self.table.get::<true>(&self.board)?;
        let best_move = entry.peek()?;

        if self.passed.insert(self.board) {
            self.board = self.board.make_move_new(best_move);
            Some(best_move)
        } else {
            None
        }
    }
}
