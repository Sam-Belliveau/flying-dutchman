pub mod evaluate;
pub mod values;

pub use evaluate::*;

pub type Score = i64;

pub const MIN_SCORE: Score = Score::MIN / 2;
pub const MAX_SCORE: Score = Score::MAX / 2;

pub const MATE: Score = Score::MAX / SCORE_BASE;
pub const MATE_CUTOFF: Score = MATE / 2;

pub const SCORE_BASE: Score = 2 * 3 * 4 * 5 * 6 * 7 * 8;

pub fn score_to_cp(score: Score) -> Score {
    score / (100 * SCORE_BASE)
}
