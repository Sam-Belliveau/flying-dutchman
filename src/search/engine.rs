use std::time::Instant;

use chess::{Board, ChessMove, EMPTY};

use crate::evaluate::{evaluate, score_mark, Score, DRAW, MATE, MATE_CUTOFF};

use crate::search::alpha_beta::{AlphaBeta, NegaMaxResult::*};
use crate::search::board_history::BoardHistory;
use crate::search::deadline::Deadline;
use crate::search::movegen::OrderedMoveGen;
use crate::search::Depth;

use crate::transposition::best_moves::BestMoves;
use crate::transposition::pv_line::PVLine;
use crate::transposition::rated_move::RatedMove;
use crate::transposition::table::{TTable, TTableSample};
use crate::transposition::table_entry::TTableEntry;

use super::opponent_engine::OpponentEngine;

const DEFAULT_TABLE_SIZE: usize = 1000 * 1000 * 1000;
const OPPONENT_EVAL_PLY: Depth = 2;

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

    pub fn ab_qsearch(board: &Board, mut window: AlphaBeta) -> Score {
        let (mut best, movegen) = {
            if *board.checkers() == EMPTY {
                let score = evaluate(&board);
                if let Pruned = window.negamax(score) {
                    return score_mark(score);
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
            let eval = -Self::ab_qsearch(&new_board, -window);

            best = best.max(eval);
            if let Pruned = window.negamax(eval) {
                break;
            }
        }

        score_mark(best)
    }

    fn ab_search<const PV: bool>(
        &mut self,
        board: &BoardHistory,
        depth: Depth,
        mut window: AlphaBeta,
        deadline: &Deadline,
        opponent_engine: &mut Option<OpponentEngine>,
    ) -> Result<TTableEntry, ()> {
        let original_window = window;
        self.nodes += 1;

        // Check for Time Exceeded
        if deadline.passed() {
            return Err(());
        }

        // Draw Detection and Handling
        if board.is_draw() {
            return TTableEntry::Edge(if window.opponent() { -DRAW } else { DRAW }).mark();
        }

        // Quiescence Search
        if depth <= 0 {
            let eval = Self::ab_qsearch(board.last(), window);
            let entry = TTableEntry::Leaf(eval);
            return entry.mark();
        }

        // Opponent Modeling to
        if window.ply < OPPONENT_EVAL_PLY && window.opponent() {
            if let Some(opponent) = opponent_engine {
                match opponent.get_move(board, deadline) {
                    Ok(opponent_move) => {
                        let mut moves = BestMoves::new();

                        let next = board.with_move(opponent_move);
                        let eval = -self
                            .ab_search::<PV>(&next, depth - 1, -window, deadline, opponent_engine)?
                            .score();

                        moves.push(RatedMove::new(eval, opponent_move));

                        let entry = original_window.new_table_entry(depth, moves);
                        self.table.update::<PV>(board.last(), entry);
                        return entry.mark();
                    }
                    Err(error) => {
                        eprintln!("Opponent Engine Error: {}", error);
                    }
                }
            }
        }

        // Transposition Table Lookup
        let pv = match self.table.sample::<PV>(&board.last(), &window, depth) {
            TTableSample::Moves(moves) => moves,
            TTableSample::Score(score) => return score.mark(),
            TTableSample::None => BestMoves::new(),
        };

        // Normal Alpha Beta Search
        let check = *board.last().checkers() != EMPTY;

        let mut moves = BestMoves::new();
        for (move_count, movement) in OrderedMoveGen::full_search(board.last(), pv).enumerate() {
            let next = board.with_move(movement);

            let eval = if PV && move_count == 0 {
                -self
                    .ab_search::<PV>(&next, depth - 1, -window, deadline, opponent_engine)?
                    .score()
            } else {
                let reduction =
                    if depth < 3 || move_count <= 3 || check || *next.last().checkers() != EMPTY {
                        0
                    } else if movement.get_promotion().is_some()
                        || board.last().piece_on(movement.get_dest()).is_some()
                    {
                        (0.7 + 0.3 * (depth as f64).ln_1p() + 0.3 * (move_count as f64).ln_1p())
                            as Depth
                    } else {
                        (1.0 + 0.5 * (depth as f64).ln_1p() + 0.7 * (move_count as f64).ln_1p())
                            as Depth
                    }
                    .clamp(0, depth - 1);

                let eval = -self
                    .ab_search::<false>(
                        &next,
                        depth - reduction - 1,
                        -window,
                        deadline,
                        opponent_engine,
                    )?
                    .score();

                if 0 < reduction && window.alpha < eval {
                    -self
                        .ab_search::<false>(&next, depth - 1, -window, deadline, opponent_engine)?
                        .score()
                } else {
                    eval
                }
            };

            moves.push(RatedMove::new(eval, movement));
            if let Pruned = window.negamax(eval) {
                break;
            }
        }

        // Check for Checkmate or Draw
        if moves.is_none() {
            let check = *board.last().checkers() != EMPTY;

            let eval = if check {
                -MATE
            } else if window.opponent() {
                -DRAW
            } else {
                DRAW
            };

            let entry = TTableEntry::Edge(eval);
            self.table.update::<PV>(board.last(), entry);
            return entry.mark();
        }

        // Store and Return Results
        let entry = original_window.new_table_entry(depth, moves);
        self.table.update::<PV>(board.last(), entry);
        entry.mark()
    }

    pub fn min_search(&mut self, history: &BoardHistory) -> TTableEntry {
        return self
            .ab_search::<false>(&history, 1, AlphaBeta::new(), &Deadline::none(), &mut None)
            .expect("Expected Complete Search");
    }

    pub fn get_pv_line(&mut self, board: &Board) -> PVLine {
        self.table.get_pv_line(board)
    }

    pub fn iterative_deepening_search(
        &mut self,
        history: &BoardHistory,
        deadline: &Deadline,
        opponent_engine: &mut Option<OpponentEngine>,
    ) -> Result<TTableEntry, ()> {
        let depth = match self.min_search(&history) {
            TTableEntry::Edge(..) => return Err(()),
            default => default.depth(),
        };

        if !deadline.check_depth(depth) {
            return Err(());
        }

        let result = self.ab_search::<true>(
            &history,
            depth + 1,
            AlphaBeta::new(),
            deadline,
            opponent_engine,
        );
        self.table.promote_pv_line(history.last());

        if let Ok(score) = result {
            Ok(score)
        } else {
            Err(())
        }
    }

    pub fn best_move(&mut self, history: &BoardHistory) -> Option<ChessMove> {
        self.min_search(&history).peek()
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
