use chess::ChessMove;

use crate::{
    evaluate::{Score, MATE_CUTOFF},
    search::Depth,
};

#[derive(Clone, Debug)]
pub struct TTableEntry {
    pub depth: Depth,
    pub score: Score,
    pub best_move: Option<ChessMove>,
}

impl TTableEntry {
    pub fn new(depth: Depth, score: Score, best_move: ChessMove) -> TTableEntry {
        TTableEntry {
            score,
            depth: depth.max(0),
            best_move: Some(best_move),
        }
    }

    pub fn leaf(score: Score) -> TTableEntry {
        TTableEntry {
            score,
            depth: 0,
            best_move: None,
        }
    }

    pub fn is_edge(&self) -> bool {
        self.score.abs() >= MATE_CUTOFF && self.depth < 1
    }

    pub fn update(&mut self, result: &TTableEntry) {
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
}
