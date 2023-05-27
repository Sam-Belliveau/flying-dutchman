use std::{io::Write, time::Duration};

use chess::{Board, Color};

use crate::{
    evaluate::score_to_cp,
    search::{deadline::Deadline, search::Searcher},
    transposition::table_entry::TTableEntry,
};

fn print_result(title: &str, result: TTableEntry) {
    println!(
        "{}|   Depth:{:16.3}   |   Score{:16.3}   |   Move {}",
        title,
        result.depth,
        score_to_cp(result.score),
        result
            .best_move
            .map_or(String::from("None"), |m| m.to_string())
    );
    std::io::stdout().flush().unwrap();
}

pub fn play_self() {
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
            Color::White => println!("White to move:"),
            Color::Black => println!("Black to move:"),
        }

        deadline = Deadline::timeout(Duration::from_millis(2000));
        print_result("Init   ", engine.min_search(&board));
        let mut presult = None;
        while let Some(result) = engine.iterative_deepening_search(&board, &deadline) {
            print_result("Iter   ", engine.min_search(&board));

            if presult == Some(result) {
                break;
            } else {
                presult = Some(result);
            }
        }

        // let depth = engine.cached_eval(&board).unwrap().depth;
        // println!("selecting");
        if let Some(choice) = engine.best_move(&board) {
            print_result("Final  ", engine.min_search(&board));
            println!();
            println!(
                "-----------------------------------------------------------------------------"
            );
            game += &format!("{} ", choice);
            println!("{}", game);
            println!(
                "-----------------------------------------------------------------------------"
            );
            println!("Memory Bytes: {}", engine.memory_bytes());
            print!("\n\n");
            board = board.make_move_new(choice);
        } else {
            println!();
            match board.status() {
                chess::BoardStatus::Checkmate => {
                    println!("Checkmate!");
                    match board.side_to_move() {
                        Color::White => println!("Black wins!"),
                        Color::Black => println!("White wins!"),
                    }
                }
                chess::BoardStatus::Stalemate => {
                    println!("Stalemate!");
                }
                chess::BoardStatus::Ongoing => {
                    println!("Error!");
                }
            }
            break;
        }
    }
    println!("Game: {}", game);
}
