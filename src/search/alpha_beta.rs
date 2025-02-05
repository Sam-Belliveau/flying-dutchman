use std::ops;

use crate::evaluate::{Score, CENTIPAWN, MATE};
use crate::search::Depth;
use crate::transposition::best_moves::BestMoves;
use crate::transposition::table_entry::TTableEntry;

pub enum NegaMaxResult {
    Worse,
    Best,
    Pruned,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AlphaBeta {
    pub alpha: Score,
    pub beta: Score,
    pub ply: Depth,
}

impl AlphaBeta {
    pub fn new() -> Self {
        Self {
            alpha: -MATE - CENTIPAWN,
            beta: MATE + CENTIPAWN,
            ply: 0,
        }
    }

    pub fn null_move(&self) -> Self {
        Self {
            alpha: 0 - self.beta,
            beta: 1 - self.beta,
            ply: self.ply,
        }
    }

    pub fn negamax(&mut self, score: Score) -> NegaMaxResult {
        if self.beta <= score {
            self.alpha = score;
            NegaMaxResult::Pruned
        } else if self.alpha <= score {
            self.alpha = score;
            NegaMaxResult::Best
        } else {
            NegaMaxResult::Worse
        }
    }

    pub fn new_table_entry(&self, depth: Depth, moves: BestMoves) -> TTableEntry {
        let score = moves.score();

        if score >= self.beta {
            TTableEntry::LowerNode(depth, moves)
        } else if score <= self.alpha {
            TTableEntry::UpperNode(depth, moves)
        } else {
            TTableEntry::ExactNode(depth, moves)
        }
    }

    pub fn opponent(&self) -> bool {
        self.ply % 2 == 1
    }

    pub fn span(&self) -> Score {
        self.beta - self.alpha
    }
}

impl ops::Neg for AlphaBeta {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            alpha: -self.beta,
            beta: -self.alpha,
            ply: self.ply + 1,
        }
    }
}
