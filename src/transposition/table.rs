use std::{collections::HashSet, mem::size_of, num::NonZeroUsize};

use chess::Board;
use lru::LruCache;

use crate::{
    evaluate::Score,
    search::{
        alpha_beta::{AlphaBeta, ProbeResult},
        Depth,
    },
};

use super::{best_moves::BestMoves, pv_line::PVLine, table_entry::TTableEntry};

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

pub enum TTableSample {
    None,
    Moves(BestMoves),
    Score(Score),
}

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

    pub fn update(&mut self, ttype: TTableType, board: Board, result: TTableEntry) {
        let entry = self
            .table
            .get_or_insert_mut((ttype, board), || result.clone());
        entry.update(&result);
    }

    pub fn sample<const LEAF: bool>(
        &mut self,
        board: &Board,
        window: &AlphaBeta,
        depth: Depth,
    ) -> TTableSample {
        let mut pv = None;

        if let Some(saved) = self.table.peek(&(Exact, *board)) {
            if !LEAF && saved.moves.is_some() {
                pv = pv.or(Some(saved.moves));
            }

            if LEAF || depth <= saved.depth {
                let result = TTableSample::Score(saved.score());
                self.table.promote(&(Exact, *board));
                return result;
            }
        }

        if let Some(saved) = self.table.peek(&(Upper, *board)) {
            if !LEAF && saved.moves.is_some() {
                pv = pv.or(Some(saved.moves));
            }

            if LEAF || depth <= saved.depth {
                let score = saved.score();
                if let ProbeResult::AlphaPrune { .. } = window.probe(score) {
                    let result = TTableSample::Score(saved.score());
                    self.table.promote(&(Upper, *board));
                    return result;
                }
            }
        }

        if let Some(saved) = self.table.peek(&(Lower, *board)) {
            if !LEAF && saved.moves.is_some() {
                pv = pv.or(Some(saved.moves));
            }

            if LEAF || depth <= saved.depth {
                let score = saved.score();
                if let ProbeResult::BetaPrune { .. } = window.probe(score) {
                    let result = TTableSample::Score(saved.score());
                    self.table.promote(&(Lower, *board));
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

    pub fn get_pv_line(&mut self, board: Board) -> PVLine {
        PVLine::new(&mut self.table, board)
    }

    fn explore_pv_line(&mut self, board: Board, explored: &mut HashSet<Board>) {
        if explored.insert(board) {
            if let Some(moves) = (self.table.peek(&(Exact, board)))
                .or_else(|| self.table.peek(&(Upper, board)))
                .or_else(|| self.table.peek(&(Lower, board)))
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
