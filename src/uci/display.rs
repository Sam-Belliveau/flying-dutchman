use std::io::{self, Write};
use std::time::Instant;

use crate::evaluate::score_to_str;
use crate::search::board_history::BoardHistory;
use crate::search::engine::Engine;

pub fn stdout_sync() {
    let _ = io::stdout().flush();
}

#[macro_export]
macro_rules! uci_token {
    ($val:expr) => {
        print!("{} ", $val)
    };
}

#[macro_export]
macro_rules! uci_variable {
    ($val:expr) => {
        print!("{} {} ", stringify!($val), $val)
    };
}

#[macro_export]
macro_rules! uci_end {
    () => {
        println!();
        stdout_sync();
    };
}

pub fn board_information(engine: &mut Engine, history: &BoardHistory, search_start: Instant) {
    let board_info = engine.min_search(history);

    let time_nano = search_start.elapsed().as_nanos() as usize + 1;

    let depth = board_info.depth();
    let seldepth = engine.get_pv_line(history.last()).count();
    let multipv = 1;
    let score = score_to_str(board_info.score());
    let nodes = 1 + engine.get_node_count();
    let nps = nodes * 1_000_000_000 / time_nano;
    let hashfull = engine.table.hashfull_permille();
    let tbhits = 0;
    let time = time_nano / 1_000_000;

    uci_token!("info");
    uci_variable!(depth);
    uci_variable!(seldepth);
    uci_variable!(multipv);
    uci_variable!(score);
    uci_variable!(nodes);
    uci_variable!(nps);
    uci_variable!(hashfull);
    uci_variable!(tbhits);
    uci_variable!(time);

    uci_token!("pv");
    for movement in engine.get_pv_line(history.last()) {
        uci_token!(movement);
    }

    uci_end!();
}

pub fn board_best_move(engine: &mut Engine, board: &BoardHistory) {
    let board_info = engine.min_search(board);

    if let Some(bestmove) = board_info.peek() {
        let score = score_to_str(board_info.score());
        let depth = board_info.depth();

        uci_variable!(bestmove);
        uci_token!("info");
        uci_variable!(score);
        uci_variable!(depth);
    } else {
        uci_token!("bestmove");
        uci_token!("(none)");
    }

    uci_end!();
}
