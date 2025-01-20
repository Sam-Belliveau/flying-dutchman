use std::mem::size_of;
use std::num::NonZeroUsize;

use chess::Board;
use lru::LruCache;

use crate::search::alpha_beta::AlphaBeta;
use crate::search::Depth;

use crate::transposition::best_moves::BestMoves;
use crate::transposition::pv_line::PVLine;
use crate::transposition::table_entry::TTableEntry;

const ELEMENT_SIZE: usize = 11
    * (size_of::<*const Board>()
        + size_of::<Board>()
        + size_of::<TTableEntry>()
        + 2 * size_of::<*const u64>()
        + size_of::<u64>())
    / 10;

const PV_TABLE_SIZE: usize = 256;

use TTableEntry::*;

pub type TTableKey = u64;
pub type TTableHashMap = LruCache<TTableKey, TTableEntry>;
pub type PVTableHashMap = LruCache<TTableKey, TTableEntry>;

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

    fn to_key(board: &Board) -> TTableKey {
        board.get_hash()
    }

    pub fn update<const PV: bool>(&mut self, board: &Board, result: TTableEntry) {
        let key = Self::to_key(board);
        let entry = self
            .table
            .get_or_insert_mut(key, || result)
            .update(result);

        if PV {
            self.pv_table.put(key, *entry);
        }
    }

    pub fn peek(&self, board: &Board) -> Option<&TTableEntry> {
        let key = Self::to_key(board);
        self.table.peek(&key).or_else(|| self.pv_table.peek(&key))
    }

    pub fn get<const PV: bool>(&mut self, board: &Board) -> Option<TTableEntry> {
        let entry = self.peek(board).cloned()?;
        self.update::<PV>(board, entry);
        Some(entry)
    }

    pub fn sample<const PV: bool>(
        &mut self,
        board: &Board,
        window: &AlphaBeta,
        depth: Depth,
    ) -> TTableSample {
        match self.peek(board).cloned() {
            Some(sample @ ExactNode(sample_depth, moves)) => {
                if depth <= sample_depth {
                    self.update::<PV>(board, sample);
                    TTableSample::Score(sample)
                } else {
                    TTableSample::Moves(moves)
                }
            }
            Some(sample @ LowerNode(sample_depth, moves)) => {
                if depth <= sample_depth && window.beta <= moves.score() {
                    self.update::<PV>(board, sample);
                    TTableSample::Score(sample)
                } else {
                    TTableSample::Moves(moves)
                }
            }
            Some(sample @ UpperNode(sample_depth, moves)) => {
                if depth <= sample_depth && moves.score() <= window.alpha {
                    self.update::<PV>(board, sample);
                    TTableSample::Score(sample)
                } else {
                    TTableSample::Moves(moves)
                }
            }
            Some(sample @ Edge(..)) => TTableSample::Score(sample),
            Some(sample @ Leaf(..)) => {
                if depth <= 0 {
                    TTableSample::Score(sample)
                } else {
                    TTableSample::None
                }
            }
            None => TTableSample::None,
        }
    }

    pub fn promote_pv_line(&mut self, board: &Board) {
        for _move in self.get_pv_line(board) {
            // This loop promotes the PV line to the top
            // of the cache just by iterating over it.
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
