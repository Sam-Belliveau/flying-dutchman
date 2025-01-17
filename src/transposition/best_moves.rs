use chess::ChessMove;

use crate::evaluate::{Score, MATE};
use crate::transposition::rated_move::RatedMove;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BestMoves {
    Empty,
    Best1(RatedMove),
    Best2(RatedMove, RatedMove),
    Best3(RatedMove, RatedMove, RatedMove),
}

impl BestMoves {
    pub fn new() -> Self {
        Self::Empty
    }

    pub fn contains(&self, mv: ChessMove) -> bool {
        match self {
            BestMoves::Empty => false,
            BestMoves::Best1(b1) => mv == b1.mv,
            BestMoves::Best2(b1, b2) => mv == b1.mv || mv == b2.mv,
            BestMoves::Best3(b1, b2, b3) => mv == b1.mv || mv == b2.mv || mv == b3.mv,
        }
    }

    pub fn push(&mut self, new: RatedMove) {
        debug_assert!(!self.contains(new.mv));

        *self = match *self {
            Self::Empty => Self::Best1(new),
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
        }
    }

    pub fn pop(&mut self) -> Option<RatedMove> {
        match *self {
            BestMoves::Empty => None,
            BestMoves::Best1(best) => {
                *self = BestMoves::Empty;
                Some(best)
            }
            BestMoves::Best2(best, n1) => {
                *self = BestMoves::Best1(n1);
                Some(best)
            }
            BestMoves::Best3(best, n1, n2) => {
                *self = BestMoves::Best2(n1, n2);
                Some(best)
            }
        }
    }

    pub fn peek(&self) -> Option<ChessMove> {
        match *self {
            BestMoves::Empty => None,
            BestMoves::Best1(best, ..) => Some(best.mv),
            BestMoves::Best2(best, ..) => Some(best.mv),
            BestMoves::Best3(best, ..) => Some(best.mv),
        }
    }

    pub fn score(&self) -> Score {
        match self {
            Self::Empty => -MATE,
            Self::Best1(b1, ..) => b1.score,
            Self::Best2(b1, ..) => b1.score,
            Self::Best3(b1, ..) => b1.score,
        }
    }

    pub fn marked(&self) -> Self {
        match self {
            Self::Empty => Self::Empty,
            Self::Best1(b1, ..) => Self::Best1(b1.mark()),
            Self::Best2(b1, b2, ..) => Self::Best2(b1.mark(), b2.mark()),
            Self::Best3(b1, b2, b3, ..) => Self::Best3(b1.mark(), b2.mark(), b3.mark()),
        }
    }
}

impl BestMoves {
    pub fn is_none(&self) -> bool {
        matches!(self, Self::Empty)
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
        self.pop().map(|rated| rated.mv)
    }
}
