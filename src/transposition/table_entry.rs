use chess::ChessMove;

use crate::{
    evaluate::{score_mark, Score},
    search::{Depth, DEPTH_EDGE},
};

use super::best_moves::BestMoves;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TTableEntry {
    Node(Depth, BestMoves),
    Edge(Depth, Score),
}

use TTableEntry::*;

impl TTableEntry {
    pub fn new(depth: Depth, moves: BestMoves) -> TTableEntry {
        TTableEntry::Node(depth.max(0), moves)
    }

    pub fn new_score(depth: Depth, score: Score) -> TTableEntry {
        TTableEntry::Edge(depth.max(0), score)
    }

    pub fn edge(score: Score) -> TTableEntry {
        TTableEntry::Edge(DEPTH_EDGE, score)
    }

    pub fn update(&mut self, result: TTableEntry) -> &mut Self {
        let (depth, score, new_depth, new_score) = match (*self, result) {
            (Node(depth, moves), Node(new_depth, new_moves)) => {
                (depth, moves.score(), new_depth, new_moves.score())
            }
            (Node(depth, moves), Edge(new_depth, new_score)) => {
                (depth, moves.score(), new_depth, new_score)
            }
            (Edge(depth, score), Node(new_depth, new_moves)) => {
                (depth, score, new_depth, new_moves.score())
            }
            (Edge(depth, score), Edge(new_depth, new_score)) => {
                (depth, score, new_depth, new_score)
            }
        };

        if depth.cmp(&new_depth).then(score.cmp(&new_score)) == std::cmp::Ordering::Less {
            *self = result;
        }

        self
    }

    pub fn score(&self) -> Score {
        match self {
            TTableEntry::Node(_, moves) => moves.score(),
            TTableEntry::Edge(_, score) => *score,
        }
    }

    pub fn mark(&self) -> Result<Self, ()> {
        Ok(match self {
            TTableEntry::Node(depth, moves) => TTableEntry::Node(*depth, moves.marked()),
            TTableEntry::Edge(depth, score) => TTableEntry::Edge(*depth, score_mark(*score)),
        })
    }

    pub fn depth(&self) -> Depth {
        match self {
            TTableEntry::Node(depth, ..) => *depth,
            TTableEntry::Edge(depth, ..) => *depth,
        }
    }

    pub fn moves(&self) -> Option<BestMoves> {
        match self {
            TTableEntry::Node(_, moves) => Some(*moves),
            TTableEntry::Edge(..) => None,
        }
    }

    pub fn peek(&self) -> Option<ChessMove> {
        match self {
            TTableEntry::Node(_, moves) => moves.peek(),
            TTableEntry::Edge(..) => None,
        }
    }
}
