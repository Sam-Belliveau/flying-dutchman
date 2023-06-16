use std::collections::HashSet;
use std::time::Instant;

use chess::{Board, ChessMove, EMPTY};

use crate::evaluate::{evaluate, score_mark, Score, DRAW, MATE};
use crate::transposition::best_moves::{BestMoves, RatedMove};
use crate::transposition::pv_line::PVLine;
use crate::transposition::table::{TTable, TTableSample, TTableType::*};
use crate::transposition::table_entry::TTableEntry;

use super::alpha_beta::{AlphaBeta, NegaMaxResult::*};
use super::deadline::Deadline;
use super::movegen::OrderedMoveGen;
use super::Depth;

const DEFAULT_TABLE_SIZE: usize = 4000 * 1000 * 1000;

pub struct Engine {
    pub table: TTable,
    prev_boards: HashSet<Board>,
    nodes: usize,
}

impl Engine {
    pub fn new() -> Engine {
        Engine {
            table: TTable::new(DEFAULT_TABLE_SIZE),
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

    pub fn ab_qsearch(board: Board, mut window: AlphaBeta, opponent: bool) -> Result<Score, ()> {
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
            let eval = -Self::ab_qsearch(new_board, -window, !opponent)?;

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
            return Self::ab_qsearch(board, window, opponent);
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
            let eval = if PV && moves.is_none() {
                -self.ab_search::<PV>(next, depth - 1, -window, !opponent, deadline)?
            } else {
                -self.ab_search::<false>(next, depth - 1, -window, !opponent, deadline)?
            };

            moves.push(RatedMove::new(eval, movement));
            let score = moves.get_score(opponent);
            if let Pruned { .. } = window.negamax(score) {
                let entry = TTableEntry::new(depth, moves);
                self.table.update::<PV>(Lower, board, entry);
                return Self::wrap(score);
            }
        }

        if moves.is_none() {
            let check = *board.checkers() != EMPTY;
            let eval = if check { -MATE } else { -DRAW };

            let entry = TTableEntry::edge(BestMoves::Static(eval));
            self.table.update::<PV>(Exact, board, entry);
            Self::wrap(eval)
        } else {
            let entry = TTableEntry::new(depth, moves);

            let exact = window.alpha <= moves.get_score(opponent);
            let ttype = if exact { Exact } else { Upper };

            self.table.update::<PV>(ttype, board, entry);
            Self::wrap(moves.get_score(opponent))
        }
    }

    pub fn min_search(&mut self, board: &Board) -> TTableEntry {
        if let Some(result) = self.table.get(Exact, *board) {
            if !result.moves.is_none() || result.is_edge() {
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

        if previous.is_edge() {
            return Err(());
        }

        let result = self.ab_search::<true>(*board, depth, AlphaBeta::new(), false, deadline);
        if let Ok(score) = result {
            self.table.refresh_pv_line(true);
            Ok(score)
        } else {
            self.table.refresh_pv_line(false);
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
