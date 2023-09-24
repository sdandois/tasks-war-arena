use rand::Rng;
use rand::SeedableRng;
use std::collections::BinaryHeap;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::MutexGuard;

use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;

use crate::game::Direction;
use crate::game::Game;
use crate::game::TaskId;

enum Message {
    None,
}

#[cfg(test)]
mod tests;

struct GameResult();

struct GameRunner {
    tokio_rt: tokio::runtime::Runtime,
}

type WrappedGame = Arc<Mutex<Game>>;

struct TaskContext {
    used_fuel: isize,
    task_id: TaskId,
}

struct TaskRunner {
    task_id: TaskId,
    game: WrappedGame,
    rx: mpsc::Receiver<Message>,
    tx: mpsc::Sender<Message>,
    context: Arc<Mutex<TaskContext>>,
}

struct TaskHandle {
    context: Arc<Mutex<TaskContext>>,
    task_id: TaskId,
    tx: mpsc::Sender<Message>,
    handle: JoinHandle<()>,
}

struct RunnerContext {
    handles: BinaryHeap<TaskHandle>,
    continue_game_rx: mpsc::Receiver<Message>,
    continue_game_tx: Sender<Message>,
    game: Arc<Mutex<Game>>,
    tasks: Vec<TaskId>,
}

impl TaskRunner {
    fn new(
        context: Arc<Mutex<TaskContext>>,
        game: WrappedGame,
        tx: mpsc::Sender<Message>,
        rx: mpsc::Receiver<Message>,
    ) -> TaskRunner {
        TaskRunner {
            task_id: context.lock().unwrap().task_id,
            game,
            rx,
            tx,
            context: context.clone(),
        }
    }
    async fn run(&mut self) {
        while let Some(_content) = self.rx.recv().await {
            println!(
                "{:?} used fuel {}",
                self.task_id,
                self.borrow_context().used_fuel
            );
            self.borrow_context().used_fuel += 1;

            self.do_play();

            self.tx.send(Message::None).await.unwrap();
        }
    }

    fn do_play(&mut self) {
        let task_context = self.borrow_context();
        let command = self.strategy_command();

        let mut rng = rand::rngs::SmallRng::seed_from_u64(34);

        let random_dir = match rng.gen_range(0..3) {
            0 => Direction::Down,
            1 => Direction::Left,
            2 => Direction::Right,
            3 => Direction::Up,
            _ => panic!(),
        };

        let random_delta = rng.gen_range(0..16);

        match command {
            () => {
                self.borrow_game()
                    .move_task(task_context.task_id, random_delta, random_dir);
            }
        }

        println!(
            "{:?} : {:?}",
            self.task_id,
            self.borrow_game().get_task(self.task_id).pos
        );
    }

    fn strategy_command(&self) {}

    fn borrow_context(&self) -> MutexGuard<'_, TaskContext> {
        self.context.lock().unwrap()
    }

    fn borrow_game(&self) -> MutexGuard<'_, Game> {
        self.game.lock().unwrap()
    }
}

impl GameRunner {
    fn new() -> GameRunner {
        GameRunner {
            tokio_rt: tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap(),
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

            let handle = tokio::spawn(async move {
                let mut task_runner =
                    TaskRunner::new(task_context_copy, game_copy, continue_tx, rx);

                task_runner.run().await;
            });

            runner_context.handles.push(TaskHandle {
                context: task_context,
                task_id: *t,
                tx,
                handle,
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
        self.continue_game_tx.send(Message::None).await.unwrap();

        while let Some((_content, next_task)) =
            self.continue_game_rx.recv().await.zip(self.handles.pop())
        {
            if self.borrow_game().is_finished() {
                break;
            }

            if !self.borrow_game().get_task(next_task.task_id).is_dead {
                let tx = next_task.tx.clone();
                self.handles.push(next_task);
                tx.send(Message::None).await.unwrap();
            } else {
                println!("{:?} has been found dead", next_task.task_id);
                self.continue_game_tx.send(Message::None).await.unwrap();
            }
        }
        println!("End of rounds {}", self.handles.len());
    }

    fn borrow_game(&self) -> MutexGuard<'_, Game> {
        self.game.lock().unwrap()
    }
}

impl PartialOrd for TaskHandle {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.context
            .lock()
            .unwrap()
            .used_fuel
            .partial_cmp(&other.context.lock().unwrap().used_fuel)
            .map(|o| o.reverse())
    }
}

impl Ord for TaskHandle {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.context
            .lock()
            .unwrap()
            .used_fuel
            .cmp(&other.context.lock().unwrap().used_fuel)
            .reverse()
    }
}

impl PartialEq for TaskHandle {
    fn eq(&self, other: &Self) -> bool {
        self.context.lock().unwrap().used_fuel == other.context.lock().unwrap().used_fuel
    }
}

impl Eq for TaskHandle {}
