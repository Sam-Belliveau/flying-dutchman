pub mod alpha_beta;
pub mod board_history;
pub mod deadline;
pub mod engine;
pub mod movegen;

pub type Depth = i64;

pub const DEPTH_EDGE: Depth = Depth::MAX / 2;
pub const DEPTH_LEAF: Depth = 0;