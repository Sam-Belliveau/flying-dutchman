use crate::{
    evaluate::{Score, MATE_CUTOFF},
    search::{Depth, DEPTH_EDGE},
};

use super::best_moves::BestMoves;

#[derive(Clone, Debug)]
pub struct TTableEntry {
    pub depth: Depth,
    pub moves: BestMoves,
}

impl TTableEntry {
    pub fn new(depth: Depth, moves: BestMoves) -> TTableEntry {
        TTableEntry {
            depth: depth.max(0),
            moves,
        }
    }

    pub fn edge(score: BestMoves) -> TTableEntry {
        TTableEntry {
            depth: DEPTH_EDGE,
            moves: score,
        }
    }

    pub fn is_edge(&self) -> bool {
        self.depth >= DEPTH_EDGE || self.score().abs() >= MATE_CUTOFF
    }

    pub fn update(&mut self, result: &TTableEntry) {
        if (self.depth.cmp(&result.depth))
            .then(self.score().cmp(&result.score()))
            .is_lt()
        {
            self.depth = result.depth;
            self.moves = result.moves;
        }
    }

    pub fn update_upper(&mut self, result: &TTableEntry) {
        if (self.depth.cmp(&result.depth))
            .then(result.score().cmp(&self.score()))
            .is_lt()
        {
            self.depth = result.depth;
            self.moves = result.moves;
        }
    }

    pub fn score(&self) -> Score {
        self.moves.best_score()
    }
}
