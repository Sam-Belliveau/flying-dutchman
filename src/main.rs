mod evaluate;
mod qsearch;
mod search;
mod types;

use std::{time::{Instant, Duration}, io::Write};

use chess::{Board, Color};

use crate::search::Searcher;

fn main() {
    let mut engine = Searcher::new();

    let mut board = Board::default();

    let mut turn = 0;
    let mut deadline = Instant::now();
    loop {
        deadline += Duration::from_millis(500);
        if board.side_to_move() == Color::White {
            turn += 1;
            print!("{}. ", turn);
            std::io::stdout().flush().unwrap();
        }

        while let Some(_result) = engine.iterative_deepening_search(&board, deadline) {
            // println!("\nDepth: {} | Score {} | Move {:?}", result.depth, result.score, result.bmove);
        }

        // let depth = engine.cached_eval(&board).unwrap().depth;
        // println!("selecting");
        if let Some(choice) = engine.best_move(&board) {

            print!("{} ", choice);
            std::io::stdout().flush().unwrap();

            board = board.make_move_new(choice);
        } else {

            println!("\n\nCHECKMATE");
            break;
        }
    }

}
