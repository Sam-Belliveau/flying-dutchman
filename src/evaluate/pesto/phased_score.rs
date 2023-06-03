use chess::{Color, Piece};

use crate::evaluate::{Score, SCORE_BASE};

use super::gamephase::GamePhase;

#[derive(Clone, Copy, Debug)]
pub struct RawPhasedScore {
    mid_game: Score,
    end_game: Score,
}

impl RawPhasedScore {
    pub fn new(mid_game: Score, end_game: Score) -> RawPhasedScore {
        Self { mid_game, end_game }
    }

    pub fn from_piece(piece: Piece) -> RawPhasedScore {
        match piece {
            Piece::Pawn => Self::new(82, 94),
            Piece::Knight => Self::new(337, 281),
            Piece::Bishop => Self::new(365, 297),
            Piece::Rook => Self::new(477, 512),
            Piece::Queen => Self::new(1025, 936),
            Piece::King => Self::new(0, 0),
        }
    }

    pub fn colorize(self, color: Color) -> PhasedScore {
        match color {
            Color::White => PhasedScore {
                mid_game: self.mid_game,
                end_game: self.end_game,
            },
            Color::Black => PhasedScore {
                mid_game: -self.mid_game,
                end_game: -self.end_game,
            },
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PhasedScore {
    mid_game: Score,
    end_game: Score,
}

impl PhasedScore {
    pub fn new() -> PhasedScore {
        Self {
            mid_game: 0,
            end_game: 0,
        }
    }

    pub fn from_piece(piece: Piece, color: Color) -> PhasedScore {
        RawPhasedScore::from_piece(piece).colorize(color)
    }

    pub fn collapse(self, phase: GamePhase) -> Score {
        phase.weight(SCORE_BASE * self.mid_game, SCORE_BASE * self.end_game)
    }
}

impl std::ops::AddAssign for PhasedScore {
    fn add_assign(&mut self, rhs: Self) {
        self.mid_game += rhs.mid_game;
        self.end_game += rhs.end_game;
    }
}
