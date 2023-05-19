use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use std::time::Instant;

use chess::{BitBoard, Board, ChessMove, MoveGen, Piece, EMPTY};
use nohash_hasher::NoHashHasher;

use crate::qsearch::qsearch;
use crate::types::{Depth, Score, DEPTH_JUMP, HASH_MAP_SIZE, MATE, MAX_SCORE, MIN_SCORE};

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

    pub fn with_depth(&self, depth: Depth) -> SearchEval {
        SearchEval {
            depth,
            score: self.score,
            bmove: self.bmove,
        }
    }
}

pub struct Searcher {
    eval_table: [HashMap<Board, SearchEval, BuildHasherDefault<NoHashHasher<u64>>>; 64],
    ticker: usize,
}

impl Searcher {
    pub fn new() -> Searcher {
        Searcher {
            eval_table: [(); 64].map(|_| {
                HashMap::with_capacity_and_hasher(HASH_MAP_SIZE, BuildHasherDefault::default())
            }),
            ticker: 0,
        }
    }

    fn random_bool(&mut self) -> bool {
        self.ticker = self.ticker.wrapping_add(1);
        self.ticker.count_ones() & 1 == 0
    }

    fn piece_count(board: &Board) -> usize {
        board.combined().popcnt() as usize
    }

    fn save(&mut self, board: &Board, result: SearchEval) -> &SearchEval {
        self.eval_table[Self::piece_count(board)]
            .entry(board.clone())
            .and_modify(|e| {
                if e.depth <= result.depth {
                    *e = result;
                }
            })
            .or_insert(result)
    }

    fn update(&mut self, board: &Board, result: SearchEval) -> &SearchEval {
        self.eval_table[Self::piece_count(board)]
            .entry(board.clone())
            .and_modify(|e| {
                if e.depth <= result.depth && e.score < result.score {
                    e.score = result.score;
                    e.bmove = result.bmove;
                }
            })
            .or_insert_with(|| result.with_depth(0))
    }

    fn get(&mut self, board: &Board) -> Option<&SearchEval> {
        self.eval_table[Self::piece_count(board)].get(board)
    }

    fn alpha_beta_search(
        &mut self,
        board: Board,
        depth: Depth,
        mut alpha: Score,
        beta: Score,
        deadline: Instant,
    ) -> Option<&SearchEval> {
        if depth > DEPTH_JUMP && Instant::now() >= deadline {
            return None;
        }

        let entry = self.get(&board).cloned();

        if let Some(saved) = entry {
            if depth <= saved.depth {
                return self.get(&board);
            }
        } else if depth <= 0 {
            let score = qsearch(board);
            return Some(self.save(&board, SearchEval::leaf(score)));
        }

        let mut score = MIN_SCORE;

        let mut moves = MoveGen::new_legal(&board);
        let mut bmove = entry.and_then(|f| f.bmove);

        'search: for (d, mask) in [
            (
                1 * DEPTH_JUMP / 3,
                bmove.map_or(EMPTY, |f| BitBoard::from_square(f.get_dest())),
            ),
            (1 * DEPTH_JUMP / 3, *board.pieces(Piece::Queen)),
            (2 * DEPTH_JUMP / 3, *board.pieces(Piece::Rook)),
            (
                2 * DEPTH_JUMP / 3,
                board.pieces(Piece::Bishop) | board.pieces(Piece::Knight),
            ),
            (3 * DEPTH_JUMP / 3, *board.pieces(Piece::Pawn)),
            (3 * DEPTH_JUMP / 2, !EMPTY),
        ] {
            moves.set_iterator_mask(mask);

            for movement in &mut moves {
                let result = board.make_move_new(movement);
                let i = if *result.checkers() == EMPTY {
                    d
                } else {
                    d / 5
                };
                let child = self.alpha_beta_search(result, depth - i, -beta, -alpha, deadline)?;
                let eval = -child.score;

                if eval > score || (eval == score && self.random_bool()) {
                    score = eval;
                    bmove = Some(movement);
                }

                alpha = alpha.max(score);

                if score >= MATE {
                    break 'search;
                } else if score >= beta {
                    Some(self.update(&board, SearchEval::new(depth, score, bmove)));
                }
            }
        }

        Some(self.save(&board, SearchEval::new(depth, score, bmove)))
    }

    pub fn depth_deadline_search(
        &mut self,
        board: Board,
        depth: Depth,
        deadline: Instant,
    ) -> Option<&SearchEval> {
        self.alpha_beta_search(board, depth, MIN_SCORE, MAX_SCORE, deadline)
    }

    pub fn min_search(&mut self, board: Board, depth: Depth) -> &SearchEval {
        self.alpha_beta_search(board, depth, MIN_SCORE, MAX_SCORE, Instant::now())
            .unwrap()
    }

    pub fn iterative_deepening_search(
        &mut self,
        board: &Board,
        deadline: Instant,
    ) -> Option<&SearchEval> {
        let current_result = self.min_search(*board, DEPTH_JUMP);

        if current_result.is_edge() {
            return None;
        } else {
            let current_depth = current_result.depth;
            self.depth_deadline_search(*board, current_depth + DEPTH_JUMP / 2, deadline)
        }
    }

    pub fn best_move(&mut self, board: &Board) -> Option<ChessMove> {
        for i in (Self::piece_count(board) + 1)..64 {
            self.eval_table[i].clear();
        }

        self.min_search(*board, 0).bmove
    }
}
