use std::sync::Arc;
use std::sync::Mutex;
use std::sync::MutexGuard;

use tokio::sync::mpsc;

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

            let task_response = self.do_play().await;

            match task_response {
                Some(resp) => self.tx.send(resp).await.unwrap(),
                None => {  
                    println!("{:?} no more to poll.", self.task_id);
                    break;
                }
            }
        }
        println!("{:?} channel closed, awaiting bot...", self.task_id);
        self.bot.wait().await;
        println!("{:?} channel closed, awaiting bot... Done.", self.task_id);
    }

    async fn do_play(&mut self) -> Option<TaskResponse> {
        let command = self.bot.poll().await?;

        let task_id = {
            let task_context = self.borrow_context();

            task_context.task_id
        };
        let res = match command {
            Command::Move(random_delta, random_dir) => {
                self.borrow_game()
                    .move_task(task_id, random_delta, random_dir);
                self.bot.update(None).await;

                TaskResponse::default()
            }
            Command::Split => {
                let res = self.borrow_game().split(task_id);

                self.bot.update(None).await;
                match res {
                    Ok(tid) => TaskResponse::NewTask(tid),
                    _ => TaskResponse::None,
                }
            }
            Command::Look(delta_x, delta_y) => {
                let res = self.borrow_game().look(task_id, delta_x, delta_y);

                self.bot.update(Some(res)).await;

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

        Some(res)
    }

    fn borrow_context(&self) -> MutexGuard<'_, TaskContext> {
        self.context.lock().unwrap()
    }

    fn borrow_game(&self) -> MutexGuard<'_, Game> {
        self.game.lock().unwrap()
    }
}
