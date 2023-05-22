use chess::{BitBoard, Board, ChessMove, MoveGen, Piece, EMPTY};

use crate::deadline::Deadline;
use crate::qsearch::qsearch;
use crate::transposition::table::TTable;
use crate::transposition::table_entry::TTableEntry;
use crate::types::{Depth, Score, MATE_CUTOFF, MAX_SCORE, MIN_SCORE};

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
        mut alpha: Score,
        beta: Score,
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
                    -self.alpha_beta_search(result, depth - 1, -beta, -alpha, deadline)?
                } else {
                    let eval =
                        -self.alpha_beta_search(result, depth - 1, -alpha - 1, -alpha, deadline)?;
                    if alpha < eval && eval < beta && depth > 1 {
                        eval.max(-self.alpha_beta_search(
                            result,
                            depth - 1,
                            -beta,
                            -eval,
                            deadline,
                        )?)
                    } else {
                        eval
                    }
                };

                if eval > alpha {
                    alpha = eval;
                    bmove = Some(movement);

                    if alpha >= MATE_CUTOFF {
                        break 'search;
                    } else if alpha >= beta {
                        self.table
                            .update(&board, TTableEntry::new(depth, alpha, bmove));
                        return Some(alpha);
                    }
                }
            }
        }

        self.table
            .save(&board, TTableEntry::new(depth, alpha, bmove));

        Some(alpha)
    }

    pub fn depth_deadline_search(
        &mut self,
        board: Board,
        depth: Depth,
        deadline: &Deadline,
    ) -> Option<Score> {
        self.table.mark();
        self.alpha_beta_search(board, depth, MIN_SCORE, MAX_SCORE, deadline)
    }

    pub fn min_search(&mut self, board: &Board) -> TTableEntry {
        if let Some(result) = self.table.get(&board) {
            if result.depth > 0 {
                return *result;
            }
        }

        self.table.mark();
        self.alpha_beta_search(*board, 1, MIN_SCORE, MAX_SCORE, &Deadline::none())
            .expect("Expected Complete Search");
        return *self.table.get(&board).unwrap();
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
        let max_size = 1000 * 1000 * 1000; // Gig
        self.table.sweep(max_size);
        best_move
    }

    pub fn memory_bytes(&self) -> usize {
        self.table.memory_bytes()
    }
}
