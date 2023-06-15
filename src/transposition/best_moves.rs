use chess::ChessMove;

use crate::evaluate::{Score, CENTIPAWN, MATE};

#[derive(Clone, Copy, Debug)]
pub struct RatedMove {
    pub score: Score,
    pub mv: ChessMove,
}

impl RatedMove {
    pub fn new(score: Score, mv: ChessMove) -> Self {
        Self { score, mv }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum BestMoves {
    Static(Score),
    Best1(RatedMove, Score),
    Best2(RatedMove, RatedMove, Score),
    Best3(RatedMove, RatedMove, RatedMove, Score),
    Best4(RatedMove, RatedMove, RatedMove, RatedMove),
}

impl BestMoves {
    pub fn new() -> Self {
        Self::Static(-MATE)
    }

    pub fn contains(&self, mv: ChessMove) -> bool {
        match self {
            BestMoves::Static(..) => false,
            BestMoves::Best1(b1, ..) => mv == b1.mv,
            BestMoves::Best2(b1, b2, ..) => mv == b1.mv || mv == b2.mv,
            BestMoves::Best3(b1, b2, b3, ..) => mv == b1.mv || mv == b2.mv || mv == b3.mv,
            BestMoves::Best4(b1, b2, b3, b4) => {
                mv == b1.mv || mv == b2.mv || mv == b3.mv || mv == b4.mv
            }
        }
    }

    pub fn push(&mut self, new: RatedMove) {
        debug_assert!(!self.contains(new.mv));

        *self = match *self {
            Self::Static(score) => {
                if new.score <= score {
                    *self
                } else {
                    Self::Best1(new, score)
                }
            }
            Self::Best1(b1, score) => {
                if new.score <= score {
                    *self
                } else if new.score <= b1.score {
                    Self::Best2(b1, new, score)
                } else {
                    Self::Best2(new, b1, score)
                }
            }
            Self::Best2(b1, b2, score) => {
                if new.score <= score {
                    *self
                } else if new.score <= b2.score {
                    Self::Best3(b1, b2, new, score)
                } else if new.score <= b1.score {
                    Self::Best3(b1, new, b2, score)
                } else {
                    Self::Best3(new, b1, b2, score)
                }
            }
            Self::Best3(b1, b2, b3, score) => {
                if new.score <= score {
                    *self
                } else if new.score <= b3.score {
                    Self::Best4(b1, b2, b3, new)
                } else if new.score <= b2.score {
                    Self::Best4(b1, b2, new, b3)
                } else if new.score <= b1.score {
                    Self::Best4(b1, new, b2, b3)
                } else {
                    Self::Best4(new, b1, b2, b3)
                }
            }
            Self::Best4(b1, b2, b3, b4) => {
                if new.score <= b4.score {
                    *self
                } else if new.score <= b3.score {
                    Self::Best4(b1, b2, b3, new)
                } else if new.score <= b2.score {
                    Self::Best4(b1, b2, new, b3)
                } else if new.score <= b1.score {
                    Self::Best4(b1, new, b2, b3)
                } else {
                    Self::Best4(new, b1, b2, b3)
                }
            }
        };
    }

    pub fn pop(&mut self) -> Option<ChessMove> {
        match *self {
            BestMoves::Static(..) => None,
            BestMoves::Best1(best, score) => {
                *self = BestMoves::Static(score);
                Some(best.mv)
            }
            BestMoves::Best2(best, n1, score) => {
                *self = BestMoves::Best1(n1, score);
                Some(best.mv)
            }
            BestMoves::Best3(best, n1, n2, score) => {
                *self = BestMoves::Best2(n1, n2, score);
                Some(best.mv)
            }
            BestMoves::Best4(best, n1, n2, n3) => {
                *self = BestMoves::Best3(n1, n2, n3, -MATE);
                Some(best.mv)
            }
        }
    }

    pub fn peek(&self) -> Option<ChessMove> {
        match *self {
            BestMoves::Static(..) => None,
            BestMoves::Best1(best, ..) => Some(best.mv),
            BestMoves::Best2(best, ..) => Some(best.mv),
            BestMoves::Best3(best, ..) => Some(best.mv),
            BestMoves::Best4(best, ..) => Some(best.mv),
        }
    }

    pub fn best_score(&self) -> Score {
        match self {
            Self::Static(score) => *score,
            Self::Best1(b1, ..) => b1.score,
            Self::Best2(b1, ..) => b1.score,
            Self::Best3(b1, ..) => b1.score,
            Self::Best4(b1, ..) => b1.score,
        }
    }

    pub fn min_score(&self, threshold: Score) -> Score {
        match self {
            BestMoves::Static(score) => *score,
            BestMoves::Best1(b1, ..) => b1.score,
            BestMoves::Best2(b1, b2, ..) => {
                if b2.score >= threshold {
                    b2.score
                } else {
                    b1.score
                }
            }
            BestMoves::Best3(b1, b2, b3, ..) => {
                if b3.score >= threshold {
                    b3.score
                } else if b2.score >= threshold {
                    b2.score
                } else {
                    b1.score
                }
            }
            BestMoves::Best4(b1, b2, b3, b4) => {
                if b4.score >= threshold {
                    b4.score
                } else if b3.score >= threshold {
                    b3.score
                } else if b2.score >= threshold {
                    b2.score
                } else {
                    b1.score
                }
            }
        }
    }

    pub fn get_score(&self, opponent: bool) -> Score {
        // This boolean controls whether or not we assume
        // that the opponent will play the best move
        const NORMAL: bool = false;

        if !NORMAL && opponent {
            let threshold = self.best_score() - 0 * CENTIPAWN;
            self.min_score(threshold)
        } else {
            self.best_score()
        }
    }
}

impl BestMoves {
    pub fn is_some(&self) -> bool {
        !(matches!(self, Self::Static(..)))
    }
}

impl Default for BestMoves {
    fn default() -> Self {
        Self::new()
    }
}

impl Iterator for BestMoves {
    type Item = ChessMove;

    fn next(&mut self) -> Option<Self::Item> {
        self.pop()
    }
}
