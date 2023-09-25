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

pub struct TaskRunner<B: Bot> {
    task_id: TaskId,
    game: WrappedGame,
    rx: mpsc::Receiver<Message>,
    tx: mpsc::Sender<Message>,
    context: Arc<Mutex<TaskContext>>,
    bot: B,
}

pub struct RandomBot {
    rng: rand::rngs::SmallRng
}

pub trait Bot {
    fn new() -> Self;
    fn poll(&mut self) -> Command;
    fn update(&mut self);
}

impl Bot for RandomBot {
    fn new() -> Self {
        RandomBot {
            // rng: rand::rngs::SmallRng::seed_from_u64(34)
            rng: rand::rngs::SmallRng::from_entropy()
        }
    }

    fn poll(&mut self) -> Command {

        let random_dir = match self.rng.gen_range(0..3) {
            0 => Direction::Down,
            1 => Direction::Left,
            2 => Direction::Right,
            3 => Direction::Up,
            _ => panic!(),
        };

        let random_delta = self.rng.gen_range(0..16);

        Command::Move(random_delta, random_dir)
    }

    fn update(&mut self) {
        todo!()
    }
}

pub enum Command {
    Move(usize, Direction),
}

impl<B: Bot> TaskRunner<B> {
    pub(super) fn new(
        context: Arc<Mutex<TaskContext>>,
        game: WrappedGame,
        tx: mpsc::Sender<Message>,
        rx: mpsc::Receiver<Message>,
    ) -> TaskRunner<B> {
        TaskRunner {
            task_id: context.lock().unwrap().task_id,
            game,
            rx,
            tx,
            context: context.clone(),
            bot: B::new(),
        }
    }
    pub async fn run(&mut self) {
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
        let command = self.bot.poll();

        let task_context = self.borrow_context();

        match command {
            Command::Move(random_delta, random_dir) => {
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


    fn borrow_context(&self) -> MutexGuard<'_, TaskContext> {
        self.context.lock().unwrap()
    }

    fn borrow_game(&self) -> MutexGuard<'_, Game> {
        self.game.lock().unwrap()
    }
}
