use chess::ChessMove;

use crate::evaluate::{score_mark, Score};
use crate::search::{Depth, DEPTH_EDGE, DEPTH_LEAF};

use super::best_moves::BestMoves;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TTableEntry {
    ExactNode(Depth, BestMoves),
    LowerNode(Depth, BestMoves),
    UpperNode(Depth, BestMoves),
    NullCut(Depth, Score),
    Edge(Score),
    Leaf(Score),
}

impl TTableEntry {
    pub fn score(&self) -> Score {
        match self {
            TTableEntry::ExactNode(_, moves) => moves.score(),
            TTableEntry::LowerNode(_, moves) => moves.score(),
            TTableEntry::UpperNode(_, moves) => moves.score(),
            TTableEntry::NullCut(_, score) => *score,
            TTableEntry::Edge(score) => *score,
            TTableEntry::Leaf(score) => *score,
        }
    }

    pub fn mark(&self) -> Result<Self, ()> {
        Ok(match self {
            TTableEntry::ExactNode(depth, moves) => TTableEntry::ExactNode(*depth, moves.marked()),
            TTableEntry::LowerNode(depth, moves) => TTableEntry::LowerNode(*depth, moves.marked()),
            TTableEntry::UpperNode(depth, moves) => TTableEntry::UpperNode(*depth, moves.marked()),
            TTableEntry::NullCut(depth, score) => TTableEntry::NullCut(*depth, score_mark(*score)),
            TTableEntry::Edge(score) => TTableEntry::Edge(score_mark(*score)),
            TTableEntry::Leaf(score) => TTableEntry::Leaf(score_mark(*score)),
        })
    }

    pub fn depth(&self) -> Depth {
        match self {
            TTableEntry::ExactNode(depth, _) => *depth,
            TTableEntry::LowerNode(depth, _) => *depth,
            TTableEntry::UpperNode(depth, _) => *depth,
            TTableEntry::NullCut(depth, _) => *depth,
            TTableEntry::Edge(..) => DEPTH_EDGE,
            TTableEntry::Leaf(..) => DEPTH_LEAF,
        }
    }

    pub fn moves(&self) -> Option<&BestMoves> {
        match self {
            TTableEntry::ExactNode(_, moves) => Some(moves),
            TTableEntry::LowerNode(_, moves) => Some(moves),
            TTableEntry::UpperNode(_, moves) => Some(moves),
            TTableEntry::NullCut(..) => None,
            TTableEntry::Edge(..) => None,
            TTableEntry::Leaf(..) => None,
        }
    }

    pub fn peek(&self) -> Option<ChessMove> {
        self.moves().and_then(|moves| moves.peek())
    }
}
