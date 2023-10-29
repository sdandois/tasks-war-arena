use std::fmt;

use rand::Rng;
use rand::SeedableRng;

mod board;
mod commons;
pub mod fruits;
pub mod tasks;
#[cfg(test)]
mod tests;

pub use board::BoardSize;
use board::*;
pub use commons::*;
use fruits::*;
pub use tasks::*;

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LookResult {
    Null,     // 0
    None,     // 1
    Player,   // 2
    Opponent, // 3
    Food,     // 4
}

#[derive(Clone, Debug)]
pub struct Game {
    tasks: Vec<Vec<Task>>,
    board_size: BoardSize,
    board: Board,
    player_points: Vec<usize>,
    config: GameConfig,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GameConfig {
    pub board_size: BoardSize,
    pub seed: u64,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            board_size: Default::default(),
            seed: 32,
        }
    }
}

pub mod error_messages {
    pub const NOT_ENOUGH_WEIGHT: &'static str = "Not enough weight";
}

impl Game {
    pub fn new() -> Game {
        Game::with_full_customization(None, None, None)
    }

    #[cfg(test)]
    fn with_tasks(tasks: Vec<Task>) -> Game {
        Game::with_full_customization(Some(tasks), None, None)
    }

    pub fn from_config(config: GameConfig) -> Game {
        let mut rng = rand::rngs::SmallRng::seed_from_u64(config.seed);

        let board_size = config.board_size;
        let x_0 = rng.gen_range(0..(board_size.0 / 2));
        let y_0 = rng.gen_range(0..(board_size.1));

        let x_1 = rng.gen_range((board_size.0 / 2)..board_size.0);
        let y_1 = rng.gen_range(0..(board_size.1));

        let player0_task = Task::new(0, (x_0, y_0));
        let player1_task = Task::new(1, (x_1, y_1));

        let tasks = vec![player0_task, player1_task];

        let amount_of_squares = board_size.0 * board_size.1;
        let fruits_amount = rng.gen_range(amount_of_squares / 10..amount_of_squares / 5);

        let mut fruits = Vec::new();

        for _ in 0..fruits_amount {
            let fruit_x = rng.gen_range(0..(board_size.0));
            let fruit_y = rng.gen_range(0..(board_size.1));

            let fruit_random = rng.gen_range(0f64..1f64);

            let fruit = if fruit_random < 0.6 {
                Fruit::Grape
            } else if fruit_random < 0.9 {
                Fruit::Grape
            } else {
                Fruit::Strawberry
            };

            fruits.push(FruitPos {
                fruit,
                pos: (fruit_x, fruit_y),
            });
        }

        let mut g = Game::with_full_customization(Some(tasks), Some(fruits), Some(board_size));
        g.config = config;

        g
    }

    fn with_full_customization(
        tasks: Option<Vec<Task>>,
        fruits: Option<Vec<FruitPos>>,
        board_size: Option<BoardSize>,
    ) -> Game {
        let final_tasks = match tasks {
            Some(tts) => {
                let (player0_tasks, player1_tasks): (Vec<_>, Vec<_>) =
                    tts.into_iter().partition(|t| t.player == 0);

                vec![player0_tasks, player1_tasks]
            }
            None => {
                let player0_task = Task::new(0, (10, 10));
                let player1_task = Task::new(1, (10, 10));

                vec![vec![player0_task], vec![player1_task]]
            }
        };

        let final_fruits = match fruits {
            Some(ffs) => ffs,
            None => {
                vec![
                    FruitPos {
                        fruit: Fruit::Banana,
                        pos: (15, 15),
                    },
                    FruitPos {
                        fruit: Fruit::Grape,
                        pos: (5, 10),
                    },
                ]
            }
        };

        let final_board_size = match board_size {
            Some(bs) => bs,
            None => BoardSize(50, 50),
        };

        let mut board: Board = Board::new(final_board_size);

        for i in 0..final_tasks.len() {
            for j in 0..final_tasks[i].len() {
                let t: &Task = &final_tasks[i][j];

                board.add_task(t.pos, TaskId(i, j))
            }
        }

        for f in &final_fruits {
            *board.get_content_mut(f.pos) = BoardContent::Food(f.fruit);
        }

        Game {
            tasks: final_tasks,
            board_size: final_board_size,
            board: board,
            player_points: vec![0, 0],
            config: Default::default(),
        }
    }

    pub fn get_tasks(&self, player: Player) -> &[Task] {
        self.tasks[player].as_slice()
    }

    pub fn get_config(&self) -> &GameConfig {
        &self.config
    }

    fn calculate_new_pos(
        &self,
        task_id: TaskId,
        mut delta: usize,
        dir: Direction,
    ) -> (Position, Position) {
        let task: &Task = self.get_task(task_id);

        if delta > task.move_distance() {
            delta = task.move_distance()
        }

        let (old_x, old_y) = task.pos;

        let new_pos = match dir {
            Direction::Down => (
                (self.board_size.0 + old_x + delta) % self.board_size.0,
                old_y,
            ),
            Direction::Up => (
                (self.board_size.0 + old_x - delta) % self.board_size.0,
                old_y,
            ),
            Direction::Left => (
                old_x,
                (self.board_size.1 + old_y - delta) % self.board_size.1,
            ),
            Direction::Right => (
                old_x,
                (self.board_size.1 + old_y + delta) % self.board_size.1,
            ),
        };

        (task.pos, new_pos)
    }

    pub fn move_task(&mut self, task_id: TaskId, delta: usize, dir: Direction) -> Position {
        debug_assert!(!self.get_task(task_id).is_dead);

        let (old_pos, new_pos) = self.calculate_new_pos(task_id, delta, dir);

        let mut old_pos_content = std::mem::take(self.board.get_content_mut(old_pos));
        let mut new_pos_content = std::mem::take(self.board.get_content_mut(new_pos));

        if let BoardContent::Food(f) = new_pos_content {
            self.player_points[task_id.0] += f.points();
        }

        if let BoardContent::Tasks(new_pos_tasks_ids) = &mut new_pos_content {
            let same_team = {
                let task = self.get_task(task_id);
                let other_task = self.get_task(*new_pos_tasks_ids.first().unwrap());

                task.player == other_task.player
            };

            if same_team {
                self.get_task_mut(task_id).pos = new_pos;

                new_pos_tasks_ids.push(task_id);
                *self.board.get_content_mut(new_pos) = new_pos_content;

                return new_pos;
            }

            let task = self.get_task(task_id);

            let other_weights: usize = new_pos_tasks_ids
                .iter()
                .map(|id| self.get_task(*id).weight)
                .sum();

            if task.weight > other_weights {
                // (task_id, new_pos_tasks_ids)
                for tid in new_pos_tasks_ids {
                    self.get_task_mut(*tid).is_dead = true;
                }
                self.get_task_mut(task_id).pos = new_pos;
                *self.board.get_content_mut(new_pos) = BoardContent::Tasks(vec![task_id]);
            } else {
                self.get_task_mut(task_id).is_dead = true;
                *self.board.get_content_mut(new_pos) = new_pos_content;
            }
        } else {
            if let BoardContent::Tasks(tts) = &mut old_pos_content {
                match tts.len() {
                    1 => {}
                    _ => {
                        let index = tts
                            .iter()
                            .position(|task_id_wanted| *task_id_wanted == task_id)
                            .unwrap();
                        tts.swap_remove(index);
                        *self.board.get_content_mut(old_pos) = old_pos_content;
                    }
                }
            }
            *self.board.get_content_mut(new_pos) = BoardContent::Tasks(vec![task_id]);
            self.get_task_mut(task_id).pos = new_pos;
        }

        return new_pos;
    }

    pub fn look(&self, task_id: TaskId, delta_x: isize, delta_y: isize) -> LookResult {
        debug_assert!(!self.get_task(task_id).is_dead);

        let task = self.get_task(task_id);
        if (delta_x.abs() + delta_y.abs()) as usize > task.look_distance() {
            return LookResult::Null;
        };

        let (old_x, old_y) = task.pos;
        let new_pos = (
            (((self.board_size.0 + old_x) as isize + delta_x) as usize) % self.board_size.0,
            (((self.board_size.1 + old_y) as isize + delta_y) as usize) % self.board_size.1,
        );

        if let BoardContent::Food(_) = self.board.get_content(new_pos) {
            return LookResult::Food;
        } else if let BoardContent::Tasks(t) = self.board.get_content(new_pos) {
            return match task.player == self.get_task(t.first().unwrap().clone()).player {
                false => LookResult::Opponent,
                true => LookResult::Player,
            };
        }

        LookResult::None
    }

    pub fn get_task(&self, task_id: TaskId) -> &Task {
        &self.tasks[task_id.0][task_id.1]
    }

    fn get_task_mut<'a>(&'a mut self, task_id: TaskId) -> &'a mut Task {
        &mut self.tasks[task_id.0][task_id.1]
    }

    pub fn points(&self, player: Player) -> usize {
        self.player_points[player]
    }

    pub fn split(&mut self, task_id: TaskId) -> Result<TaskId, GameError> {
        debug_assert!(!self.get_task(task_id).is_dead);

        if self.get_task(task_id).weight < 2 {
            return Err(error_messages::NOT_ENOUGH_WEIGHT);
        }

        let new_task = {
            let task = self.get_task_mut(task_id);

            task.split()
        };

        let player = new_task.player;
        let new_task_id = TaskId(player, self.tasks[player].len());

        self.board.add_task(new_task.pos, new_task_id);

        self.tasks[player].push(new_task);

        Ok(new_task_id)
    }

    pub fn get_all_task_ids(&self) -> Vec<TaskId> {
        let mut v = Vec::new();

        for i in 0..2 {
            for j in 0..self.tasks[i].len() {
                v.push(TaskId(i, j));
            }
        }

        v
    }

    pub fn is_finished(&self) -> bool {
        self.tasks
            .iter()
            .any(|t| t.iter().filter(|tt| !tt.is_dead).next().is_none())
    }

    pub fn kill(&mut self, tid: TaskId) {
        let pos = {
            let task = self.get_task_mut(tid);
            task.is_dead = true;

            task.pos
        };

        self.board.remove_task(pos, tid);
    }

    fn display_header(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Player 0 points: {}\n", self.player_points[0])?;
        write!(f, "Player 1 points: {}\n\n", self.player_points[1])?;

        for player in 0..2 {
            write!(f, "Player {} tasks: ", player)?;

            for t in self.get_tasks(player) {
                if t.is_dead {
                    write!(f, "D")?;
                } else {
                    write!(f, "A")?;
                }
            }
            write!(f, "\n")?;
        }

        Ok(())
    }
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display_header(f)?;
        self.board.fmt(f)?;

        Ok(())
    }
}

impl TryFrom<i32> for Direction {
    type Error = String;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Direction::Right),
            2 => Ok(Direction::Down),
            3 => Ok(Direction::Left),
            4 => Ok(Direction::Up),
            _ => Err(format!("Invalid direction value: {}", value)),
        }
    }
}
impl From<LookResult> for i32 {
    fn from(value: LookResult) -> Self {
        match value {
            LookResult::Null => 0,
            LookResult::None => 1,
            LookResult::Player => 2,
            LookResult::Opponent => 3,
            LookResult::Food => 4,
        }
    }
}
