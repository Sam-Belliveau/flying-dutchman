pub type Score = i64;

pub const MATE: Score = Score::MAX / SCORE_BASE;

pub const MIN_SCORE: Score = Score::MIN / 2;
pub const MAX_SCORE: Score = Score::MAX / 2;

pub const SCORE_BASE: Score = 2 * 3 * 4 * 5 * 6 * 7 * 8 * 9 * 10;

pub type Depth = i64;

// This is very factorable as it allows the depths to be very dynamic
pub const DEPTH_JUMP: Depth = 2 * 2 * 2 * 3 * 3 * 5;

pub const HASH_MAP_SIZE: usize = 1 << 10;

pub fn from_depth(depth: Depth) -> f64 {
    depth as f64 / DEPTH_JUMP as f64
}

pub fn from_score(depth: Score) -> f64 {
    depth as f64 / (100.0 * SCORE_BASE as f64)
}