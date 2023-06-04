use chess::ChessMove;

use crate::{
    evaluate::{Score, MATE_CUTOFF},
    search::Depth,
};

use super::markers::Marker;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct TTableEntry {
    pub depth: Depth,
    pub score: Score,
    pub best_move: Option<ChessMove>,
    pub marker: Marker,
}

impl TTableEntry {
    pub fn new(depth: Depth, score: Score, best_move: ChessMove) -> TTableEntry {
        TTableEntry {
            score,
            depth: depth.max(0),
            best_move: Some(best_move),
            marker: 0,
        }
    }

    pub fn leaf(score: Score) -> TTableEntry {
        TTableEntry {
            score,
            depth: 0,
            best_move: None,
            marker: 0,
        }
    }

    pub fn is_edge(&self) -> bool {
        self.score.abs() >= MATE_CUTOFF && self.depth < 1
    }

    pub fn update(&mut self, result: TTableEntry) {
        if self.depth <= result.depth {
            self.depth = result.depth;
            self.score = result.score;
            self.best_move = result.best_move;
        }
    }

    pub fn lazy_update(&mut self, result: &TTableEntry) {
        if self.depth <= result.depth && self.score < result.score {
            self.score = result.score;
            self.best_move = result.best_move;
        }
    }

    pub fn with_depth(self, depth: Depth) -> TTableEntry {
        TTableEntry {
            depth,
            score: self.score,
            best_move: self.best_move,
            marker: self.marker,
        }
    }
}
