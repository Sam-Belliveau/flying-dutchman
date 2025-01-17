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

const DEFAULT_TABLE_SIZE: usize = 1000 * 1000 * 1000;

pub struct Engine {
    pub table: TTable,
    pub opponent_engine: Option<Box<Engine>>,
    nodes: usize,
}

impl Engine {
    pub fn new() -> Engine {
        Engine {
            table: TTable::new(DEFAULT_TABLE_SIZE),
            opponent_engine: Some(Box::new(Engine {
                table: TTable::new(DEFAULT_TABLE_SIZE),
                opponent_engine: None,
                nodes: 0,
            })),
            nodes: 0,
        }
    }

    fn wrap(score: Score) -> Result<Score, ()> {
        Ok(score_mark(score))
    }

    pub fn ab_qsearch(board: &Board, mut window: AlphaBeta) -> Result<Score, ()> {
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
            let eval = -Self::ab_qsearch(&new_board, -window)?;

            best = best.max(eval);
            if let Pruned = window.negamax(eval) {
                return Self::wrap(best);
            }
        }

        Self::wrap(best)
    }

    fn ab_search<const PV: bool>(
        &mut self,
        board: &BoardHistory,
        depth: Depth,
        mut window: AlphaBeta,
        deadline: &Deadline,
    ) -> Result<TTableEntry, ()> {
        let original_window = window;
        self.nodes += 1;

        // Check for Time Exceeded
        if deadline.passed() {
            return Err(());
        }

        // Draw Detection and Handling
        let draw = if board.is_draw() {
            Some(TTableEntry::edge(if window.opponent() {
                -DRAW
            } else {
                DRAW
            }))
        } else {
            None
        };

        // Quiescence Search
        if depth <= 0 {
            let entry = TTableEntry::new_score(0, Self::ab_qsearch(board.last(), window)?);
            return draw.unwrap_or(entry).mark();
        }

        // Opponent Modeling to improve play against humans.
        let opponent_depth = 7;
        if let Some(opponent_engine) = &mut self.opponent_engine {
            if window.opponent() && window.ply < 2 {
                let opponent_eval = opponent_engine.ab_search::<PV>(
                    board,
                    opponent_depth.min(depth),
                    AlphaBeta::new(),
                    deadline,
                )?;

                if let Some(opponent_move) = opponent_eval.peek() {
                    let next = board.with_move(opponent_move);
                    let eval = -self
                        .ab_search::<PV>(&next, depth - 1, -window, deadline)?
                        .score();

                    let entry = TTableEntry::new(
                        depth,
                        BestMoves::Best1(RatedMove::new(eval, opponent_move)),
                    );

                    let ttype = window.table_entry_type(eval);
                    self.table.update::<PV>(ttype, board.last(), entry);
                    return draw.unwrap_or(entry).mark();
                }
            }
        }

        // Transposition Table Lookup
        let pv = match self.table.sample::<PV>(&board.last(), &window, depth) {
            TTableSample::Moves(moves) => moves,
            TTableSample::Score(score) => return score.mark(),
            TTableSample::None => BestMoves::new(),
        };

        // Null Move Pruning
        let r: Depth = 2;
        if !PV && depth > r {
            if let Some(null_board) = board.with_null_move() {
                let null_eval = -self
                    .ab_search::<false>(&null_board, depth - r, window.null_move(), deadline)?
                    .score();

                if null_eval >= window.beta {
                    let entry = TTableEntry::new_score(depth, null_eval);
                    self.table.update::<false>(Lower, board.last(), entry);
                    return draw.unwrap_or(entry).mark();
                }
            }
        }

        // Normal Alpha Beta Search
        let check = *board.last().checkers() != EMPTY;

        let mut moves = BestMoves::new();
        for (move_count, movement) in OrderedMoveGen::full_search(board.last(), pv).enumerate() {
            let next = board.with_move(movement);

            let eval = if PV && move_count == 0 {
                -self
                    .ab_search::<PV>(&next, depth - 1, -window, deadline)?
                    .score()
            } else {
                let reduction =
                    if depth < 3 || move_count < 4 || check || *next.last().checkers() != EMPTY {
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
                    .ab_search::<false>(&next, depth - reduction - 1, -window, deadline)?
                    .score();

                if 0 < reduction && window.alpha < eval {
                    -self
                        .ab_search::<false>(&next, depth - 1, -window, deadline)?
                        .score()
                } else {
                    eval
                }
            };

            moves.push(RatedMove::new(eval, movement));
            if let Pruned = window.negamax(eval) {
                let entry = TTableEntry::new(depth, moves);
                self.table.update::<PV>(Lower, board.last(), entry);
                return draw.unwrap_or(entry).mark();
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

            let entry = TTableEntry::edge(eval);
            self.table.update::<PV>(Exact, board.last(), entry);
            return draw.unwrap_or(entry).mark();
        }

        // Store and Return Results
        let entry = TTableEntry::new(depth, moves);
        let ttype = original_window.table_entry_type(moves.score());

        self.table.update::<PV>(ttype, board.last(), entry);
        draw.unwrap_or(entry).mark()
    }

    pub fn min_search(&mut self, history: &BoardHistory) -> TTableEntry {
        return self
            .ab_search::<false>(&history, 1, AlphaBeta::new(), &Deadline::none())
            .expect("Expected Complete Search");
    }

    pub fn get_pv_line(&mut self, board: &Board) -> PVLine {
        self.table.get_pv_line(board)
    }

    pub fn iterative_deepening_search(
        &mut self,
        history: &BoardHistory,
        deadline: &Deadline,
    ) -> Result<TTableEntry, ()> {
        match self.min_search(&history) {
            TTableEntry::Node(mut depth, _) => {
                if !deadline.check_depth(depth) {
                    return Err(());
                }

                depth += 1;
                let result = self.ab_search::<true>(&history, depth, AlphaBeta::new(), deadline);
                self.table.promote_pv_line(history.last());

                if let Ok(score) = result {
                    Ok(score)
                } else {
                    Err(())
                }
            }
            TTableEntry::Edge(..) => Err(()),
        }
    }

    pub fn best_move(&mut self, history: &BoardHistory) -> Option<ChessMove> {
        self.min_search(&history).peek()
    }

    pub fn start_new_search(&mut self) -> Instant {
        self.nodes = 0;

        if let Some(opponent_engine) = &mut self.opponent_engine {
            opponent_engine.start_new_search();
        }

        Instant::now()
    }

    pub fn get_node_count(&self) -> usize {
        if let Some(opponent_engine) = &self.opponent_engine {
            self.nodes + opponent_engine.get_node_count()
        } else {
            self.nodes
        }
    }

    pub fn memory_bytes(&self) -> usize {
        self.table.memory_bytes()
    }
}
