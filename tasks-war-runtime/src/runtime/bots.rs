use rand::Rng;
use rand::SeedableRng;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::Arc;

use crate::game::Direction;
use crate::game::LookResult;
use crate::game::TaskId;

#[derive(Debug, Clone)]
pub enum Command {
    Move(usize, Direction),
    Look(isize, isize),
    Split,
    Pass,
}

pub trait Bot: Send {
    fn poll(&mut self) -> Command;
    fn update(&mut self, result: Option<LookResult>);
}

pub struct RandomBot {
    rng: rand::rngs::SmallRng,
    weight: usize,
}

impl RandomBot {
    fn new(seed: u64) -> Self {
        RandomBot {
            rng: rand::rngs::SmallRng::seed_from_u64(seed),
            weight: 64,
        }
    }
}
impl Bot for RandomBot {
    fn poll(&mut self) -> Command {
        let random_dir = match self.rng.gen_range(0..4) {
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

    fn update(&mut self, result: Option<LookResult>) {}
}

pub struct MockBot {
    previous_result: Option<LookResult>,
    commands: VecDeque<Arc<dyn Fn(Option<LookResult>) -> Command + Sync + Send + 'static>>,
}

impl MockBot {
    fn new(tid: TaskId) -> Self {
        MockBot {
            commands: VecDeque::default(),
            previous_result: None,
        }
    }
}

impl Bot for MockBot {
    fn poll(&mut self) -> Command {
        self.commands
            .pop_front()
            .unwrap_or(Arc::new(|_| Command::Pass))(self.previous_result)
    }

    fn update(&mut self, result: Option<LookResult>) {
        self.previous_result = result;
    }
}

pub trait BotFactory: Clone {
    type B: Bot;

    fn create_bot(&self, task_id: TaskId) -> Self::B;
}

type CommandClosure = dyn Fn(Option<LookResult>) -> Command + Send + Sync;

#[derive(Clone)]
pub struct MockedBotFactory {
    commands: HashMap<TaskId, VecDeque<Arc<CommandClosure>>>,
}

impl MockedBotFactory {
    pub fn new() -> MockedBotFactory {
        let mut commands = HashMap::new();

        commands.insert(TaskId(0, 0), VecDeque::default());
        commands.insert(TaskId(1, 0), VecDeque::default());

        MockedBotFactory { commands: commands }
    }

    pub fn mock(mut self, tid: TaskId, command: Command) -> Self {
        let closure = move |_| command.clone();
        self.mock_fn(tid, closure)
    }

    pub fn mock_fn<F>(mut self, tid: TaskId, closure: F) -> Self
    where
        F: Fn(Option<LookResult>) -> Command + Send + Sync + Clone + 'static,
    {
        if !self.commands.contains_key(&tid) {
            self.commands.insert(tid, VecDeque::default());
        }
        let queue = self.commands.get_mut(&tid).unwrap();

        queue.push_back(Arc::new(closure.clone()));

        self
    }
}

impl BotFactory for MockedBotFactory {
    type B = MockBot;
    fn create_bot(&self, task_id: TaskId) -> MockBot {
        let commands_copy = self.commands.get(&task_id).unwrap().clone();
        MockBot {
            commands: commands_copy,
            previous_result: None,
        }
    }
}

#[derive(Clone)]
pub struct RandomBotFactory;

impl RandomBotFactory {
    pub fn new() -> RandomBotFactory {
        RandomBotFactory
    }
}

impl BotFactory for RandomBotFactory {
    type B = RandomBot;
    fn create_bot(&self, tid: TaskId) -> RandomBot {
        let seed = (tid.0 + 2 * tid.1) as u64;

        RandomBot::new(seed)
    }
}
