use std::{collections::HashMap, hash::BuildHasherDefault, mem::size_of};

use chess::Board;

use super::{markers::MarkerQueue, pv_line::PVLine, table_entry::TTableEntry};
use nohash_hasher::NoHashHasher;

const ELEMENT_SIZE: usize =
    11 * (size_of::<TTableEntry>() + size_of::<Board>() + size_of::<u64>()) / 10;

pub struct TTable {
    table: HashMap<Board, TTableEntry, BuildHasherDefault<NoHashHasher<u64>>>,
    queue: MarkerQueue,
}

impl TTable {
    pub fn new(table_size: usize) -> TTable {
        TTable {
            table: HashMap::with_hasher(BuildHasherDefault::default()),
            queue: MarkerQueue::new(table_size / ELEMENT_SIZE),
        }
    }

    pub fn set_table_size(&mut self, table_size: usize) {
        self.queue.set_table_size(table_size / ELEMENT_SIZE);
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

    pub fn get_pv_line(&mut self, board: Board) -> PVLine {
        PVLine::new(self, board)
    }

    pub fn refresh_pv_line(&mut self, board: Board) {
        for _ in self.get_pv_line(board) {}
    }

    pub fn sweep(&mut self) {
        debug_assert_eq!(self.table.len(), self.queue.total());

        let table_len = self.table.len();
        let max_len = self.queue.max_table_size();

        if table_len > max_len {
            let quota = table_len - max_len / 2;

            let mut amount = 0;
            let mut sweeper = 0;

            while let (Some((marker, count)), true) = (self.queue.pop(), amount < quota) {
                amount += count;
                sweeper = marker + 1;
            }

            self.table.retain(|_, v| sweeper < v.marker);
        }
    }

    fn size_to_bytes(size: usize) -> usize {
        size * ELEMENT_SIZE
    }

    pub fn memory_bytes(&self) -> usize {
        Self::size_to_bytes(self.table.len())
    }
}
