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
    rng: rand::rngs::SmallRng,
    weight: usize,
}

pub trait Bot : Send {
    fn new(tid: TaskId) -> Self;
    fn poll(&mut self) -> Command;
    fn update(&mut self);
}

impl Bot for RandomBot {
    fn new(tid: TaskId) -> Self {
        RandomBot {
            rng: rand::rngs::SmallRng::seed_from_u64((tid.0 + 2 * tid.1) as u64),
            weight: 64,
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

        let coin_flip = self
            .rng
            .sample(rand::distributions::Bernoulli::new(0.1).unwrap());

        if coin_flip && self.weight > 1 {
            self.weight /= 2;
            Command::Split
        } else {
            let random_delta = self.rng.gen_range(0..16);

            Command::Move(random_delta, random_dir)
        }
    }

    fn update(&mut self) {
        todo!()
    }
}

pub enum Command {
    Move(usize, Direction),
    Split,
}

impl<B: Bot> TaskRunner<B> {
    pub(super) fn new(
        context: Arc<Mutex<TaskContext>>,
        game: WrappedGame,
        tx: mpsc::Sender<Message>,
        rx: mpsc::Receiver<Message>,
    ) -> TaskRunner<B> {
        let task_id = context.lock().unwrap().task_id.clone();
        TaskRunner {
            task_id,
            game,
            rx,
            tx,
            context: context.clone(),
            bot: B::new(task_id),
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

