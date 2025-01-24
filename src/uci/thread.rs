use std::{
    sync::{Arc, Mutex},
    thread,
};

use crate::search::deadline::Deadline;
use crate::search::engine::Engine;
use crate::search::{board_history::BoardHistory, opponent_engine::OpponentEngine};

use crate::uci::display;

pub struct UCIThread {
    deadline: Arc<Deadline>,
    engine: Arc<Mutex<Engine>>,
    opponent_engine: Arc<Mutex<Option<OpponentEngine>>>,
    search_thread: Option<thread::JoinHandle<()>>,
}

impl UCIThread {
    pub fn new() -> UCIThread {
        UCIThread {
            deadline: Arc::new(Deadline::none()),
            engine: Arc::new(Mutex::new(Engine::new())),
            opponent_engine: Arc::new(Mutex::new(Some(OpponentEngine::new().unwrap()))),
            search_thread: None,
        }
    }

    pub fn reset(&mut self) {
        self.stop();
    }

    pub fn search_thread(
        engine: Arc<Mutex<Engine>>,
        opponent_engine: Arc<Mutex<Option<OpponentEngine>>>,
        history: BoardHistory,
        deadline: Arc<Deadline>,
        print_result: bool,
    ) -> thread::JoinHandle<()> {
        thread::spawn(move || match (engine.lock(), opponent_engine.lock()) {
            (Ok(mut engine), Ok(mut opponent_engine)) => {
                let start = engine.start_new_search();

                if print_result {
                    display::board_information(&mut engine, &history, start);
                }

                while let Ok(..) =
                    engine.iterative_deepening_search(&history, &deadline, &mut opponent_engine)
                {
                    if print_result {
                        display::board_information(&mut engine, &history, start);
                    }
                }

                if print_result {
                    display::board_information(&mut engine, &history, start);
                    display::board_best_move(&mut engine, &history);
                }
            }
            _ => {
                panic!("Engine lock failed")
            }
        })
    }

    pub fn search(&mut self, history: &BoardHistory, deadline: Deadline) {
        if let Some(thread) = self.search_thread.take() {
            thread.join().unwrap();
        }

        self.deadline = Arc::new(deadline);

        self.search_thread = Some(Self::search_thread(
            Arc::clone(&self.engine),
            Arc::clone(&self.opponent_engine),
            history.clone(),
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
