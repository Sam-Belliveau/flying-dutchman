use crate::{
    evaluate::{Score},
    search::{Depth, DEPTH_EDGE, DEPTH_LEAF},
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

    pub fn leaf(score: Score) -> TTableEntry {
        TTableEntry {
            depth: DEPTH_LEAF,
            moves: BestMoves::None(score),
        }
    }

    pub fn edge(score: Score) -> TTableEntry {
        TTableEntry {
            depth: DEPTH_EDGE,
            moves: BestMoves::None(score),
        }
    }

    pub fn is_edge(&self) -> bool {
        self.depth >= DEPTH_EDGE
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

    pub fn score(&self) -> Score {
        self.moves.best_score()
    }
}
