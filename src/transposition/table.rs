use std::{collections::HashMap, hash::BuildHasherDefault, mem::size_of};

use chess::Board;

use super::table_entry::{Marker, TTableEntry};
use nohash_hasher::NoHashHasher;

pub struct TTable {
    table: HashMap<Board, TTableEntry, BuildHasherDefault<NoHashHasher<u64>>>,
    marker: Marker,
    sweeper: Marker,
}

impl TTable {
    pub fn new() -> TTable {
        TTable {
            table: HashMap::with_hasher(BuildHasherDefault::default()),
            marker: 1,
            sweeper: 1,
        }
    }

    pub fn save(&mut self, board: &Board, result: TTableEntry) {
        self.table
            .entry(board.clone())
            .and_modify(|e| e.update(result))
            .or_insert(result)
            .set_marker(self.marker);
    }

    pub fn update(&mut self, board: &Board, result: TTableEntry) {
        self.table
            .entry(board.clone())
            .and_modify(|e| e.lazy_update(&result))
            .or_insert(result.with_depth(0))
            .set_marker(self.marker);
    }

    pub fn get(&mut self, board: &Board) -> Option<&TTableEntry> {
        if let Some(entry) = self.table.get_mut(board) {
            Some(entry.set_marker(self.marker))
        } else {
            None
        }
    }

    pub fn mark(&mut self) {
        self.marker += 1;
    }

    pub fn sweep(&mut self, max_size: usize) {
        while self.memory_bytes() > max_size {
            self.table.retain(|_, v| v.is_valid(self.sweeper));
            self.marker += 1;
            self.sweeper += 1;
        }
    }

    pub fn memory_bytes(&self) -> usize {
        let element_size = size_of::<TTableEntry>() + size_of::<Board>() + size_of::<u64>();
        self.table.len() * element_size * 11 / 10
    }
}
