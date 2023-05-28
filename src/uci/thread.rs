use std::{
    sync::{Arc, Mutex},
    thread,
};

use chess::Board;

use crate::{
    evaluate::score_to_cp,
    search::{deadline::Deadline, search::Searcher},
    uci::sync,
};

pub struct UCIThread {
    deadline: Arc<Deadline>,
    board: Board,
    engine: Arc<Mutex<Searcher>>,
    search_thread: Option<thread::JoinHandle<()>>,
}

impl UCIThread {
    pub fn new() -> UCIThread {
        UCIThread {
            deadline: Arc::new(Deadline::none()),
            board: Board::default(),
            engine: Arc::new(Mutex::new(Searcher::new())),
            search_thread: None,
        }
    }

    pub fn search(&mut self, board: Board, deadline: Deadline) {
        if let Some(thread) = &self.search_thread {
            if thread.is_finished() {
                self.search_thread = None;
            } else {
                panic!("Search thread already running");
            }
        }

        self.deadline = deadline.into();
        self.board = board;

        self.search_thread = Some({
            let engine = Arc::clone(&self.engine);
            let board = self.board;
            let deadline = Arc::clone(&self.deadline);
            thread::spawn(move || match engine.lock() {
                Ok(mut engine) => {
                    let mut reps = 0;
                    let mut presult = None;
                    while let Some(result) = engine.iterative_deepening_search(&board, &deadline) {
                        if presult == Some(result) {
                            reps += 1;

                            if reps >= 16 {
                                break;
                            }
                        } else {
                            presult = Some(result);
                        }

                        let info = engine.min_search(&board);
                        print!(
                            "info depth {} seldepth {} multipv 1 score cp {} pv",
                            info.depth,
                            info.depth,
                            score_to_cp(info.score)
                        );

                        let mut pv_board = board;
                        while let Some(chess_move) =
                            engine.opt_search(&pv_board).and_then(|f| f.best_move)
                        {
                            print!(" {}", chess_move);
                            pv_board = pv_board.make_move_new(chess_move);
                        }
                        println!();
                        sync();
                    }

                    if let Some(best_move) = engine.best_move(&board) {
                        let info = engine.min_search(&board);
                        println!(
                            "bestmove {} info depth {} score cp {}",
                            best_move,
                            info.depth,
                            score_to_cp(info.score)
                        );
                        sync();
                    }
                }
                Err(_) => {
                    eprintln!("Engine lock failed")
                }
            })
        });
    }

    pub fn stop(&mut self) {
        self.deadline.trigger();

        // Wait for the search thread to finish.
        if let Some(search_thread) = self.search_thread.take() {
            if let Err(err) = search_thread.join() {
                panic!("Search thread panicked: {:?}", err);
            }
        }
    }
}
