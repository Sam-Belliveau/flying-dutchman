pub type Score = i64;

// By defining a Centipawn as this number,
// the score will be a multiple of many prime factors,
// making it easier to divide and work with.
pub const CENTIPAWN: Score = 720720;

pub const MATE: Score = Score::MAX / 16;
pub const MATE_MOVE: Score = CENTIPAWN;
pub const MATE_CUTOFF: Score = MATE - 1024 * MATE_MOVE;

pub const DRAW: Score = 0;

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
    score / (CENTIPAWN)
}

pub fn score_to_str(score: Score) -> String {
    let side = if score < 0 { "-" } else { "" };
    let score = score.abs();

    if score >= MATE_CUTOFF {
        let diff = MATE - score;
        let moves = (diff + 3 * MATE_MOVE / 2) / (2 * MATE_MOVE);

        format!("mate {}{}", side, moves)
    } else {
        format!("cp {}{}", side, score_to_cp(score))
    }
}
