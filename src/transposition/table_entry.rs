use chess::ChessMove;

use crate::{
    evaluate::{Score, MATE_CUTOFF},
    search::Depth,
};

pub type Marker = u64;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct TTableEntry {
    pub depth: Depth,
    pub score: Score,
    pub best_move: Option<ChessMove>,
    pub marker: Marker,
}

impl TTableEntry {
    pub fn new(depth: Depth, score: Score, bmove: Option<ChessMove>) -> TTableEntry {
        TTableEntry {
            score,
            depth: depth.max(0),
            best_move: bmove,
            marker: 0,
        }
    }

    pub fn leaf(score: Score) -> TTableEntry {
        TTableEntry::new(0, score, None)
    }

    pub fn is_edge(&self) -> bool {
        self.score.abs() >= MATE_CUTOFF
    }

    pub fn update(&mut self, result: TTableEntry) {
        if self.depth <= result.depth && result.best_move.is_some() {
            *self = result;
        }
    }

    pub fn lazy_update(&mut self, result: &TTableEntry) {
        if self.depth <= result.depth && self.score < result.score && result.best_move.is_some() {
            self.score = result.score;
            self.best_move = result.best_move;
        }
    }

    pub fn with_depth(&self, depth: Depth) -> TTableEntry {
        TTableEntry {
            depth,
            score: self.score,
            best_move: self.best_move,
            marker: self.marker,
        }
    }

    pub fn set_marker(&mut self, marker: Marker) -> bool {
        if self.marker < marker {
            self.marker = marker;
            true
        } else {
            false
        }
    }
}
