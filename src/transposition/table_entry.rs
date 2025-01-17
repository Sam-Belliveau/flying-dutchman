use chess::ChessMove;

use crate::evaluate::{score_mark, Score};
use crate::search::{Depth, DEPTH_EDGE, DEPTH_LEAF};

use super::best_moves::BestMoves;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TTableEntry {
    Node(Depth, BestMoves),
    Edge(Score),
    Leaf(Score),
}

impl TTableEntry {
    pub fn update(&mut self, result: TTableEntry) -> &mut Self {
        let depth_cmp = self.depth().cmp(&result.depth());
        let score_cmp = self.score().cmp(&result.score());

        if depth_cmp.then(score_cmp).is_lt() {
            *self = result;
        }

        self
    }

    pub fn score(&self) -> Score {
        match self {
            TTableEntry::Node(_, moves) => moves.score(),
            TTableEntry::Edge(score) => *score,
            TTableEntry::Leaf(score) => *score,
        }
    }

    pub fn mark(&self) -> Result<Self, ()> {
        Ok(match self {
            TTableEntry::Node(depth, moves) => TTableEntry::Node(*depth, moves.marked()),
            TTableEntry::Edge(score) => TTableEntry::Edge(score_mark(*score)),
            TTableEntry::Leaf(score) => TTableEntry::Leaf(score_mark(*score)),
        })
    }

    pub fn depth(&self) -> Depth {
        match self {
            TTableEntry::Node(depth, ..) => *depth,
            TTableEntry::Edge(..) => DEPTH_EDGE,
            TTableEntry::Leaf(..) => DEPTH_LEAF,
        }
    }

    pub fn moves(&self) -> Option<BestMoves> {
        match self {
            TTableEntry::Node(_, moves) => Some(*moves),
            TTableEntry::Edge(..) => None,
            TTableEntry::Leaf(..) => None,
        }
    }

    pub fn peek(&self) -> Option<ChessMove> {
        match self {
            TTableEntry::Node(_, moves) => moves.peek(),
            TTableEntry::Edge(..) => None,
            TTableEntry::Leaf(..) => None,
        }
    }
}
