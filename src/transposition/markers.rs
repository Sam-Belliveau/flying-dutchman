use std::collections::VecDeque;

use super::table_entry::TTableEntry;

pub type Marker = u64;
const MARKER_START: Marker = Marker::MAX / 4;

#[derive(Debug)]
pub struct MarkerQueue {
    queue: VecDeque<usize>,
    slice_size: usize,
    head: Marker,
    begin: Marker,
}

impl MarkerQueue {
    pub fn new(slice_size: usize) -> MarkerQueue {
        MarkerQueue {
            queue: VecDeque::from([0]),
            slice_size,
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

    pub fn count(&mut self, entry: &mut TTableEntry) {
        if self.head != entry.marker {
            if let Some(count) = self.get(entry.marker) {
                *count -= 1;
            }

            entry.marker = self.head;

            let slice_size = self.slice_size;
            if let Some(count) = self.get(entry.marker) {
                *count += 1;

                if slice_size < *count {
                    self.head += 1;
                    self.queue.push_back(0)
                }
            }
        }
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
}
