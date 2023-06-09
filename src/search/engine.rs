use std::time::Instant;

use chess::{Board, ChessMove, Color, EMPTY};

use crate::evaluate::{evaluate, score_mark, Score};
use crate::transposition::best_moves::{BestMoves, RatedMove};
use crate::transposition::pv_line::PVLine;
use crate::transposition::table::{TTable, TTableSample, TTableType::*};
use crate::transposition::table_entry::TTableEntry;

use super::alpha_beta::{AlphaBeta, NegaMaxResult::*, ProbeResult::*};
use super::deadline::Deadline;
use super::movegen::OrderedMoveGen;
use super::Depth;

const DEFAULT_TABLE_SIZE: usize = 4000 * 1000 * 1000;

pub struct Engine {
    pub table: TTable,
    nodes: usize,
    side: Color,
}

impl Engine {
    pub fn new() -> Engine {
        Engine {
            table: TTable::new(DEFAULT_TABLE_SIZE),
            nodes: 0,
            side: Color::White,
        }
    }

    fn wrap(score: Score) -> Result<Score, ()> {
        Ok(score_mark(score))
    }

    pub fn ab_qsearch(&mut self, board: Board, mut window: AlphaBeta) -> Result<Score, ()> {
        match self.table.sample::<true>(&board, &window, Depth::MIN) {
            TTableSample::Score(score) => return Self::wrap(score),
            _ => {}
        };

        let start_alpha = window.alpha;

        let movegen = {
            if *board.checkers() == EMPTY {
                let score = evaluate(&board);
                if let Pruned { .. } = window.negamax(score) {
                    return Self::wrap(score);
                }

                OrderedMoveGen::quiescence_search(&board)
            } else {
                OrderedMoveGen::full_search(&board, BestMoves::new())
            }
        };

        for movement in movegen {
            let new_board = board.make_move_new(movement);
            let score = -self.ab_qsearch(new_board, -window)?;

            if let Pruned { .. } = window.negamax(score) {
                return Self::wrap(score);
            }
        }

        if start_alpha < window.alpha {
            self.table
                .update(Exact, board, TTableEntry::leaf(window.alpha));
        }

        Self::wrap(window.alpha)
    }

    fn ab_search(
        &mut self,
        board: Board,
        depth: Depth,
        mut window: AlphaBeta,
        deadline: &Deadline,
    ) -> Result<Score, ()> {
        self.nodes += 1;

        if depth <= 0 {
            return self.ab_qsearch(board, AlphaBeta::new());
        }

        let pv = match self.table.sample::<false>(&board, &window, depth) {
            TTableSample::Moves(moves) => Some(moves),
            TTableSample::Score(score) => return Self::wrap(score),
            TTableSample::None => None,
        };

        if deadline.passed() {
            return Err(());
        }

        let mut moves = BestMoves::new();
        for movement in OrderedMoveGen::full_search(&board, pv.unwrap_or_default()) {
            let next = board.make_move_new(movement);

            let eval = if moves.is_some() {
                let eval = -self.ab_search(next, depth - 1, -(window.null_window()), deadline)?;

                if let Contained { .. } = window.probe(eval) {
                    eval.max(-self.ab_search(
                        next,
                        depth - 1,
                        -(window.raise_alpha(eval)),
                        deadline,
                    )?)
                } else {
                    eval
                }
            } else {
                -self.ab_search(next, depth - 1, -window, deadline)?
            };

            moves.push(RatedMove::new(eval, movement));
            if let Pruned { .. } = window.negamax(eval) {
                let entry = TTableEntry::new(depth, moves);
                self.table.update(Lower, board, entry);

                return Self::wrap(eval);
            }
        }

        let eval = moves.score();

        let entry = TTableEntry::new(depth, moves);
        self.table.update(window.table_type(eval), board, entry);

        Self::wrap(eval)
    }

    pub fn min_search(&mut self, board: &Board) -> TTableEntry {
        if let Some(result) = self.table.get(Exact, *board) {
            if result.moves.is_some() || result.is_edge() {
                return result.clone();
            }
        }

        self.ab_search(*board, 1, AlphaBeta::new(), &Deadline::none())
            .expect("Expected Complete Search");

        return self.table.get(Exact, *board).unwrap().clone();
    }

    pub fn get_pv_line(&mut self, board: Board) -> PVLine {
        self.table.get_pv_line(board)
    }

    pub fn iterative_deepening_search(
        &mut self,
        board: &Board,
        deadline: &Deadline,
    ) -> Result<Score, ()> {
        self.side = board.side_to_move();

        let previous = self.min_search(board);
        let depth = previous.depth + 1;

        self.ab_search(*board, depth, AlphaBeta::new(), deadline)
    }

    pub fn best_move(&mut self, board: &Board) -> Option<ChessMove> {
        self.min_search(board).moves.peek()
    }

    pub fn start_new_search(&mut self) -> Instant {
        self.nodes = 0;
        Instant::now()
    }

    pub fn get_node_count(&self) -> usize {
        self.nodes
    }

    pub fn memory_bytes(&self) -> usize {
        self.table.memory_bytes()
    }
}
