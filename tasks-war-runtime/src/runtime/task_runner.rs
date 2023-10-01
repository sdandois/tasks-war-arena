use rand::Rng;
use rand::SeedableRng;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::MutexGuard;

use tokio::sync::mpsc;

use crate::game::Direction;
use crate::game::Game;
use crate::game::TaskId;

use super::{Message, TaskContext, WrappedGame};

use super::bots::{Bot, Command};

pub struct TaskRunner<B: Bot> {
    task_id: TaskId,
    game: WrappedGame,
    rx: mpsc::Receiver<Message>,
    tx: mpsc::Sender<Message>,
    context: Arc<Mutex<TaskContext>>,
    bot: B,
}

impl<B: Bot> TaskRunner<B> {
    pub(super) fn new(
        context: Arc<Mutex<TaskContext>>,
        game: WrappedGame,
        tx: mpsc::Sender<Message>,
        rx: mpsc::Receiver<Message>,
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

            self.do_play();

            self.tx.send(Message::None).await.unwrap();
        }
    }

    fn do_play(&mut self) {
        let command = self.bot.poll();

        let task_context = self.borrow_context();
        let task_id = task_context.task_id;
        match command {
            Command::Move(random_delta, random_dir) => {
                self.borrow_game()
                    .move_task(task_id, random_delta, random_dir);
            }
            Command::Split => {
                self.borrow_game().split(task_id);
            }
        }

        println!(
            "{:?}: after {:?} at {:?}",
            self.task_id,
            command,
            self.borrow_game().get_task(self.task_id).pos
        );
    }

    fn borrow_context(&self) -> MutexGuard<'_, TaskContext> {
        self.context.lock().unwrap()
    }

    fn borrow_game(&self) -> MutexGuard<'_, Game> {
        self.game.lock().unwrap()
    }
}
