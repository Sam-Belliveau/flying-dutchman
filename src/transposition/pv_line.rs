use std::collections::HashSet;

use chess::{Board, ChessMove};

use super::table::{TTableHashMap, TTableType::*};

pub struct PVLine<'a> {
    table: &'a mut TTableHashMap,
    passed: HashSet<Board>,
    board: Board,
}

impl<'a> PVLine<'a> {
    pub fn new(table: &'a mut TTableHashMap, board: Board) -> PVLine<'a> {
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
        if let Some(entry) = (self.table.get(&(Exact, self.board)).cloned())
            .or_else(|| self.table.get(&(Upper, self.board)).cloned())
            .or_else(|| self.table.get(&(Lower, self.board)).cloned())
        {
            if let Some(best_move) = entry.moves.peek() {
                if self.passed.insert(self.board) {
                    self.board = self.board.make_move_new(best_move);

                    return Some(best_move);
                }
            }
        }

        None
    }
}
