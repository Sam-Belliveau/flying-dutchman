use std::time::Instant;

use chess::{Board, ChessMove, Color};

use crate::evaluate::{score_mark, Score};
use crate::search::qsearch::*;
use crate::transposition::best_moves::{BestMoves, RatedMove};
use crate::transposition::pv_line::PVLine;
use crate::transposition::table::{TTable, TTableType::*};
use crate::transposition::table_entry::TTableEntry;

use super::alpha_beta::{AlphaBeta, NegaMaxResult::*, ProbeResult::*};
use super::deadline::Deadline;
use super::movegen::OrderedMoveGen;
use super::Depth;

const DEFAULT_TABLE_SIZE: usize = 3000 * 1000 * 1000;

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

    fn ab_search(
        &mut self,
        board: Board,
        depth: Depth,
        mut window: AlphaBeta,
        deadline: &Deadline,
    ) -> Result<Score, ()> {
        self.nodes += 1;

        let mut pv = None;

        if let Some(saved) = self.table.get(Exact, board) {
            pv = pv.or(Some(saved.moves));
            if depth <= saved.depth {
                return Self::wrap(saved.score());
            }
        }

        if let Some(saved) = self.table.get(Lower, board) {
            pv = pv.or(Some(saved.moves));
            if depth <= saved.depth {
                let score = saved.score();
                if let Pruned { .. } = window.negamax(score) {
                    return Self::wrap(saved.score());
                }
            }
        }

        if let Some(saved) = self.table.get(Upper, board) {
            pv = pv.or(Some(saved.moves));
            if depth <= saved.depth {
                let score = saved.score();
                if let AlphaPrune { .. } = window.probe(score) {
                    return Self::wrap(saved.score());
                }
            }
        }

        if depth <= 0 {
            let eval = ab_qsearch(board, window);

            self.table
                .update(window.table_type(eval), board, TTableEntry::leaf(eval));

            return Self::wrap(eval);
        }

        if deadline.passed() {
            return Err(());
        }

        let mut moves = BestMoves::new();
        for movement in OrderedMoveGen::full_search(&board, pv.unwrap_or_default()) {
            let next = board.make_move_new(movement);
            let eval = -self.ab_search(next, depth - 1, -window, deadline)?;

            moves.push(RatedMove::new(eval, movement));
            if let Pruned { .. } = window.negamax(eval) {
                let entry = TTableEntry::new(depth, moves);
                self.table.update(Lower, board, entry);

                return Self::wrap(eval);
            }
        }

        let score = moves.score();

        let entry = TTableEntry::new(depth, moves);
        self.table.update(window.table_type(score), board, entry);

        Self::wrap(score)
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

        self.table.refresh_pv_line(*board);
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
