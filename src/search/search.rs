use chess::{Board, ChessMove};

use crate::evaluate::{Score, MATE_CUTOFF};
use crate::search::qsearch::qsearch;
use crate::transposition::pv_line::PVLine;
use crate::transposition::table::TTable;
use crate::transposition::table_entry::TTableEntry;

use super::alpha_beta::{AlphaBeta, NegaMaxResult::*, ProbeResult::*};
use super::deadline::Deadline;
use super::movegen::OrderedMoveGen;
use super::Depth;

const DEFAULT_TABLE_SIZE: usize = 1000 * 1000 * 1000;

pub struct Searcher {
    table: TTable,
}

impl Searcher {
    pub fn new() -> Searcher {
        Searcher {
            table: TTable::new(DEFAULT_TABLE_SIZE),
        }
    }

    pub fn set_table_size(&mut self, table_size: usize) {
        self.table.set_table_size(table_size);
    }

    fn ab_search(
        &mut self,
        board: Board,
        depth: Depth,
        mut window: AlphaBeta,
        deadline: &Deadline,
    ) -> Option<Score> {
        let entry = self.table.get(&board);

        if let Some(saved) = entry {
            if depth <= saved.depth {
                return Some(saved.score);
            }
        }

        if depth <= 0 {
            let score = qsearch(board);
            self.table.save(&board, TTableEntry::leaf(score));
            return Some(score);
        }

        if deadline.passed() {
            return None;
        }

        let mut best_move = None;
        for movement in OrderedMoveGen::new(&board, entry.and_then(|f| f.best_move)) {
            let new_board = board.make_move_new(movement);

            let eval = if best_move.is_some() {
                let eval =
                    -self.ab_search(new_board, depth - 1, -window.null_window(), deadline)?;
                if let (true, Contained { .. }) = (depth > 1, window.probe(eval)) {
                    eval.max(-self.ab_search(
                        new_board,
                        depth - 1,
                        -window.raise_min(eval),
                        deadline,
                    )?)
                } else {
                    eval
                }
            } else {
                -self.ab_search(new_board, depth - 1, -window, deadline)?
            };

            match window.negamax(eval) {
                Worse { .. } | Equal { .. } => {}
                Best { .. } => {
                    best_move = Some(movement);
                }
                Pruned { .. } => {
                    best_move = Some(movement);

                    if eval >= MATE_CUTOFF {
                        break;
                    } else {
                        self.table
                            .update(&board, TTableEntry::new(depth, eval, movement));
                        return Some(eval);
                    }
                }
            }
        }

        if let Some(best_move) = best_move {
            self.table
                .save(&board, TTableEntry::new(depth, window.alpha(), best_move));
        }

        Some(window.alpha())
    }

    pub fn min_search(&mut self, board: &Board) -> TTableEntry {
        if let Some(result) = self.table.get(board) {
            if result.best_move.is_some() {
                return *result;
            }
        }

        self.ab_search(*board, 1, AlphaBeta::new(), &Deadline::none())
            .expect("Expected Complete Search");

        return *self.table.get(board).unwrap();
    }

    pub fn get_pv_line(&mut self, board: Board) -> PVLine {
        self.table.get_pv_line(board)
    }

    pub fn iterative_deepening_search(
        &mut self,
        board: &Board,
        deadline: &Deadline,
    ) -> Option<Score> {
        let current = self.min_search(board);

        self.table.refresh_pv_line(*board);
        self.table.sweep();

        if current.is_edge() {
            None
        } else {
            self.ab_search(*board, current.depth + 1, AlphaBeta::new(), deadline)
        }
    }

    pub fn best_move(&mut self, board: &Board) -> Option<ChessMove> {
        self.min_search(board).best_move
    }

    pub fn memory_bytes(&self) -> usize {
        self.table.memory_bytes()
    }
}
