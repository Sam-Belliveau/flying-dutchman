use chess::ChessMove;

use crate::evaluate::{Score, MATE};

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
    Best1(RatedMove),
    Best2(RatedMove, RatedMove),
    Best3(RatedMove, RatedMove, RatedMove),
}

impl BestMoves {
    pub fn new() -> Self {
        Self::Static(-MATE)
    }

    pub fn contains(&self, mv: ChessMove) -> bool {
        match self {
            BestMoves::Static(..) => false,
            BestMoves::Best1(b1) => mv == b1.mv,
            BestMoves::Best2(b1, b2) => mv == b1.mv || mv == b2.mv,
            BestMoves::Best3(b1, b2, b3) => mv == b1.mv || mv == b2.mv || mv == b3.mv,
        }
    }

    pub fn push(&mut self, new: RatedMove) {
        if !self.contains(new.mv) {
            *self = match *self {
                Self::Static(score) => {
                    if new.score <= score {
                        *self
                    } else {
                        Self::Best1(new)
                    }
                }
                Self::Best1(b1) => {
                    if new.score <= b1.score {
                        Self::Best2(b1, new)
                    } else {
                        Self::Best2(new, b1)
                    }
                }
                Self::Best2(b1, b2) => {
                    if new.score <= b2.score {
                        Self::Best3(b1, b2, new)
                    } else if new.score <= b1.score {
                        Self::Best3(b1, new, b2)
                    } else {
                        Self::Best3(new, b1, b2)
                    }
                }
                Self::Best3(b1, b2, b3) => {
                    if new.score <= b3.score {
                        *self
                    } else if new.score <= b2.score {
                        Self::Best3(b1, b2, new)
                    } else if new.score <= b1.score {
                        Self::Best3(b1, new, b2)
                    } else {
                        Self::Best3(new, b1, b2)
                    }
                }
            };
        } else {
            panic!();
        }
    }

    pub fn pop(&mut self) -> Option<ChessMove> {
        match *self {
            BestMoves::Static(..) => None,
            BestMoves::Best1(best) => {
                *self = BestMoves::new();
                Some(best.mv)
            }
            BestMoves::Best2(best, n1) => {
                *self = BestMoves::Best1(n1);
                Some(best.mv)
            }
            BestMoves::Best3(best, n1, n2) => {
                *self = BestMoves::Best2(n1, n2);
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
        }
    }

    pub fn best_score(&self) -> Score {
        match self {
            Self::Static(score) => *score,
            Self::Best1(b1, ..) | Self::Best2(b1, ..) | Self::Best3(b1, ..) => b1.score,
        }
    }

    pub fn avg_score(&self) -> Score {
        match self {
            Self::Static(score) => *score,
            Self::Best1(b1) => b1.score,
            Self::Best2(b1, b2) => (b1.score + 2 * b2.score) / 3,
            Self::Best3(b1, b2, b3) => (b1.score + 2 * b2.score + 3 * b3.score) / 6,
        }
    }

    pub fn get_score(&self, opponent: bool) -> Score {
        // This boolean controls whether or not we assume
        // that the opponent will play the best move
        const NORMAL: bool = false;

        if !NORMAL && opponent {
            self.avg_score()
        } else {
            self.best_score()
        }
    }
}

impl BestMoves {
    pub fn is_some(&self) -> bool {
        match self {
            Self::Static(..) => false,
            _ => true,
        }
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
