mod evaluate;
mod qsearch;
mod search;
mod types;

use std::{time::{Instant, Duration}, io::Write};

use chess::{Board, Color};
use search::SearchEval;

use crate::{search::Searcher, types::{from_score, from_depth}};

fn print_result(title: &str, result: &SearchEval) {
    println!("{}|   Depth:{:7.3}   |   Score{:7.3}   |   Move {}", title, from_depth(result.depth), from_score(result.score), result.bmove.map_or(String::from("None"), |m| m.to_string()));
}

fn main() {
    let mut engine = Searcher::new();

    let mut board = Board::default();

    let mut turn = 0;
    let mut game = String::from("");
    let mut deadline;
    loop {
        if board.side_to_move() == Color::White {
            turn += 1;
            game += &format!("{}. ", turn);
        }

        match board.side_to_move() {
            Color::White => print!("White to move:\n"),
            Color::Black => print!("Black to move:\n"),
        }

        deadline = Instant::now() + Duration::from_millis(20000);
        print_result("Init   ", engine.min_search(board, 0));
        while let Some(result) = engine.iterative_deepening_search(&board, deadline) {
            print_result("Iter   ", result);
        }

        // let depth = engine.cached_eval(&board).unwrap().depth;
        // println!("selecting");
        if let Some(choice) = engine.best_move(&board) {
            print_result("Final  ", engine.min_search(board, 0));
            print!("------------------------------------------------------------\n");
            game += &format!("{} ", choice);
            print!("{}\n", game);
            print!("------------------------------------------------------------");

            std::io::stdout().flush().unwrap();
            print!("\n\n");
            board = board.make_move_new(choice);
        } else {

            match board.status() {
                chess::BoardStatus::Checkmate => {
                    print!("Checkmate!\n");
                    match board.side_to_move() {
                        Color::White => print!("Black wins!\n"),
                        Color::Black => print!("White wins!\n"),
                    }
                },
                chess::BoardStatus::Stalemate => {
                    print!("Stalemate!\n");
                },
                chess::BoardStatus::Ongoing => {
                    print!("Error!\n");
                },
            }
            break;
        }
    }

}
