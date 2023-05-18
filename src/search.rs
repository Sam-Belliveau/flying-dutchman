use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use std::time::Instant;

use chess::{BitBoard, Board, ChessMove, MoveGen, EMPTY};
use nohash_hasher::NoHashHasher;

use crate::qsearch::qsearch;
use crate::types::{Depth, Score, HASH_MAP_SIZE, MATE, MAX_SCORE, MIN_SCORE};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct SearchEval {
    pub depth: Depth,
    pub score: Score,
    pub bmove: Option<ChessMove>,
}

impl SearchEval {
    pub fn new(depth: Depth, score: Score, bmove: Option<ChessMove>) -> SearchEval {
        SearchEval {
            depth: depth.max(0),
            score,
            bmove,
        }
    }

    pub fn leaf(score: Score) -> SearchEval {
        SearchEval::new(0, score, None)
    }

    pub fn is_edge(&self) -> bool {
        self.depth > 0 && self.bmove.is_none()
    }
}

pub struct Searcher {
    eval_table: HashMap<Board, SearchEval, BuildHasherDefault<NoHashHasher<u64>>>,
}

impl Searcher {
    pub fn new() -> Searcher {
        Searcher {
            eval_table: HashMap::with_capacity_and_hasher(
                HASH_MAP_SIZE,
                BuildHasherDefault::default(),
            ),
        }
    }

    fn save(&mut self, board: &Board, result: SearchEval) -> SearchEval {
        self.eval_table.insert(board.clone(), result);
        result
    }

    fn alpha_beta_search(
        &mut self,
        board: &Board,
        depth: Depth,
        mut alpha: Score,
        beta: Score,
        deadline: Instant,
    ) -> Option<SearchEval> {
        if depth > 1 && Instant::now() >= deadline {
            return None;
        }

        let entry = self.eval_table.get(board).cloned();

        if let Some(saved) = entry {
            if depth <= saved.depth {
                return entry;
            }
        } else if depth <= 0 {
            let score = qsearch(board);
            return Some(self.save(board, SearchEval::leaf(score)));
        }

        let mut score = MIN_SCORE;

        let mut moves = MoveGen::new_legal(&board);

        let mut bmove = entry.and_then(|f| f.bmove);
        let m = !bmove.map_or(EMPTY, |f| BitBoard::from_square(f.get_dest()));
        'search: for mask in [
            !m,
            m & board.pieces(chess::Piece::Queen),
            m & board.pieces(chess::Piece::Rook),
            m & (board.pieces(chess::Piece::Bishop) | board.pieces(chess::Piece::Knight)),
            m & board.pieces(chess::Piece::Pawn),
            m & !board.combined(),
        ] {
            moves.set_iterator_mask(mask);

            for movement in &mut moves {
                let eval = -self
                    .alpha_beta_search(
                        &board.make_move_new(movement),
                        depth - 1,
                        -beta,
                        -alpha,
                        deadline,
                    )?
                    .score;

                alpha = alpha.max(score);
                if eval > score {
                    score = eval;
                    bmove = Some(movement);
                }

                if score >= MATE {
                    break 'search;
                } else if score > beta {
                    return Some(SearchEval::new(depth, score, bmove));
                }
            }
        }

        Some(self.save(board, SearchEval::new(depth, score, bmove)))
    }

    pub fn depth_deadline_search(
        &mut self,
        board: &Board,
        depth: Depth,
        deadline: Instant,
    ) -> Option<SearchEval> {
        self.alpha_beta_search(board, depth, MIN_SCORE, MAX_SCORE, deadline)
    }

    pub fn min_search(&mut self, board: &Board) -> SearchEval {
        self.alpha_beta_search(board, 1, MIN_SCORE, MAX_SCORE, Instant::now())
            .unwrap()
    }

    pub fn iterative_deepening_search(
        &mut self,
        board: &Board,
        deadline: Instant,
    ) -> Option<SearchEval> {
        let current_result = self.min_search(board);

        if current_result.is_edge() {
            return None;
        } else {
            let current_depth = current_result.depth;
            self.depth_deadline_search(board, current_depth + 1, deadline)
        }
    }

    pub fn best_move(&mut self, board: &Board) -> Option<ChessMove> {
        self.min_search(board).bmove
    }
}
