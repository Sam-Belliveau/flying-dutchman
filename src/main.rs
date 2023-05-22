mod evaluate;
mod qsearch;
mod search;
mod types;
mod transposition;
mod deadline;

use std::{time::{Duration}, io::Write};

use chess::{Board, Color};
use deadline::Deadline;
use transposition::table_entry::TTableEntry;

use crate::{search::Searcher, types::{from_score}};

fn print_result(title: &str, result: TTableEntry) {
    print!("{}|   Depth:{:16.3}   |   Score{:16.3}   |   Move {}\n", title, result.depth, from_score(result.score), result.best_move.map_or(String::from("None"), |m| m.to_string()));
    std::io::stdout().flush().unwrap();
}

fn main() {
    let mut engine = Searcher::new();

    let mut board = Board::default();

    let mut turn = 0;
    let mut game = String::from("");
    let mut deadline;
    for _ in 0..500 {
        if board.side_to_move() == Color::White {
            turn += 1;
            game += &format!("{}. ", turn);
        }

        match board.side_to_move() {
            Color::White => print!("White to move:\n"),
            Color::Black => print!("Black to move:\n"),
        }

        deadline = Deadline::timeout(Duration::from_millis(2000));
        print_result("Init   ", engine.min_search(&board));
        while let Some(_) = engine.iterative_deepening_search(&board, &deadline) {
            print_result("Iter   ", engine.min_search(&board));
        }

        // let depth = engine.cached_eval(&board).unwrap().depth;
        // println!("selecting");
        if let Some(choice) = engine.best_move(&board) {
            print_result("Final  ", engine.min_search(&board));
            print!("\n");
            print!("-----------------------------------------------------------------------------\n");
            game += &format!("{} ", choice);
            print!("{}\n", game);
            print!("-----------------------------------------------------------------------------\n");
            print!("Memory Bytes: {}\n", engine.memory_bytes());
            print!("\n\n");
            board = board.make_move_new(choice);
        } else {
            print!("\n");
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
    println!("Game: {}", game);

}
