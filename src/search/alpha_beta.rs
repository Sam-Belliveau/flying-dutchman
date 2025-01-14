use std::ops;

use crate::evaluate::{Score, MATE};

use super::Depth;

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
            alpha: -MATE,
            beta: MATE,
            ply: 0,
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

    pub fn opponent(&self) -> bool {
        self.ply % 2 == 1
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
