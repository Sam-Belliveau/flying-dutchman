use std::{collections::HashMap, hash::BuildHasherDefault, mem::size_of};

use chess::Board;

use super::{markers::MarkerQueue, table_entry::TTableEntry};
use nohash_hasher::NoHashHasher;

const SLICE_SIZE: usize = 100000;
const TABLE_SIZE: usize = 1000 * 1000 * 1000;

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
                .entry(board.clone())
                .and_modify(|e| e.update(result))
                .or_insert(result),
        );
    }

    pub fn update(&mut self, board: &Board, result: TTableEntry) {
        self.queue.count(
            self.table
                .entry(board.clone())
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

    pub fn refresh_pv(&mut self, mut board: Board) {
        while let Some(entry) = self.get(&board).and_then(|e| e.best_move) {
            board = board.make_move_new(entry);
        }
    }

    pub fn sweep(&mut self) {
        assert_eq!(self.queue.total(), self.table.len());
        if self.memory_bytes() > TABLE_SIZE {
            let quota = self.memory_bytes() - TABLE_SIZE;
            let mut amount = 0;
            let mut sweeper = 0;

            while let (Some((marker, count)), true) = (self.queue.pop(), amount < quota) {
                let bytes = Self::size_to_bytes(count);
                amount += bytes;
                sweeper = marker;

                println!("Cleared Marker {} to save {} bytes", marker, bytes);
            }

            self.table.retain(|_, v| v.marker > sweeper);
        }
    }

    fn size_to_bytes(size: usize) -> usize {
        let element_size = size_of::<TTableEntry>() + size_of::<Board>() + size_of::<u64>();
        size * element_size * 11 / 10
    }

    pub fn memory_bytes(&self) -> usize {
        Self::size_to_bytes(self.table.len())
    }
}
