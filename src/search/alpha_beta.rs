use std::ops;

use crate::evaluate::{Score, MAX_SCORE, MIN_SCORE};

#[derive(Clone, Copy, Debug)]
pub struct AlphaBeta {
    alpha: Score,
    beta: Score,
}

pub enum NegaMaxResult {
    Worse { score: Score },
    Matches { score: Score },
    NewBest { score: Score },
    BetaPrune { beta: Score },
}

pub enum ProbeResult {
    Alpha { alpha: Score },
    Contained { score: Score },
    Beta { beta: Score },
}

impl AlphaBeta {
    pub fn new() -> Self {
        Self {
            alpha: MIN_SCORE,
            beta: MAX_SCORE,
        }
    }

    pub fn negamax(&mut self, score: Score) -> NegaMaxResult {
        if self.beta <= score {
            self.alpha = score;
            NegaMaxResult::BetaPrune { beta: self.beta }
        } else if self.alpha < score {
            self.alpha = score;
            NegaMaxResult::NewBest { score }
        } else if self.alpha == score {
            NegaMaxResult::Matches { score }
        } else {
            NegaMaxResult::Worse { score }
        }
    }

    pub fn probe(&self, score: Score) -> ProbeResult {
        if score <= self.alpha {
            ProbeResult::Alpha { alpha: self.alpha }
        } else if score >= self.beta {
            ProbeResult::Beta { beta: self.beta }
        } else {
            ProbeResult::Contained { score }
        }
    }

    pub fn null_window(&self) -> AlphaBeta {
        AlphaBeta {
            alpha: self.alpha,
            beta: self.alpha + 1,
        }
    }

    pub fn alpha(&self) -> Score {
        self.alpha
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
