// score.rs

pub type Score = i64;

pub const MATE: Score = Score::MAX / 2;
pub const MATE_CUTOFF: Score = MATE / 2;
pub const MATE_MOVE: Score = MATE / 256;

pub const SCORE_BASE: Score = 2 * 3 * 4 * 5 * 6 * 7 * 8;

pub fn score_mark(score: Score) -> Score {
    // This is how we keep track of how many moves we are from mate.
    // Every time we call this function, we take a bit from the score,
    // to indicate that it takes an extra move to get here.
    if score >= MATE_CUTOFF {
        score - MATE_MOVE
    } else if score <= -MATE_CUTOFF {
        score + MATE_MOVE
    } else {
        score
    }
}

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
