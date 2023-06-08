use std::{collections::HashSet, mem::size_of, num::NonZeroUsize};

use chess::Board;
use lru::LruCache;

use super::{pv_line::PVLine, table_entry::TTableEntry};

const ELEMENT_SIZE: usize = 11
    * (size_of::<*const Board>()
        + size_of::<Board>()
        + size_of::<TTableEntry>()
        + 2 * size_of::<*const u64>()
        + size_of::<u64>())
    / 10;

pub type TTableHashMap = LruCache<(TTableType, Board), TTableEntry>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TTableType {
    Exact,
    Lower,
    Upper,
}

use TTableType::*;

pub struct TTable {
    table: TTableHashMap,
}

impl TTable {
    pub fn new(table_size: usize) -> TTable {
        TTable {
            table: TTableHashMap::new(NonZeroUsize::new(table_size / ELEMENT_SIZE).unwrap()),
        }
    }

    pub fn set_table_size(&mut self, table_size: usize) {
        self.table
            .resize(NonZeroUsize::new(table_size / ELEMENT_SIZE).unwrap())
    }

    pub fn get(&mut self, ttype: TTableType, board: Board) -> Option<&TTableEntry> {
        self.table.get(&(ttype, board))
    }

    fn free_space(&self) -> bool {
        self.table.len() < self.table.cap().into()
    }

    pub fn update(&mut self, ttype: TTableType, board: Board, result: TTableEntry) {
        if self.free_space() || ttype == Exact {
            let entry = self
                .table
                .get_or_insert_mut((ttype, board), || result.clone());
            entry.update(&result);
        } else {
            if let Some(entry) = self.table.get_mut(&(ttype, board)) {
                entry.update(&result);
            }
        }
    }

    pub fn get_pv_line(&mut self, board: Board) -> PVLine {
        PVLine::new(&mut self.table, board)
    }

    fn explore_pv_line(&mut self, board: Board, explored: &mut HashSet<Board>) {
        if explored.insert(board) {
            if let Some(moves) = (self.table.peek(&(Exact, board)))
                .or_else(|| self.table.peek(&(Lower, board)))
                .or_else(|| self.table.peek(&(Upper, board)))
                .map(|f| f.moves.clone())
            {
                self.table.promote(&(Exact, board));
                self.table.promote(&(Lower, board));
                self.table.promote(&(Upper, board));

                for movement in moves {
                    self.explore_pv_line(board.make_move_new(movement), explored);
                }
            }
        }
    }

    pub fn refresh_pv_line(&mut self, board: Board) {
        self.explore_pv_line(board, &mut HashSet::new());
    }

    pub fn hashfull_permille(&self) -> usize {
        self.table.len() * 1000 / self.table.cap()
    }

    pub fn memory_bytes(&self) -> usize {
        self.table.len() * ELEMENT_SIZE
    }
}
