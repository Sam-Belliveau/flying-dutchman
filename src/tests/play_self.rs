use std::{io::Write, time::Duration};

use chess::{Board, Color};

use crate::evaluate::score_to_str;

use crate::search::board_history::BoardHistory;
use crate::search::deadline::Deadline;
use crate::search::engine::Engine;

use crate::transposition::table_entry::TTableEntry;

fn print_result(title: &str, result: TTableEntry) {
    println!(
        "{}|   Depth:{:16.3}   |   Move {}   |    Score {}",
        title,
        result.depth(),
        result
            .peek()
            .map_or(String::from("None"), |m| m.to_string()),
        score_to_str(result.score()),
    );
    std::io::stdout().flush().unwrap();
}

pub fn play_self() {
    let mut engine = Engine::new();
    engine.table.set_table_size(16000000000);

    let mut board = BoardHistory::new(Board::default());

    let mut turn = 0;
    let mut game = String::from("");
    let mut deadline;
    for _ in 0..500 {
        if board.last().side_to_move() == Color::White {
            turn += 1;
            game += &format!("{}. ", turn);
        }

        match board.last().side_to_move() {
            Color::White => println!("White to move:"),
            Color::Black => println!("Black to move:"),
        }

        deadline = Deadline::timeout(Duration::from_millis(500));
        print_result("Init   ", engine.min_search(&board));
        let mut rep = 0;
        let mut presult = None;
        while let Ok(result) = engine.iterative_deepening_search(&board, &deadline) {
            print_result("Iter   ", engine.min_search(&board));

            if presult == Some(result) {
                rep += 1;
                if rep > 10 {
                    break;
                }
            } else {
                rep = 0;
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
            board = board.with_move(choice);
        } else {
            println!();
            match board.last().status() {
                chess::BoardStatus::Checkmate => {
                    println!("Checkmate!");
                    match board.last().side_to_move() {
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
