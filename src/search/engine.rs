use std::collections::HashSet;
use std::mem::replace;
use std::time::Instant;

use chess::{Board, ChessMove, EMPTY};

use crate::evaluate::{evaluate, score_mark, Score, DRAW, MATE};
use crate::transposition::best_moves::{BestMoves, RatedMove};
use crate::transposition::pv_line::PVLine;
use crate::transposition::table::{
    TTable, TTableSample,
    TTableType::{self, *},
};
use crate::transposition::table_entry::TTableEntry;

use super::alpha_beta::{AlphaBeta, NegaMaxResult::*};
use super::deadline::Deadline;
use super::movegen::OrderedMoveGen;
use super::Depth;

const DEFAULT_TABLE_SIZE: usize = 4000 * 1000 * 1000;

pub struct Engine {
    pub table: TTable,
    pv_cache: Vec<(TTableType, Board, TTableEntry)>,
    prev_boards: HashSet<Board>,
    nodes: usize,
}

impl Engine {
    pub fn new() -> Engine {
        Engine {
            table: TTable::new(DEFAULT_TABLE_SIZE),
            pv_cache: Vec::new(),
            prev_boards: HashSet::new(),
            nodes: 0,
        }
    }

    pub fn reset(&mut self) {
        self.prev_boards.clear();
    }

    fn wrap(score: Score) -> Result<Score, ()> {
        Ok(score_mark(score))
    }

    pub fn ab_qsearch(
        &mut self,
        board: Board,
        mut window: AlphaBeta,
        opponent: bool,
    ) -> Result<Score, ()> {
        self.nodes += 1;

        let (mut moves, movegen) = {
            if *board.checkers() == EMPTY {
                let score = evaluate(&board);
                if let Pruned { .. } = window.negamax(score) {
                    return Self::wrap(score);
                }

                (
                    BestMoves::Static(score),
                    OrderedMoveGen::quiescence_search(&board, BestMoves::new()),
                )
            } else {
                (
                    BestMoves::Static(-MATE),
                    OrderedMoveGen::full_search(&board, BestMoves::new()),
                )
            }
        };

        for movement in movegen {
            let new_board = board.make_move_new(movement);
            let eval = -self.ab_qsearch(new_board, -window, !opponent)?;

            moves.push(RatedMove::new(eval, movement));
            let score = moves.get_score(opponent);
            if let Pruned { .. } = window.negamax(score) {
                return Self::wrap(score);
            }
        }

        Self::wrap(moves.get_score(opponent))
    }

    fn ab_search<const PV: bool>(
        &mut self,
        board: Board,
        depth: Depth,
        mut window: AlphaBeta,
        opponent: bool,
        deadline: &Deadline,
    ) -> Result<Score, ()> {
        self.nodes += 1;

        if depth <= 0 {
            return self.ab_qsearch(board, AlphaBeta::new(), opponent);
        }

        let pv = match self.table.sample(&board, &window, opponent, depth) {
            TTableSample::Moves(moves) => moves,
            TTableSample::Score(score) => return Self::wrap(score),
            TTableSample::None => BestMoves::new(),
        };

        if deadline.passed() {
            return Err(());
        }

        let mut moves = BestMoves::new();
        for movement in OrderedMoveGen::full_search(&board, pv) {
            let next = board.make_move_new(movement);
            let eval = if moves.is_some() {
                -self.ab_search::<false>(next, depth - 1, -window, !opponent, deadline)?
            } else {
                -self.ab_search::<PV>(next, depth - 1, -window, !opponent, deadline)?
            };

            moves.push(RatedMove::new(eval, movement));
            let score = moves.get_score(opponent);
            if let Pruned { .. } = window.negamax(score) {
                let entry = TTableEntry::new(depth, moves);

                if PV {
                    self.pv_cache.push((Lower, board, entry.clone()));
                }

                self.table.update(Lower, board, entry);

                return Self::wrap(score);
            }
        }

        if moves.is_some() {
            let entry = TTableEntry::new(depth, moves);
            let ttype = window.table_type(&moves);

            if PV {
                self.pv_cache.push((ttype, board, entry.clone()));
            }

            self.table.update(ttype, board, entry);
            Self::wrap(moves.get_score(opponent))
        } else {
            let eval = if *board.checkers() == EMPTY {
                -DRAW
            } else {
                -MATE
            };

            let entry = TTableEntry::edge(BestMoves::Static(eval));

            if PV {
                self.pv_cache.push((Exact, board, entry.clone()));
            }

            self.table.update(Exact, board, entry);
            Self::wrap(eval)
        }
    }

    pub fn min_search(&mut self, board: &Board) -> TTableEntry {
        if let Some(result) = self.table.get(Exact, *board) {
            if result.moves.is_some() || result.is_edge() {
                return result.clone();
            }
        }

        self.ab_search::<false>(*board, 1, AlphaBeta::new(), false, &Deadline::none())
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
        self.prev_boards.insert(*board);
        let previous = self.min_search(board);
        let depth = previous.depth + 1;

        if depth > 7 {
            return Err(());
        }

        let old_pv = replace(&mut self.pv_cache, Vec::new());

        if let Ok(score) = self.ab_search::<true>(*board, depth, AlphaBeta::new(), false, deadline)
        {
            for (s, b, e) in &self.pv_cache {
                self.table.update(*s, *b, e.clone());
            }

            Ok(score)
        } else {
            for (s, b, e) in replace(&mut self.pv_cache, old_pv) {
                self.table.update(s, b, e);
            }

            for (s, b, e) in &self.pv_cache {
                self.table.update(*s, *b, e.clone());
            }

            self.table.update(Exact, *board, previous);
            Err(())
        }
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
