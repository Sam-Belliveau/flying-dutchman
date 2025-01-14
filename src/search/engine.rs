use std::time::Instant;

use chess::{Board, ChessMove, EMPTY};

use crate::evaluate::{evaluate, score_mark, Score, DRAW, MATE};
use crate::transposition::best_moves::{BestMoves, RatedMove};
use crate::transposition::pv_line::PVLine;
use crate::transposition::table::{TTable, TTableSample, TTableType::*};
use crate::transposition::table_entry::TTableEntry;

use super::alpha_beta::{AlphaBeta, NegaMaxResult::*};
use super::board_history::BoardHistory;
use super::deadline::Deadline;
use super::movegen::OrderedMoveGen;
use super::Depth;

const DEFAULT_TABLE_SIZE: usize = 4000 * 1000 * 1000;

pub struct Engine {
    pub table: TTable,
    depth: Depth,
    nodes: usize,
}

impl Engine {
    pub fn new() -> Engine {
        Engine {
            table: TTable::new(DEFAULT_TABLE_SIZE),
            depth: 0,
            nodes: 0,
        }
    }

    fn wrap(score: Score) -> Result<Score, ()> {
        Ok(score_mark(score))
    }

    pub fn ab_qsearch(board: Board, mut window: AlphaBeta) -> Result<Score, ()> {
        let (mut best, movegen) = {
            if *board.checkers() == EMPTY {
                let score = evaluate(&board);
                if let Pruned = window.negamax(score) {
                    return Self::wrap(score);
                }

                (
                    score,
                    OrderedMoveGen::quiescence_search(&board, BestMoves::new()),
                )
            } else {
                (-MATE, OrderedMoveGen::full_search(&board, BestMoves::new()))
            }
        };

        for movement in movegen {
            let new_board = board.make_move_new(movement);
            let eval = -Self::ab_qsearch(new_board, -window)?;

            best = best.max(eval);
            if let Pruned = window.negamax(eval) {
                return Self::wrap(best);
            }
        }

        Self::wrap(best)
    }

    fn ab_search<const PV: bool>(
        &mut self,
        board: BoardHistory,
        depth: Depth,
        mut window: AlphaBeta,
        deadline: &Deadline,
    ) -> Result<TTableEntry, ()> {
        self.nodes += 1;

        let draw = if board.is_draw() {
            Some(TTableEntry::Edge(if window.opponent() {
                -DRAW
            } else {
                DRAW
            }))
        } else {
            None
        };

        if depth <= 0 {
            return draw
                .unwrap_or(TTableEntry::Edge(Self::ab_qsearch(board.last(), window)?))
                .mark();
        }

        let pv = match self.table.sample(&board.last(), &window, depth) {
            TTableSample::Moves(moves) => moves,
            TTableSample::Score(score) => return score.mark(),
            TTableSample::None => BestMoves::new(),
        };

        if deadline.passed() {
            return Err(());
        }

        let original_alpha = window.alpha;

        let mut moves = BestMoves::new();
        for movement in OrderedMoveGen::full_search(&board.last(), pv) {
            let next = board.with_move(movement);
            let eval = if PV && moves.is_none() {
                self.ab_search::<PV>(next, depth - 1, -window, deadline)?
                    .neg_score()
            } else {
                self.ab_search::<false>(next, depth - 1, -window, deadline)?
                    .neg_score()
            };

            moves.push(RatedMove::new(eval, movement));

            let score = moves.score();
            if let Pruned = window.negamax(score) {
                let entry = TTableEntry::new(depth, moves);
                self.table.update::<PV>(Lower, board.last(), entry);
                return draw.unwrap_or(entry).mark();
            }
        }

        if moves.is_none() {
            let check = *board.last().checkers() != EMPTY;

            let eval = if check {
                -MATE
            } else if window.opponent() {
                -DRAW
            } else {
                DRAW
            };

            let entry = TTableEntry::edge(eval);
            self.table.update::<PV>(Exact, board.last(), entry);
            draw.unwrap_or(entry).mark()
        } else {
            let entry = TTableEntry::new(depth, moves);

            let ttype = if original_alpha < window.alpha {
                Exact
            } else {
                Upper
            };

            self.table.update::<PV>(ttype, board.last(), entry);

            draw.unwrap_or(entry).mark()
        }
    }

    pub fn min_search(&mut self, history: BoardHistory) -> TTableEntry {
        return self
            .ab_search::<false>(history, 2, AlphaBeta::new(), &Deadline::none())
            .expect("Expected Complete Search");
    }

    pub fn get_pv_line(&mut self, board: Board) -> PVLine {
        self.table.get_pv_line(board)
    }

    pub fn iterative_deepening_search(
        &mut self,
        history: BoardHistory,
        deadline: &Deadline,
    ) -> Result<TTableEntry, ()> {
        match self.min_search(history) {
            TTableEntry::Node(depth, _) => {
                self.depth = depth + 1;
                let result =
                    self.ab_search::<true>(history, self.depth, AlphaBeta::new(), deadline);

                if let Ok(score) = result {
                    self.table.refresh_pv_line(true);
                    Ok(score)
                } else {
                    self.table.refresh_pv_line(false);
                    Err(())
                }
            }
            TTableEntry::Edge(_score) => Err(()),
        }
    }

    pub fn best_move(&mut self, history: BoardHistory) -> Option<ChessMove> {
        self.min_search(history).peek()
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
