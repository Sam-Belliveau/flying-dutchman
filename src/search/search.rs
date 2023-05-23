use chess::{BitBoard, Board, ChessMove, MoveGen, Piece, EMPTY};

use crate::evaluate::{Score, MATE_CUTOFF};
use crate::search::qsearch::qsearch;
use crate::transposition::table::TTable;
use crate::transposition::table_entry::TTableEntry;

use super::alpha_beta::{AlphaBeta, NegaMaxResult::*, ProbeResult::*};
use super::deadline::Deadline;
use super::Depth;

pub struct Searcher {
    table: TTable,
}

impl Searcher {
    pub fn new() -> Searcher {
        Searcher {
            table: TTable::new(),
        }
    }

    fn alpha_beta_search(
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

        let mut moves = MoveGen::new_legal(&board);
        let mut bmove = None;

        let pv = entry.and_then(|f| f.best_move);
        'search: for mask in [
            pv.map_or(EMPTY, |f| BitBoard::from_square(f.get_dest())),
            *board.pieces(Piece::Queen),
            *board.pieces(Piece::Rook),
            *board.pieces(Piece::Bishop),
            *board.pieces(Piece::Knight),
            *board.pieces(Piece::Pawn),
            !EMPTY,
        ] {
            moves.set_iterator_mask(mask);

            for movement in &mut moves {
                let result = board.make_move_new(movement);

                let eval = if bmove.is_none() {
                    -self.alpha_beta_search(result, depth - 1, -window, deadline)?
                } else {
                    let eval = -self.alpha_beta_search(
                        result,
                        depth - 1,
                        -window.null_window(),
                        deadline,
                    )?;
                    if let (true, Contained { .. }) = (depth > 1, window.probe(eval)) {
                        eval.max(-self.alpha_beta_search(result, depth - 1, -window, deadline)?)
                    } else {
                        eval
                    }
                };

                match window.negamax(eval) {
                    Worse { .. } => {}
                    Matches { .. } => {
                        // Do not replace best move if there is no improvement
                    }
                    NewBest { .. } => {
                        bmove = Some(movement);
                    }
                    BetaPrune { .. } => {
                        bmove = Some(movement);

                        if eval >= MATE_CUTOFF {
                            break 'search;
                        } else {
                            self.table
                                .update(&board, TTableEntry::new(depth, eval, bmove));
                            return Some(eval);
                        }
                    }
                }
            }
        }

        self.table
            .save(&board, TTableEntry::new(depth, window.alpha(), bmove));

        Some(window.alpha())
    }

    pub fn depth_deadline_search(
        &mut self,
        board: Board,
        depth: Depth,
        deadline: &Deadline,
    ) -> Option<Score> {
        self.alpha_beta_search(board, depth, AlphaBeta::new(), deadline)
    }

    pub fn min_search(&mut self, board: &Board) -> TTableEntry {
        if let Some(result) = self.table.get(&board) {
            if result.depth > 0 {
                return *result;
            }
        }

        self.alpha_beta_search(*board, 1, AlphaBeta::new(), &Deadline::none())
            .expect("Expected Complete Search");
        return *self.table.get(&board).unwrap();
    }

    pub fn opt_search(&mut self, board: &Board) -> Option<&TTableEntry> {
        self.table.get(&board)
    }

    pub fn iterative_deepening_search(
        &mut self,
        board: &Board,
        deadline: &Deadline,
    ) -> Option<Score> {
        let current = self.min_search(board);
        if current.is_edge() {
            None
        } else {
            self.depth_deadline_search(*board, current.depth + 1, deadline)
        }
    }

    pub fn best_move(&mut self, board: &Board) -> Option<ChessMove> {
        let best_move = self.min_search(board).best_move;
        self.table.sweep();
        best_move
    }

    pub fn memory_bytes(&self) -> usize {
        self.table.memory_bytes()
    }
}