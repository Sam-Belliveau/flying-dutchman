use std::ops;

use crate::evaluate::{Score, MATE};

pub enum NegaMaxResult {
    Worse,
    Best,
    Pruned,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AlphaBeta {
    pub alpha: Score,
    pub beta: Score,
}

impl AlphaBeta {
    pub fn new() -> Self {
        Self {
            alpha: -MATE,
            beta: MATE,
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
}

impl ops::Neg for AlphaBeta {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            alpha: -self.beta,
            beta: -self.alpha,
        }
    }
}
