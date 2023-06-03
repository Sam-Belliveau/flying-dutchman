use std::collections::HashSet;

use chess::{Board, ChessMove};

use super::table::TTable;

pub struct PVLine<'a> {
    table: &'a mut TTable,
    passed: HashSet<ChessMove>,
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
        if let Some(entry) = self.table.get(&self.board) {
            if let Some(best_move) = entry.best_move {
                if self.passed.insert(best_move) {
                    self.board = self.board.make_move_new(best_move);

                    return Some(best_move);
                }
            }
        }

        None
    }
}
