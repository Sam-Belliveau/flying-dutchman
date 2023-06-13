use std::{
    io::{self, BufRead},
    str::FromStr,
};

use chess::{Board, ChessMove};
use logos::Logos;

use crate::tests;

use super::{
    go_options::GoOptions,
    stdout_sync,
    thread::UCIThread,
    tokens::UCIToken::{self, *},
};

pub fn uci_loop() {
    let mut thread = UCIThread::new();
    let mut board = Board::default();

    let stdin = io::stdin();
    loop {
        let line = stdin.lock().lines().next().unwrap().unwrap();
        let mut lexer = UCIToken::lexer(&line);

        while let Some(token) = lexer.next() {
            match token {
                Ok(FlyingDutchmanTest) => {
                    tests::play_self::play_self();
                }
                Ok(Uci) => {
                    // Respond to the UCI identification command.
                    println!("id name Flying-Dutchman");
                    println!("id author Sam Belliveau");
                    println!("uciok");
                }
                Ok(NewGame) => {
                    thread.reset()
                }
                Ok(IsReady) => {
                    // Respond to the isready command.
                    println!("readyok");
                }
                Ok(Go) => {
                    // Start searching for a move.
                    let info = GoOptions::build(lexer.remainder().trim());
                    let deadline = info.to_deadline(&board);
                    for _ in lexer.by_ref() {}

                    thread.search(board, deadline);
                }
                Ok(Stop) => {
                    thread.stop();
                }
                Ok(Quit) => {
                    // Quit the application when the quit command is received.
                    thread.stop();
                    return;
                }
                Ok(Position) => {
                    let info = lexer.remainder().trim();
                    for _ in lexer.by_ref() {}

                    if info.starts_with("startpos") {
                        board = Board::default();
                        if info.starts_with("startpos moves") {
                            let moves = info.trim_start_matches("startpos moves ");
                            for chess_move in moves.split_whitespace() {
                                if let Ok(chess_move) = ChessMove::from_str(chess_move) {
                                    board = board.make_move_new(chess_move);
                                }
                            }
                        }
                    } else if info.starts_with("fen") {
                        let fen = info.trim_start_matches("fen ");
                        if let Ok(new_board) = Board::from_str(fen.trim()) {
                            board = new_board;
                        }

                        for chess_move in fen.split_whitespace() {
                            if let Ok(chess_move) = ChessMove::from_str(chess_move) {
                                board = board.make_move_new(chess_move);
                            }
                        }
                    }
                }
                Ok(SetOption) => {
                    eprintln!("unknown option: {}", lexer.remainder());
                }
                _ => {
                    eprintln!("unknown token: {}", lexer.slice());
                }
            }
        }

        stdout_sync();
    }
}
