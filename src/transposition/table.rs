use std::{hash::Hash, mem::size_of, num::NonZeroUsize};

use chess::Board;
use lru::LruCache;

use crate::search::{alpha_beta::AlphaBeta, Depth};

use super::{best_moves::BestMoves, pv_line::PVLine, table_entry::TTableEntry};

const ELEMENT_SIZE: usize = 11
    * (size_of::<*const Board>()
        + size_of::<Board>()
        + size_of::<TTableEntry>()
        + 2 * size_of::<*const u64>()
        + size_of::<u64>())
    / 10;

const PV_TABLE_SIZE: usize = 256;

pub type TTableHashMap = LruCache<(TTableType, Board), TTableEntry>;
pub type PVTableHashMap = LruCache<(TTableType, Board), TTableEntry>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TTableType {
    Exact,
    Lower,
    Upper,
}

use TTableType::*;

pub enum TTableSample {
    None,
    Moves(BestMoves),
    Score(TTableEntry),
}

pub struct TTable {
    table: TTableHashMap,
    pv_table: PVTableHashMap,
}

impl TTable {
    pub fn new(table_size: usize) -> TTable {
        TTable {
            table: TTableHashMap::new(NonZeroUsize::new(table_size / ELEMENT_SIZE).unwrap()),
            pv_table: PVTableHashMap::new(NonZeroUsize::new(PV_TABLE_SIZE).unwrap()),
        }
    }

    pub fn set_table_size(&mut self, table_size: usize) {
        self.table
            .resize(NonZeroUsize::new(table_size / ELEMENT_SIZE).unwrap())
    }

    pub fn update<const PV: bool>(
        &mut self,
        ttype: TTableType,
        board: &Board,
        result: TTableEntry,
    ) {
        let key = (ttype, *board);
        let entry = self.table
            .get_or_insert_mut(key, || result)
            .update(result);

        if PV {
            self.pv_table.put(key, *entry);
        }
    }

    pub fn peek(&self, ttype: TTableType, board: &Board) -> Option<&TTableEntry> {
        let key = (ttype, *board);
        self.table.peek(&key).or_else(|| self.pv_table.peek(&key))
    }

    pub fn get<const PV: bool>(&mut self, ttype: TTableType, board: &Board) -> Option<TTableEntry> {
        let entry = self.peek(ttype, board).cloned()?;
        self.update::<PV>(ttype, board, entry);
        Some(entry)
    }

    pub fn sample<const PV: bool>(
        &mut self,
        board: &Board,
        window: &AlphaBeta,
        depth: Depth,
    ) -> TTableSample {
        let mut pv = None;

        if let Some(saved) = self.peek(Exact, board) {
            pv = pv.or(saved.moves());

            if depth <= saved.depth() {
                let result = TTableSample::Score(*saved);
                self.update::<PV>(Exact, board, *saved);
                return result;
            }
        }

        if let Some(saved) = self.peek(Lower, board) {
            pv = pv.or(saved.moves());

            if depth <= saved.depth() {
                let score = saved.score();
                if window.beta <= score {
                    let result = TTableSample::Score(*saved);
                    self.update::<PV>(Exact, board, *saved);
                    return result;
                }
            }
        }

        if let Some(saved) = self.peek(Upper, board) {
            pv = pv.or(saved.moves());

            if depth <= saved.depth() {
                let score = saved.score();
                if score <= window.alpha {
                    let result = TTableSample::Score(*saved);
                    self.update::<PV>(Exact, board, *saved);
                    return result;
                }
            }
        }

        if let Some(moves) = pv {
            TTableSample::Moves(moves)
        } else {
            TTableSample::None
        }
    }

    pub fn promote_pv_line(&mut self, board: &Board) {
        for _move in self.get_pv_line(board) {
            // This promotes the PV line to the front of the cache
        }
    }

    pub fn get_pv_line(&mut self, board: &Board) -> PVLine {
        PVLine::new(self, *board)
    }

    pub fn hashfull_permille(&self) -> usize {
        self.table.len() * 1000 / self.table.cap()
    }

    pub fn memory_bytes(&self) -> usize {
        self.table.len() * ELEMENT_SIZE
    }
}
