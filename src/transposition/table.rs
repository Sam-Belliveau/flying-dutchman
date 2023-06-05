use std::{hash::BuildHasherDefault, mem::size_of, num::NonZeroUsize};

use chess::Board;
use lru::LruCache;

use super::{pv_line::PVLine, table_entry::TTableEntry};
use nohash_hasher::NoHashHasher;

const ELEMENT_SIZE: usize =
    3 * (size_of::<TTableEntry>() + size_of::<Board>() + 2 * size_of::<*const u64>()) / 2;

pub type TTableHashMap = LruCache<Board, TTableEntry, BuildHasherDefault<NoHashHasher<u64>>>;

pub struct TTable {
    table: TTableHashMap,
}

impl TTable {
    pub fn new(table_size: usize) -> TTable {
        TTable {
            table: TTableHashMap::with_hasher(
                NonZeroUsize::new(table_size / ELEMENT_SIZE).unwrap(),
                BuildHasherDefault::default(),
            ),
        }
    }

    pub fn set_table_size(&mut self, table_size: usize) {
        self.table
            .resize(NonZeroUsize::new(table_size / ELEMENT_SIZE).unwrap())
    }

    pub fn get(&mut self, board: &Board) -> Option<&TTableEntry> {
        self.table.get(board)
    }

    pub fn update(&mut self, board: &Board, result: TTableEntry) {
        let entry = self.table.get_or_insert_mut(*board, || result.clone());
        entry.update(&result);
    }

    pub fn lazy_update(&mut self, board: &Board, result: TTableEntry) {
        if let Some(entry) = self.table.peek_mut(board) {
            entry.lazy_update(&result);
        }
    }

    pub fn get_pv_line(&mut self, board: Board) -> PVLine {
        PVLine::new(&mut self.table, board)
    }

    pub fn refresh_pv_line(&mut self, board: Board) {
        for _ in self.get_pv_line(board) {}
    }

    pub fn hashfull_permille(&self) -> usize {
        self.table.len() * 1000 / self.table.cap()
    }

    pub fn memory_bytes(&self) -> usize {
        self.table.len() * ELEMENT_SIZE
    }
}
