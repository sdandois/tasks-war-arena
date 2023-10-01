use rand::Rng;
use rand::SeedableRng;
use std::collections::HashMap;
use std::collections::VecDeque;

use crate::game::Direction;
use crate::game::TaskId;

#[derive(Debug, Clone)]
pub enum Command {
    Move(usize, Direction),
    Split,
}

pub trait Bot: Send {
    fn poll(&mut self) -> Command;
    fn update(&mut self);
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

    fn update(&mut self) {
        todo!()
    }
}

pub struct MockBot {
    commands: VecDeque<Command>,
}

impl MockBot {
    fn new(tid: TaskId) -> Self {
        MockBot {
            commands: VecDeque::default(),
        }
    }
}

impl Bot for MockBot {
    fn poll(&mut self) -> Command {
        self.commands
            .pop_front()
            .unwrap_or(Command::Move(1, Direction::Left))
    }

    fn update(&mut self) {
        todo!()
    }
}

pub trait BotFactory {
    type B: Bot;

    fn create_bot(&self, task_id: TaskId) -> Self::B;
}

pub struct MockedBotFactory {
    commands: HashMap<TaskId, VecDeque<Command>>,
}

impl MockedBotFactory {
    pub fn new() -> MockedBotFactory {
        MockedBotFactory {
            commands: HashMap::new(),
        }
    }

    pub fn mock(mut self, tid: TaskId, command: Command) -> Self {
        if !self.commands.contains_key(&tid) {
            self.commands.insert(tid, VecDeque::default());
        }
        let queue: &mut VecDeque<Command> = self.commands.get_mut(&tid).unwrap();

        queue.push_back(command);

        self
    }
}

impl BotFactory for MockedBotFactory {
    type B = MockBot;
    fn create_bot(&self, task_id: TaskId) -> MockBot {
        let commands_copy: VecDeque<Command> = self.commands.get(&task_id).unwrap().clone();
        MockBot {
            commands: commands_copy,
        }
    }
}

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
