use std::sync::Arc;
use std::sync::Mutex;
use std::sync::MutexGuard;

use tokio::sync::mpsc;

use tracing::{event, Level};

use crate::game::TaskId;
use crate::command::CommandResponse;
use crate::game_with_history::GameWithHistory;

use super::{TaskContext, WrappedGame};

use super::bots::Bot;

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
            event!(Level::INFO,
                "{:?}: at {:?} with {} of fuel",
                self.task_id,
                self.borrow_game().get_task(self.task_id).pos,
                self.borrow_context().used_fuel
            );

            let task_response = self.do_play().await;

            match task_response {
                Some(resp) => self.tx.send(resp).await.unwrap(),
                None => {  
                    event!(Level::INFO,"{:?} no more to poll.", self.task_id);
                    break;
                }
            }
        }
        event!(Level::INFO,"{:?} channel closed, awaiting bot...", self.task_id);
        self.bot.wait().await;
        event!(Level::INFO,"{:?} channel closed, awaiting bot... Done.", self.task_id);
    }

    async fn do_play(&mut self) -> Option<TaskResponse> {
        let (command, consumed_fuel) = self.bot.poll().await?;
        self.borrow_context().used_fuel += consumed_fuel as isize;

        let task_id = {
            let task_context = self.borrow_context();

            task_context.task_id
        };

        let res = self.borrow_game().accept(task_id, &command);

        self.bot_update(&res).await;

        let task_response = match res {
            CommandResponse::NewTask(tid) => TaskResponse::NewTask(tid),
            _ => TaskResponse::None,
        };

        event!(Level::INFO,
            "{:?}: after {:?} at {:?}",
            self.task_id,
            command,
            self.borrow_game().get_task(self.task_id).pos
        );

        Some(task_response)
    }

    async fn bot_update(&mut self, command_resp: &CommandResponse) {
        match command_resp {
            CommandResponse::Look(look_result) => {
                self.bot.update(Some(*look_result)).await;
            }
            _ => {
                self.bot.update(None).await;
            }
        };
    }

    fn borrow_context(&self) -> MutexGuard<'_, TaskContext> {
        self.context.lock().unwrap()
    }

    fn borrow_game(&self) -> MutexGuard<'_, GameWithHistory> {
        self.game.lock().unwrap()
    }
}
