use chess::ChessMove;

use crate::evaluate::{score_mark, Score};
use crate::search::{Depth, DEPTH_EDGE, DEPTH_LEAF};

use super::best_moves::BestMoves;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TTableEntry {
    ExactNode(Depth, BestMoves),
    LowerNode(Depth, BestMoves),
    UpperNode(Depth, BestMoves),
    Edge(Score),
    Leaf(Score),
}

impl TTableEntry {
    pub fn update(&mut self, result: TTableEntry) -> &mut Self {
        if *self <= result {
            *self = result;
        }

        self
    }

    pub fn score(&self) -> Score {
        match self {
            TTableEntry::ExactNode(_, moves) => moves.score(),
            TTableEntry::LowerNode(_, moves) => moves.score(),
            TTableEntry::UpperNode(_, moves) => moves.score(),
            TTableEntry::Edge(score) => *score,
            TTableEntry::Leaf(score) => *score,
        }
    }

    pub fn mark(&self) -> Result<Self, ()> {
        Ok(match self {
            TTableEntry::ExactNode(depth, moves) => TTableEntry::ExactNode(*depth, moves.marked()),
            TTableEntry::LowerNode(depth, moves) => TTableEntry::LowerNode(*depth, moves.marked()),
            TTableEntry::UpperNode(depth, moves) => TTableEntry::UpperNode(*depth, moves.marked()),
            TTableEntry::Edge(score) => TTableEntry::Edge(score_mark(*score)),
            TTableEntry::Leaf(score) => TTableEntry::Leaf(score_mark(*score)),
        })
    }

    pub fn depth(&self) -> Depth {
        match self {
            TTableEntry::ExactNode(depth, _) => *depth,
            TTableEntry::LowerNode(depth, _) => *depth,
            TTableEntry::UpperNode(depth, _) => *depth,
            TTableEntry::Edge(..) => DEPTH_EDGE,
            TTableEntry::Leaf(..) => DEPTH_LEAF,
        }
    }

    pub fn peek(&self) -> Option<ChessMove> {
        match self {
            TTableEntry::ExactNode(_, moves) => moves.peek(),
            TTableEntry::LowerNode(_, moves) => moves.peek(),
            TTableEntry::UpperNode(_, moves) => moves.peek(),
            TTableEntry::Edge(..) => None,
            TTableEntry::Leaf(..) => None,
        }
    }
}

impl TTableEntry {
    fn bound_order(&self) -> u8 {
        match self {
            TTableEntry::Edge(..) => 4,

            TTableEntry::ExactNode(..) => 3,
            TTableEntry::Leaf(..) => 2,

            TTableEntry::LowerNode(..) => 1,
            TTableEntry::UpperNode(..) => 0,
        }
    }
}

impl PartialOrd for TTableEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TTableEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let bound_cmp = self.bound_order().cmp(&other.bound_order());
        let depth_cmp = || self.depth().cmp(&other.depth());
        let score_cmp = || self.score().cmp(&other.score());

        bound_cmp.then_with(depth_cmp).then_with(score_cmp)
    }
}
