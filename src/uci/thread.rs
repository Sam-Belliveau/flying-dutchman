use std::{
    sync::{Arc, Mutex},
    thread,
};

use chess::Board;

use crate::search::{deadline::Deadline, engine::Engine};

use super::display;

pub struct UCIThread {
    deadline: Arc<Deadline>,
    board: Board,
    engine: Arc<Mutex<Engine>>,
    search_thread: Option<thread::JoinHandle<()>>,
}

impl UCIThread {
    pub fn new() -> UCIThread {
        UCIThread {
            deadline: Arc::new(Deadline::none()),
            board: Board::default(),
            engine: Arc::new(Mutex::new(Engine::new())),
            search_thread: None,
        }
    }

    pub fn search_thread(
        engine: Arc<Mutex<Engine>>,
        board: Board,
        deadline: Arc<Deadline>,
        print_result: bool,
    ) -> thread::JoinHandle<()> {
        thread::spawn(move || match engine.lock() {
            Ok(mut engine) => {
                let start = engine.start_new_search();

                if print_result {
                    display::board_information(&mut engine, board, start);
                }

                while let Ok(..) = engine.iterative_deepening_search(&board, &deadline) {
                    if print_result {
                        display::board_information(&mut engine, board, start);
                    }
                }

                if print_result {
                    display::board_information(&mut engine, board, start);
                    display::board_best_move(&mut engine, board);
                }
            }
            Err(_) => {
                panic!("Engine lock failed")
            }
        })
    }

    pub fn search(&mut self, board: Board, deadline: Deadline) {
        if let Some(thread) = self.search_thread.take() {
            if !thread.is_finished() {
                panic!("Search thread already running");
            }
        }

        self.deadline = Arc::new(deadline);
        self.board = board;

        self.search_thread = Some(Self::search_thread(
            Arc::clone(&self.engine),
            self.board,
            Arc::clone(&self.deadline),
            true,
        ));
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
