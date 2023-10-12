use std::collections::BinaryHeap;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::MutexGuard;

use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use crate::game::Game;
use crate::game::TaskId;

pub mod bots;
mod task_runner;
#[cfg(test)]
mod tests;

use bots::BotFactory;
use task_runner::*;

const MAX_FUEL: isize = 15000;

pub type GameResult = Game;

pub struct GameRunner<F: BotFactory> {
    tokio_rt: tokio::runtime::Runtime,
    bot_factory: F,
}

type WrappedGame = Arc<Mutex<Game>>;

struct RunnerContext<F: BotFactory> {
    handles: BinaryHeap<TaskHandle>,
    game: Arc<Mutex<Game>>,
    tasks: Vec<TaskId>,
    max_turns: usize,
    bot_factory: F,
}

struct TaskHandle {
    context: Arc<Mutex<TaskContext>>,
    task_id: TaskId,
    tx: mpsc::Sender<TaskRequest>,
    rx: mpsc::Receiver<TaskResponse>,
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

#[derive(Clone)]
struct TaskContext {
    used_fuel: isize,
    task_id: TaskId,
}

impl<F: BotFactory + 'static> GameRunner<F> {
    pub fn new(bot_factory: F) -> GameRunner<F> {
        GameRunner {
            tokio_rt: tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap(),
            bot_factory,
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
        let mut runner_context = RunnerContext::new(self.bot_factory.clone(), max_rounds);

        runner_context.spawn_all_tasks().await;

        runner_context.play_rounds().await;
        runner_context.await_tasks_finish().await;

        let g = runner_context.borrow_game().clone();

        g
    }
}

impl<F: BotFactory + 'static> RunnerContext<F> {
    fn new(bot_factory: F, max_turns: usize) -> RunnerContext<F> {
        let game: WrappedGame = Arc::new(Mutex::new(Game::from_seed(32)));
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
                println!("Task  finished with error: {:?} ", res.unwrap_err());
            }
        }

        println!("handles joined")
    }

    async fn play_rounds(&mut self) {
        let mut time_counter = 1;

        while let Some(mut next_task) = self.handles.pop() {
            if self.borrow_game().is_finished() || time_counter > self.max_turns {
                self.handles.push(next_task);
                break;
            }

            next_task.tx.send(TaskRequest::None).await.unwrap();
            let message = next_task.rx.recv().await;

            if let Some(TaskResponse::NewTask(tid)) = message {
                self.spawn_task(tid, next_task.context.lock().unwrap().used_fuel)
                    .await;
            }

            if let None = message {
                println!("{:?} has panicked", next_task.task_id);
                std::mem::drop(next_task.tx);
                let _ = next_task.handle.await;
            } else if next_task.context.lock().unwrap().used_fuel > MAX_FUEL {
                println!("{:?} has run out of fuel", next_task.task_id);
                self.borrow_game().kill(next_task.task_id);
                std::mem::drop(next_task.tx);
                let _ = next_task.handle.await;
            } else if self.borrow_game().get_task(next_task.task_id).is_dead {
                println!("{:?} has been found dead", next_task.task_id);
                std::mem::drop(next_task.tx);
                let _ = next_task.handle.await;
            } else {
                next_task.timestamp = time_counter;
                self.handles.push(next_task);
            }
            time_counter += 1;
        }
        println!("End of rounds {:?}", self.handles);
    }

    fn borrow_game(&self) -> MutexGuard<'_, Game> {
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
