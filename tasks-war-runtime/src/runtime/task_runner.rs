use rand::Rng;
use rand::SeedableRng;
use std::default;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::MutexGuard;

use tokio::sync::mpsc;

use crate::game::Direction;
use crate::game::Game;
use crate::game::TaskId;

use super::{TaskContext, WrappedGame};

use super::bots::{Bot, Command};

#[derive(Default)]
pub enum TaskRequest {
    #[default]
    None,
}

#[derive(Default)]
pub enum TaskResponse {
    #[default]
    None,
    NewTask(TaskId),
    Panicked,
}

pub struct TaskRunner<B: Bot> {
    task_id: TaskId,
    game: WrappedGame,
    rx: mpsc::Receiver<TaskRequest>,
    tx: mpsc::Sender<TaskResponse>,
    context: Arc<Mutex<TaskContext>>,
    bot: B,
}

impl<B: Bot> TaskRunner<B> {
    pub(super) fn new(
        context: Arc<Mutex<TaskContext>>,
        game: WrappedGame,
        tx: mpsc::Sender<TaskResponse>,
        rx: mpsc::Receiver<TaskRequest>,
        bot: B,
    ) -> TaskRunner<B> {
        let task_id = context.lock().unwrap().task_id.clone();
        TaskRunner {
            task_id,
            game,
            rx,
            tx,
            context: context.clone(),
            bot,
        }
    }
    pub async fn run(&mut self) {
        while let Some(_content) = self.rx.recv().await {
            println!(
                "{:?}: at {:?} with {} of fuel",
                self.task_id,
                self.borrow_game().get_task(self.task_id).pos,
                self.borrow_context().used_fuel
            );
            self.borrow_context().used_fuel += 1;

            let task_response = self.do_play();

            self.tx.send(task_response).await.unwrap();
        }
    }

    fn do_play(&mut self) -> TaskResponse {
        let command = self.bot.poll();

        let task_id = {
            let task_context = self.borrow_context();

            task_context.task_id
        };
        let res = match command {
            Command::Move(random_delta, random_dir) => {
                self.borrow_game()
                    .move_task(task_id, random_delta, random_dir);
                self.bot.update(None);

                TaskResponse::default()
            }
            Command::Split => {
                let res = self.borrow_game().split(task_id);

                self.bot.update(None);
                match res {
                    Ok(tid) => TaskResponse::NewTask(tid),
                    _ => TaskResponse::None,
                }
            }
            Command::Look(delta_x, delta_y) => {
                let res = self.borrow_game().look(task_id, delta_x, delta_y);

                self.bot.update(Some(res));

                TaskResponse::None
            }
            Command::Pass => TaskResponse::None,
        };

        println!(
            "{:?}: after {:?} at {:?}",
            self.task_id,
            command,
            self.borrow_game().get_task(self.task_id).pos
        );

        res
    }

    fn borrow_context(&self) -> MutexGuard<'_, TaskContext> {
        self.context.lock().unwrap()
    }

    fn borrow_game(&self) -> MutexGuard<'_, Game> {
        self.game.lock().unwrap()
    }
}

impl<B: Bot> Drop for TaskRunner<B> {
    fn drop(&mut self) {
        if std::thread::panicking() {
            let tx_copy = self.tx.clone();
            let _ = tokio::spawn(async move {
                tx_copy.clone().send(TaskResponse::Panicked).await.unwrap();
            });
        }
    }
}
