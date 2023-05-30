use std::{collections::HashMap, hash::BuildHasherDefault, mem::size_of};

use chess::{Board, ChessMove};

use super::{markers::MarkerQueue, table_entry::TTableEntry};
use nohash_hasher::NoHashHasher;

const ELEMENT_SIZE: usize = size_of::<TTableEntry>() + size_of::<Board>() + size_of::<u64>();
const SLICE_SIZE: usize = 100 * 1000 * 1000 / ELEMENT_SIZE;
const TABLE_SIZE: usize = 2000 * 1000 * 1000;

pub struct TTable {
    table: HashMap<Board, TTableEntry, BuildHasherDefault<NoHashHasher<u64>>>,
    queue: MarkerQueue,
}

impl TTable {
    pub fn new() -> TTable {
        TTable {
            table: HashMap::with_hasher(BuildHasherDefault::default()),
            queue: MarkerQueue::new(SLICE_SIZE),
        }
    }

    pub fn save(&mut self, board: &Board, result: TTableEntry) {
        self.queue.count(
            self.table
                .entry(*board)
                .and_modify(|e| e.update(result))
                .or_insert(result),
        );
    }

    pub fn update(&mut self, board: &Board, result: TTableEntry) {
        self.queue.count(
            self.table
                .entry(*board)
                .and_modify(|e| e.lazy_update(&result))
                .or_insert(result.with_depth(0)),
        );
    }

    pub fn get(&mut self, board: &Board) -> Option<&TTableEntry> {
        if let Some(entry) = self.table.get_mut(board) {
            self.queue.count(entry);
            Some(entry)
        } else {
            None
        }
    }

    pub fn get_pv_line(&mut self, board: &Board) -> Option<(Board, ChessMove)> {
        if let Some(entry) = self.get(board).cloned() {
            if let Some(best_move) = entry.best_move {
                let next_board = board.make_move_new(best_move);

                if let Some(next_entry) = self.get(&next_board) {
                    if next_entry.depth < entry.depth {
                        return Some((next_board, best_move));
                    }
                }
            }
        }

        None
    }

    pub fn refresh_pv_line(&mut self, mut board: Board) {
        while let Some((next_board, _)) = self.get_pv_line(&board) {
            board = next_board;
        }
    }

    pub fn sweep(&mut self) {
        if self.memory_bytes() > TABLE_SIZE {
            let quota = self.memory_bytes() - TABLE_SIZE / 2;
            let mut amount = 0;
            let mut sweeper = 0;

            while let (Some((marker, count)), true) =
                (self.queue.pop(), Self::size_to_bytes(amount) < quota)
            {
                amount += count;
                sweeper = marker + 1;
            }

            self.table.retain(|_, v| sweeper < v.marker);
        }
    }

    fn size_to_bytes(size: usize) -> usize {
        size * ELEMENT_SIZE * 11 / 10
    }

    pub fn memory_bytes(&self) -> usize {
        Self::size_to_bytes(self.table.len())
    }
}
