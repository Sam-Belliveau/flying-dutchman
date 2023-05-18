pub type Score = i64;

pub const MATE: Score = Score::MAX / SCORE_BASE;
pub const DRAW: Score = 0;

pub const MIN_SCORE: Score = Score::MIN / 2;
pub const MAX_SCORE: Score = Score::MAX / 2;

pub const SCORE_BASE: Score = 1 << 16;

pub type Depth = i64;

pub const HASH_MAP_SIZE: usize = 1 << 10;
