use rand::Rng;
use rand::SeedableRng;
use std::collections::BinaryHeap;
use std::marker::PhantomData;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::MutexGuard;

use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;

use crate::game::Direction;
use crate::game::Game;
use crate::game::TaskId;

mod bots;
mod task_runner;
#[cfg(test)]
mod tests;

use bots::{Bot, BotFactory};
use task_runner::*;

const MAX_FUEL: isize = 15000;

enum Message {
    None,
}

struct GameResult();

struct GameRunner<F: BotFactory> {
    tokio_rt: tokio::runtime::Runtime,
    bot_factory: F,
}

type WrappedGame = Arc<Mutex<Game>>;

struct RunnerContext {
    handles: BinaryHeap<TaskHandle>,
    continue_game_rx: mpsc::Receiver<Message>,
    continue_game_tx: Sender<Message>,
    game: Arc<Mutex<Game>>,
    tasks: Vec<TaskId>,
}

struct TaskHandle {
    context: Arc<Mutex<TaskContext>>,
    task_id: TaskId,
    tx: mpsc::Sender<Message>,
    handle: JoinHandle<()>,
    timestamp: usize,
}

impl core::fmt::Debug for TaskHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TaskHandle")
            .field("task_id", &self.task_id)
            .field("used_fuel", &self.context.lock().unwrap().used_fuel)
            .field("timestamp", &self.timestamp)
            .finish()
    }
}

struct TaskContext {
    used_fuel: isize,
    task_id: TaskId,
}

impl<F: BotFactory + 'static> GameRunner<F> {
    fn new(bot_factory: F) -> GameRunner<F> {
        GameRunner {
            tokio_rt: tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap(),
            bot_factory,
        }
    }

    fn run_game(&self) -> GameResult {
        let future = self.start_game();
        let game_result = self.tokio_rt.block_on(future);

        game_result
    }

    async fn start_game(&self) -> GameResult {
        let mut runner_context = RunnerContext::new();

        for t in &runner_context.tasks {
            let (tx, rx): (mpsc::Sender<Message>, mpsc::Receiver<Message>) = mpsc::channel(32);
            let continue_tx = runner_context.continue_game_tx.clone();
            let game_copy = runner_context.game.clone();
            let task_context = Arc::new(Mutex::new(TaskContext {
                used_fuel: 0,
                task_id: *t,
            }));
            let task_context_copy = task_context.clone();
            let bot = self.bot_factory.create_bot(*t);

            let handle = tokio::spawn(async move {
                let mut task_runner =
                    TaskRunner::new(task_context_copy, game_copy, continue_tx, rx, bot);

                task_runner.run().await;
            });

            runner_context.handles.push(TaskHandle {
                context: task_context,
                task_id: *t,
                tx,
                handle,
                timestamp: 0,
            })
        }

        runner_context.play_rounds().await;
        runner_context.await_tasks_finish().await;

        GameResult()
    }
}

impl RunnerContext {
    fn new() -> RunnerContext {
        let (continue_game_tx, continue_game_rx): (mpsc::Sender<Message>, mpsc::Receiver<Message>) =
            mpsc::channel(32);

        let game: WrappedGame = Arc::new(Mutex::new(Game::from_seed(32)));
        let tasks = game.lock().unwrap().get_all_task_ids();

        RunnerContext {
            handles: BinaryHeap::new(),
            continue_game_rx,
            continue_game_tx,
            game,
            tasks,
        }
    }

    async fn await_tasks_finish(&mut self) {
        let thread_handles: Vec<_> = std::mem::take(&mut self.handles)
            .into_iter()
            .map(|h| h.handle)
            .collect();

        for i in thread_handles {
            let _ = i.await;
        }

        println!("handles joined")
    }

    async fn play_rounds(&mut self) {
        let mut time_counter = 1;

        while let Some(mut next_task) = self.handles.pop() {
            if self.borrow_game().is_finished() {
                break;
            }

            next_task.tx.send(Message::None).await.unwrap();
            self.continue_game_rx.recv().await;

            if next_task.context.lock().unwrap().used_fuel > MAX_FUEL {
                println!("{:?} has run out of fuel", next_task.task_id);
                self.borrow_game().kill(next_task.task_id);
            } else if self.borrow_game().get_task(next_task.task_id).is_dead {
                println!("{:?} has been found dead", next_task.task_id);
            } else {
                next_task.timestamp = time_counter;
                self.handles.push(next_task);
                println!("{:?}", self.handles);
            }
            time_counter += 1;
        }
        println!("End of rounds {:?}", self.handles);
    }

    fn borrow_game(&self) -> MutexGuard<'_, Game> {
        self.game.lock().unwrap()
    }
}

impl PartialOrd for TaskHandle {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TaskHandle {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let task_context = self.context.lock().unwrap();

        let ordering = task_context
            .used_fuel
            .cmp(&other.context.lock().unwrap().used_fuel);

        let comparison = match ordering {
            std::cmp::Ordering::Equal => self.timestamp.cmp(&other.timestamp),
            _ => ordering,
        };

        comparison.reverse()
    }
}

impl PartialEq for TaskHandle {
    fn eq(&self, other: &Self) -> bool {
        self.context.lock().unwrap().used_fuel == other.context.lock().unwrap().used_fuel
    }
}

impl Eq for TaskHandle {}
