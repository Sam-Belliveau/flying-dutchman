use chess::ChessMove;

use crate::{
    evaluate::Score,
    search::{Depth, DEPTH_EDGE},
};

use super::best_moves::BestMoves;

#[derive(Copy, Clone, Debug)]
pub enum TTableEntry {
    Node(Depth, BestMoves),
    Edge(Score),
}

impl TTableEntry {
    pub fn new(depth: Depth, moves: BestMoves) -> TTableEntry {
        TTableEntry::Node(depth.max(0), moves)
    }

    pub fn edge(score: Score) -> TTableEntry {
        TTableEntry::Edge(score)
    }

    pub fn update(&mut self, result: TTableEntry) {
        match *self {
            TTableEntry::Node(depth, ..) => {
                if let TTableEntry::Node(new_depth, ..) = result {
                    if depth
                        .cmp(&new_depth)
                        .then(self.score().cmp(&result.score()))
                        .is_lt()
                    {
                        *self = result;
                    }
                }
            }
            TTableEntry::Edge(score) => {
                if let TTableEntry::Edge(new_score) = result {
                    if score < new_score {
                        *self = result;
                    }
                }
            }
        }
    }

    pub fn update_upper(&mut self, result: TTableEntry) {
        match *self {
            TTableEntry::Node(depth, ..) => {
                if let TTableEntry::Node(new_depth, ..) = result {
                    if depth
                        .cmp(&new_depth)
                        .then(self.score().cmp(&result.score()))
                        .is_lt()
                    {
                        *self = result
                    }
                }
            }
            TTableEntry::Edge(score) => {
                if let TTableEntry::Edge(new_score) = result {
                    if score > new_score {
                        *self = result;
                    }
                }
            }
        }
    }

    pub fn score(&self) -> Score {
        match self {
            TTableEntry::Node(_, moves) => moves.best_score(),
            TTableEntry::Edge(score) => *score,
        }
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
