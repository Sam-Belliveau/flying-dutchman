use std::ops;

use crate::{
    evaluate::{Score, MATE},
    transposition::{table::TTableType, best_moves::BestMoves},
};

pub enum NegaMaxResult {
    Worse { delta: Score },
    Best { score: Score },
    Pruned { beta: Score },
}

pub enum ProbeResult {
    AlphaPrune { alpha: Score },
    Contained { score: Score },
    BetaPrune { beta: Score },
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
            NegaMaxResult::Pruned { beta: self.beta }
        } else if self.alpha <= score {
            self.alpha = score;
            NegaMaxResult::Best { score }
        } else {
            NegaMaxResult::Worse {
                delta: self.alpha - score,
            }
        }
    }

    pub fn probe(&self, score: Score) -> ProbeResult {
        if score <= self.alpha {
            ProbeResult::AlphaPrune { alpha: self.alpha }
        } else if score >= self.beta {
            ProbeResult::BetaPrune { beta: self.beta }
        } else {
            ProbeResult::Contained { score }
        }
    }

    pub fn table_type(&self, moves: &BestMoves) -> TTableType {
        let score = moves.best_score();
        if score < self.alpha {
            TTableType::Upper
        } else if score > self.beta {
            TTableType::Lower
        } else {
            TTableType::Exact
        }
    }

    pub fn null_window(&self) -> AlphaBeta {
        AlphaBeta {
            alpha: self.alpha,
            beta: self.alpha + 1,
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
