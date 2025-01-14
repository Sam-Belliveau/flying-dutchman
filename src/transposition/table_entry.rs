use chess::ChessMove;

use crate::{
    evaluate::{score_mark, Score},
    search::{Depth, DEPTH_EDGE},
};

use super::best_moves::BestMoves;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TTableEntry {
    Node(Depth, BestMoves),
    Edge(Score),
}

use TTableEntry::*;

impl TTableEntry {
    pub fn new(depth: Depth, moves: BestMoves) -> TTableEntry {
        TTableEntry::Node(depth.max(0), moves)
    }

    pub fn edge(score: Score) -> TTableEntry {
        TTableEntry::Edge(score)
    }

    pub fn update(&mut self, result: TTableEntry) {
        match (*self, result) {
            (Node(depth, moves), Node(new_depth, new_moves)) => {
                if depth
                    .cmp(&new_depth)
                    .then(moves.score().cmp(&new_moves.score()))
                    .is_lt()
                {
                    *self = result;
                }
            }
            (Node(..), Edge(..)) => { *self = result; }
            (Edge(..), Node(..)) => {}
            (Edge(score), Edge(new_score)) => {
                if score < new_score {
                    *self = result;
                }
            }
        }
    }

    pub fn score(&self) -> Score {
        match self {
            TTableEntry::Node(_, moves) => moves.score(),
            TTableEntry::Edge(score) => *score,
        }
    }

    pub fn neg_score(&self) -> Score {
        match self {
            TTableEntry::Node(_, moves) => -moves.score(),
            TTableEntry::Edge(score) => -*score,
        }
    }

    pub fn mark(&self) -> Result<Self, ()> {
        Ok(match self {
            TTableEntry::Node(depth, moves) => TTableEntry::Node(*depth, moves.marked()),
            TTableEntry::Edge(score) => TTableEntry::Edge(score_mark(*score)),
        })
    }

    pub fn depth(&self) -> Depth {
        match self {
            TTableEntry::Node(depth, ..) => *depth,
            TTableEntry::Edge(_) => DEPTH_EDGE,
        }
    }

    pub fn moves(&self) -> Option<BestMoves> {
        match self {
            TTableEntry::Node(_, moves) => Some(*moves),
            TTableEntry::Edge(_) => None,
        }
    }

    pub fn peek(&self) -> Option<ChessMove> {
        match self {
            TTableEntry::Node(_, moves) => moves.peek(),
            TTableEntry::Edge(_) => None,
        }
    }
}
