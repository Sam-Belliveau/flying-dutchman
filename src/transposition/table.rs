use std::{
    collections::{BTreeMap, HashMap},
    hash::BuildHasherDefault,
    mem::size_of,
};

use chess::Board;

use super::table_entry::{Marker, TTableEntry};
use nohash_hasher::NoHashHasher;

const SLICE_SIZE: usize = 100 * 1000 * 1000;
const TABLE_SIZE: usize = 10000 * 1000 * 1000;

pub struct TTable {
    table: HashMap<Board, TTableEntry, BuildHasherDefault<NoHashHasher<u64>>>,
    marker: Marker,
    slice: usize,
}

impl TTable {
    pub fn new() -> TTable {
        TTable {
            table: HashMap::with_hasher(BuildHasherDefault::default()),
            marker: 1,
            slice: 0,
        }
    }

    pub fn save(&mut self, board: &Board, result: TTableEntry) {
        let new = self
            .table
            .entry(board.clone())
            .and_modify(|e| e.update(result))
            .or_insert(result)
            .set_marker(self.marker);

        if new {
            Self::mark(&mut self.slice, &mut self.marker);
        }
    }

    pub fn update(&mut self, board: &Board, result: TTableEntry) {
        let new = self
            .table
            .entry(board.clone())
            .and_modify(|e| e.lazy_update(&result))
            .or_insert(result.with_depth(0))
            .set_marker(self.marker);

        if new {
            Self::mark(&mut self.slice, &mut self.marker);
        }
    }

    pub fn get(&mut self, board: &Board) -> Option<&TTableEntry> {
        if let Some(entry) = self.table.get_mut(board) {
            let new = entry.set_marker(self.marker);

            if new {
                Self::mark(&mut self.slice, &mut self.marker);
            }

            Some(entry)
        } else {
            None
        }
    }

    fn mark(slice: &mut usize, marker: &mut Marker) {
        *slice += 1;

        if Self::size_to_bytes(*slice) > SLICE_SIZE {
            *slice = 0;
            *marker += 1;
        }
    }

    pub fn sweep(&mut self) {
        if self.memory_bytes() > TABLE_SIZE {
            let mut alloc_map = BTreeMap::new();

            for (_, v) in self.table.iter() {
                alloc_map
                    .entry(v.marker)
                    .and_modify(|e| *e += 1)
                    .or_insert(1);
            }

            let quota = self.memory_bytes() - TABLE_SIZE;
            let mut amount = 0;
            let mut sweeper = 0;

            for (marker, count) in alloc_map {
                amount += Self::size_to_bytes(count);
                sweeper = marker;

                if amount >= quota {
                    break;
                }
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
