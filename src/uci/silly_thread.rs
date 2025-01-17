use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    thread,
};

use crate::{search::{
    alpha_beta::AlphaBeta, board_history::BoardHistory, deadline::Deadline, engine::Engine, movegen::OrderedMoveGen,
}, transposition::best_moves::BestMoves};

use super::display;
use chess::MoveGen;

pub struct UCISillyThread {
    deadline: Arc<Deadline>,
    engine: Arc<Mutex<(Engine, Engine)>>,
    search_thread: Option<thread::JoinHandle<()>>,
}

impl UCISillyThread {
    pub fn new() -> UCISillyThread {
        UCISillyThread {
            deadline: Arc::new(Deadline::none()),
            engine: Arc::new(Mutex::new((Engine::new(), Engine::new()))),
            search_thread: None,
        }
    }

    pub fn reset(&mut self) {
        self.stop();
    }

    pub fn search_thread(
        engine: Arc<Mutex<(Engine, Engine)>>,
        history: BoardHistory,
        deadline: Arc<Deadline>,
        print_result: bool,
    ) -> thread::JoinHandle<()> {
        thread::spawn(move || match engine.lock() {
            Ok(mut engines) => {
                let (ref mut engine, ref mut human_engine) = *engines;

                let start = engine.start_new_search();

                if print_result {
                    display::board_information(engine, &history, start);
                }

                let mut forced_plays = HashMap::new();

                let mut window = AlphaBeta::new();
                for movement in OrderedMoveGen::full_search(history.last(), BestMoves::Empty) {
                    let next_board = history.with_move(movement);

                    if let Ok(opponent_response) =
                        human_engine.ab_search::<true>(&next_board, 7, window, &deadline)
                    {
                        window.negamax(-opponent_response.score());
                        if let Some(opponent_move) = opponent_response.peek() {
                            forced_plays.insert(next_board, opponent_move.clone());
                        }
                    } else {
                        break;
                    }
                }

                engine.forced_plays = forced_plays;
                while let Ok(..) = engine.iterative_deepening_search(&history, &deadline) {
                    if print_result {
                        display::board_information(engine, &history, start);
                    }
                }

                if print_result {
                    display::board_information(engine, &history, start);
                    display::board_best_move(engine, &history);
                }
            }
            Err(_) => {
                panic!("Engine lock failed")
            }
        })
    }

    pub fn search(&mut self, history: &BoardHistory, deadline: Deadline) {
        if let Some(thread) = self.search_thread.take() {
            if !thread.is_finished() {
                panic!("Search thread already running");
            }
        }

        self.deadline = Arc::new(deadline);

        self.search_thread = Some(Self::search_thread(
            Arc::clone(&self.engine),
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
