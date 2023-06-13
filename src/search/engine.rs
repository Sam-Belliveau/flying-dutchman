use std::time::Instant;

use chess::{Board, ChessMove, EMPTY};

use crate::evaluate::{evaluate, score_mark, Score, DRAW, MATE};
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
}

impl Engine {
    pub fn new() -> Engine {
        Engine {
            table: TTable::new(DEFAULT_TABLE_SIZE),
            nodes: 0,
        }
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
        let pv = match self
            .table
            .sample::<true>(&board, &window, false, Depth::MIN)
        {
            TTableSample::Score(score) => return Self::wrap(score),
            TTableSample::Moves(pv) => pv,
            _ => BestMoves::new(),
        };

        let start_alpha = window.alpha;

        let movegen = {
            if *board.checkers() == EMPTY {
                let score = evaluate(&board);
                if let Pruned { .. } = window.negamax(score) {
                    return Self::wrap(score);
                }

                OrderedMoveGen::quiescence_search(&board, pv)
            } else {
                OrderedMoveGen::full_search(&board, pv)
            }
        };

        let mut moves = BestMoves::Static(window.alpha);
        for movement in movegen {
            let new_board = board.make_move_new(movement);
            let eval = -self.ab_qsearch(new_board, -window, !opponent)?;

            moves.push(RatedMove::new(eval, movement));
            let score = moves.get_score(opponent);
            if let Pruned { .. } = window.negamax(score) {
                return Self::wrap(score);
            }
        }

        let eval = moves.get_score(opponent);
        if start_alpha < eval {
            self.table.update(Exact, board, TTableEntry::leaf(moves));
        }

        Self::wrap(window.alpha)
    }

    fn ab_search(
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

        let pv = match self.table.sample::<false>(&board, &window, opponent, depth) {
            TTableSample::Moves(moves) => moves,
            TTableSample::Score(score) => return Self::wrap(score),
            TTableSample::None => BestMoves::new(),
        };

        if deadline.passed() {
            return Err(());
        }

        let mut moves = BestMoves::new();
        for movement in OrderedMoveGen::full_search(&board, pv) {
            match {
                let next = board.make_move_new(movement);

                if moves.is_some() {
                    self.ab_search(
                        next,
                        depth - 1,
                        -(window.null_window()),
                        !opponent,
                        deadline,
                    )
                    .map(|eval| -eval)
                    .and_then(|eval| {
                        if let Contained { .. } = window.probe(eval) {
                            self.ab_search(next, depth - 1, -window, !opponent, deadline)
                                .map(|eval| -eval)
                        } else {
                            Ok(eval)
                        }
                    })
                } else {
                    self.ab_search(next, depth - 1, -window, !opponent, deadline)
                        .map(|eval| -eval)
                }
            } {
                Ok(eval) => {
                    moves.push(RatedMove::new(eval, movement));
                    let score = moves.get_score(opponent);
                    if let Pruned { .. } = window.negamax(score) {
                        let entry = TTableEntry::new(depth, moves);
                        self.table.update(Lower, board, entry);

                        return Self::wrap(score);
                    }
                }
                Err(()) => {
                    // If we receive Err(()), it means that the deadline has struck
                    // and we are unwinding the call stack. We need to save some work
                    // if it has been pushed out of the transposition table.
                    self.table.update(Exact, board, TTableEntry::leaf(pv));

                    return Err(());
                }
            }
        }

        if moves.is_some() {
            let entry = TTableEntry::new(depth, moves);
            self.table.update(window.table_type(&moves), board, entry);

            Self::wrap(moves.get_score(opponent))
        } else {
            let eval = if *board.checkers() == EMPTY {
                -DRAW
            } else {
                -MATE
            };

            let entry = TTableEntry::edge(BestMoves::Static(eval));
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

        self.ab_search(*board, 1, AlphaBeta::new(), false, &Deadline::none())
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
        let previous = self.min_search(board);
        let depth = previous.depth + 1;

        // if depth > 6 {
        //     return Err(());
        // }

        self.ab_search(*board, depth, AlphaBeta::new(), false, deadline)
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
