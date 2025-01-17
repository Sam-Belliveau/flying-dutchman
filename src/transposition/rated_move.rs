use chess::ChessMove;

use crate::evaluate::{score_mark, Score};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RatedMove {
    pub score: Score,
    pub mv: ChessMove,
}

impl RatedMove {
    pub fn new(score: Score, mv: ChessMove) -> Self {
        Self { score, mv }
    }

    pub fn mark(&self) -> Self {
        Self {
            score: score_mark(self.score),
            mv: self.mv,
        }
    }
}
