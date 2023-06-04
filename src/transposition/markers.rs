use std::collections::VecDeque;

use super::table_entry::TTableEntry;

pub type Marker = u64;
const MARKER_START: Marker = Marker::MAX / 4;

const TABLE_CHUNKS: usize = 128;

#[derive(Debug)]
pub struct MarkerQueue {
    queue: VecDeque<usize>,
    table_size: usize,
    head: Marker,
    begin: Marker,
}

impl MarkerQueue {
    pub fn new(table_size: usize) -> MarkerQueue {
        MarkerQueue {
            queue: VecDeque::from([0]),
            table_size,
            head: MARKER_START,
            begin: MARKER_START,
        }
    }

    fn get(&mut self, marker: Marker) -> Option<&mut usize> {
        if marker == 0 {
            None
        } else {
            Some(&mut self.queue[(marker - self.begin) as usize])
        }
    }

    pub fn count(&mut self, entry: &mut TTableEntry) -> bool {
        if entry.marker < self.head {
            if let Some(count) = self.get(entry.marker) {
                *count -= 1;
            }

            entry.marker = self.head;

            let slice_size = self.table_size / TABLE_CHUNKS;
            if let Some(count) = self.get(entry.marker) {
                *count += 1;

                if slice_size < *count {
                    self.head += 1;
                    self.queue.push_back(0);
                    return true;
                }
            } else {
                panic!("It is not possible for the head to not be in the queue");
            }
        }

        return false;
    }

    pub fn pop(&mut self) -> Option<(Marker, usize)> {
        if let Some(count) = self.queue.pop_front() {
            let marker = self.begin;
            self.begin += 1;
            Some((marker, count))
        } else {
            None
        }
    }

    pub fn set_table_size(&mut self, table_size: usize) {
        self.table_size = table_size;
    }

    pub fn max_table_size(&self) -> usize {
        self.table_size
    }

    pub fn total(&self) -> usize {
        self.queue.iter().sum()
    }
}
