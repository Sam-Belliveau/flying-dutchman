// score.rs

pub type Score = i64;

pub const MATE: Score = Score::MAX / 2;
pub const MATE_CUTOFF: Score = MATE / 2;
pub const MATE_MOVE: Score = MATE / 256;

pub const SCORE_BASE: Score = 2 * 3 * 4 * 5 * 6 * 7 * 8;

pub fn score_to_cp(score: Score) -> Score {
    score / (SCORE_BASE)
}

pub fn score_to_str(score: Score) -> String {
    if score >= MATE_CUTOFF {
        format!("mate {}", (MATE - score) / (2 * MATE_MOVE))
    } else if score <= -MATE_CUTOFF {
        format!("mate -{}", (score + MATE) / (2 * MATE_MOVE))
    } else {
        format!("cp {}", score_to_cp(score))
    }
}
