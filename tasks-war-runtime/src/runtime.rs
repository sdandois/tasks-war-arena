use std::collections::BinaryHeap;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::MutexGuard;

use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use tracing::{event, Level};

use crate::game::GameConfig;
use crate::game::TaskId;
use crate::game_replay::GameWithHistory;

pub mod bots;
mod task_handle;
mod task_runner;

#[cfg(test)]
mod tests;

use bots::BotFactory;
use task_handle::*;
use task_runner::*;

const MAX_FUEL: isize = 1500000;

pub type GameResult = GameWithHistory;

pub struct GameRunner<F: BotFactory> {
    tokio_rt: tokio::runtime::Runtime,
    bot_factory: F,
    config: GameConfig,
}

type WrappedGame = Arc<Mutex<GameWithHistory>>;

struct RunnerContext<F: BotFactory> {
    handles: BinaryHeap<TaskHandle>,
    game: WrappedGame,
    tasks: Vec<TaskId>,
    max_turns: usize,
    bot_factory: F,
}

impl<F: BotFactory + 'static> GameRunner<F> {
    pub fn new(bot_factory: F) -> GameRunner<F> {
        GameRunner::with_config(bot_factory, GameConfig::default())
    }

    pub fn with_config(bot_factory: F, config: GameConfig) -> GameRunner<F> {
        GameRunner {
            tokio_rt: tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap(),
            bot_factory,
            config,
        }
    }

    pub fn run_game(&self) -> GameResult {
        let future = self.start_game(usize::MAX);
        let game_result = self.tokio_rt.block_on(future);

        game_result
    }

    pub fn run_some_rounds(&self, rounds: usize) -> GameResult {
        let future = self.start_game(rounds);
        let game_result = self.tokio_rt.block_on(future);

        game_result
    }

    async fn start_game(&self, max_rounds: usize) -> GameResult {
        let mut runner_context =
            RunnerContext::new(self.bot_factory.clone(), max_rounds, self.config.clone());

        runner_context.spawn_all_tasks().await;

        runner_context.play_rounds().await;
        runner_context.await_tasks_finish().await;

        let g = runner_context.borrow_game().clone();

        g
    }
}

impl<F: BotFactory + 'static> RunnerContext<F> {
    fn new(bot_factory: F, max_turns: usize, config: GameConfig) -> RunnerContext<F> {
        let game: WrappedGame = Arc::new(Mutex::new(GameWithHistory::from_config(config)));
        let tasks = game.lock().unwrap().get_all_task_ids();

        RunnerContext {
            handles: BinaryHeap::new(),
            game,
            tasks,
            max_turns,
            bot_factory,
        }
    }

    async fn await_tasks_finish(&mut self) {
        let thread_handles: Vec<_> = std::mem::take(&mut self.handles)
            .into_iter()
            .map(|h| h.handle)
            .collect();

        for i in thread_handles {
            let res = i.await;

            if res.is_err() {
                event!(
                    Level::INFO,
                    "Task  finished with error: {:?} ",
                    res.unwrap_err()
                );
            }
        }

        event!(Level::INFO, "handles joined")
    }

    async fn play_rounds(&mut self) {
        let mut time_counter = 1;

        while let Some(mut next_task) = self.handles.pop() {
            if self.borrow_game().is_finished() || time_counter > self.max_turns {
                self.handles.push(next_task);
                break;
            }

            if self.borrow_game().get_task(next_task.task_id).is_dead {
                event!(Level::INFO, "{:?} has been found dead", next_task.task_id);
                std::mem::drop(next_task.tx);
                let _ = next_task.handle.await;
            } else {
                next_task.tx.send(TaskRequest::None).await.unwrap();
                let message = next_task.rx.recv().await;

                if let Some(TaskResponse::NewTask(tid)) = message {
                    self.spawn_task(tid, next_task.context.lock().unwrap().used_fuel)
                        .await;
                }

                if let None = message {
                    event!(Level::INFO, "{:?} has panicked", next_task.task_id);
                    std::mem::drop(next_task.tx);
                    let _ = next_task.handle.await;
                } else if next_task.context.lock().unwrap().used_fuel > MAX_FUEL {
                    event!(Level::INFO, "{:?} has run out of fuel", next_task.task_id);
                    self.borrow_game().kill(next_task.task_id);
                    std::mem::drop(next_task.tx);
                    let _ = next_task.handle.await;
                } else {
                    next_task.timestamp = time_counter;
                    self.handles.push(next_task);
                }
            }
            time_counter += 1;
        }
        event!(Level::INFO, "End of rounds {:?}", self.handles);
    }

    fn borrow_game(&self) -> MutexGuard<'_, GameWithHistory> {
        self.game.lock().unwrap()
    }

    async fn spawn_all_tasks(&mut self) {
        let tasks = std::mem::take(&mut self.tasks);

        for &t in &tasks {
            self.spawn_task(t, 0).await;
        }

        let _ = std::mem::replace(&mut self.tasks, tasks);
    }

    async fn spawn_task(&mut self, task_id: TaskId, used_fuel: isize) {
        let (tx, rx): (mpsc::Sender<TaskRequest>, mpsc::Receiver<TaskRequest>) = mpsc::channel(32);

        let (continue_tx, continue_rx): (mpsc::Sender<TaskResponse>, mpsc::Receiver<TaskResponse>) =
            mpsc::channel(32);

        let game_copy = self.game.clone();
        let task_context = Arc::new(Mutex::new(TaskContext { used_fuel, task_id }));
        let task_context_copy = task_context.clone();
        let bot = self.bot_factory.create_bot(task_id).await;

        let handle = tokio::spawn(async move {
            let mut task_runner =
                TaskRunner::new(task_context_copy, game_copy, continue_tx, rx, bot);

            task_runner.run().await;
        });

        self.handles.push(TaskHandle {
            context: task_context,
            task_id,
            tx,
            rx: continue_rx,
            handle,
            timestamp: 0,
        })
    }
}
