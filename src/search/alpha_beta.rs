use std::ops;

use crate::evaluate::{Score, MATE_CUTOFF, MATE_MOVE, MATE};

pub enum NegaMaxResult {
    Worse { delta: Score },
    Best,
    Pruned { beta: Score },
}

pub enum ProbeResult {
    AlphaPrune { alpha: Score },
    Contained,
    BetaPrune { beta: Score },
}

#[derive(Clone, Copy, Debug)]
pub struct AlphaBeta {
    alpha: Score,
    beta: Score,
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
        } else if self.alpha < score {
            self.alpha = score;
            NegaMaxResult::Best
        } else {
            NegaMaxResult::Worse { delta: self.alpha - score }
        }
    }

    pub fn probe(&self, score: Score) -> ProbeResult {
        if score <= self.alpha {
            ProbeResult::AlphaPrune { alpha: self.alpha }
        } else if score >= self.beta {
            ProbeResult::BetaPrune { beta: self.beta }
        } else {
            ProbeResult::Contained
        }
    }

    pub fn raise_min(&self, score: Score) -> AlphaBeta {
        AlphaBeta {
            alpha: self.alpha.max(score),
            beta: self.beta,
        }
    }

    pub fn null_window(&self) -> AlphaBeta {
        AlphaBeta {
            alpha: self.alpha,
            beta: self.alpha + 1,
        }
    }

    pub fn alpha(&self) -> Score {
        if self.alpha.abs() >= MATE_CUTOFF {
            self.alpha - MATE_MOVE * self.alpha.signum()
        } else {
            self.alpha
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
