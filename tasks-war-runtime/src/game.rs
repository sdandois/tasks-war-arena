use std::fmt;
use std::vec;

mod commons;
mod tasks;
mod fruits;
#[cfg(test)]
mod tests;

use commons::*;
use tasks::*;
use fruits::*;

enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Copy)]
struct BoardSize(usize, usize);

#[derive(Debug, PartialEq, Eq, Clone)]
enum LookResult {
    Null,
    None,
    Player,
    Opponent,
    Food,
}

pub struct Game {
    tasks: Vec<Vec<Task>>,
    board_size: BoardSize,
    board: Board,
    player_points: Vec<usize>,
}

#[derive(PartialEq, Eq, Debug, Default)]
enum BoardContent {
    #[default]
    None,
    Tasks(Vec<TaskId>),
    Food(Fruit),
}

type Board = Vec<Vec<BoardContent>>;

impl Game {
    pub fn new() -> Game {
        Game::with_full_customization(None, None, None)
    }

    fn with_tasks(tasks: Vec<Task>) -> Game {
        Game::with_full_customization(Some(tasks), None, None)
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

        let mut board: Board = Vec::new();

        for i in 0..final_board_size.0 {
            let mut v = Vec::new();

            for j in 0..final_board_size.1 {
                v.push(BoardContent::None)
            }
            board.push(v)
        }

        for i in 0..final_tasks.len() {
            for j in 0..final_tasks[i].len() {
                let t = &final_tasks[i][j];
                board[t.pos.0][t.pos.1] = match std::mem::take(&mut board[t.pos.0][t.pos.1]) {
                    BoardContent::None => BoardContent::Tasks(vec![TaskId(i, j)]),
                    BoardContent::Tasks(mut tts) => BoardContent::Tasks({
                        tts.push(TaskId(i, j));

                        tts
                    }),
                    BoardContent::Food(_) => todo!(),
                };
            }
        }

        for f in &final_fruits {
            board[f.pos.0][f.pos.1] = BoardContent::Food(f.fruit);
        }

        Game {
            tasks: final_tasks,
            board_size: final_board_size,
            board: board,
            player_points: vec![0, 0],
        }
    }

    fn get_tasks_mut(&mut self, player: Player) -> &mut [Task] {
        self.tasks[player].as_mut_slice()
    }

    pub fn get_tasks(&self, player: Player) -> &[Task] {
        self.tasks[player].as_slice()
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

    pub fn move_task(&mut self, task_id: TaskId, mut delta: usize, dir: Direction) -> Position {
        let (old_pos, new_pos) = self.calculate_new_pos(task_id, delta, dir);

        let mut old_pos_content = std::mem::take(&mut self.board[old_pos.0][old_pos.1]);
        let mut new_pos_content = std::mem::take(&mut self.board[new_pos.0][new_pos.1]);

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
                self.board[old_pos.0][old_pos.1] = BoardContent::None;
                self.board[new_pos.0][new_pos.1] = new_pos_content;

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
                self.board[new_pos.0][new_pos.1] = BoardContent::Tasks(vec![task_id]);
            } else {
                self.get_task_mut(task_id).is_dead = true;
                self.board[new_pos.0][new_pos.1] = new_pos_content;
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
                        self.board[old_pos.0][old_pos.1] = old_pos_content;
                    }
                }
            }
            self.board[new_pos.0][new_pos.1] = BoardContent::Tasks(vec![task_id]);
            self.get_task_mut(task_id).pos = new_pos;
        }

        return new_pos;
    }

    pub fn look(&self, task_id: TaskId, delta_x: isize, delta_y: isize) -> LookResult {
        let task = self.get_task(task_id);
        if (delta_x.abs() + delta_y.abs()) as usize > task.look_distance() {
            return LookResult::Null;
        };

        let (old_x, old_y) = task.pos;
        let new_pos = (
            (((self.board_size.0 + old_x) as isize + delta_x) as usize) % self.board_size.0,
            (((self.board_size.1 + old_y) as isize + delta_y) as usize) % self.board_size.1,
        );

        if let BoardContent::Food(_) = self.board[new_pos.0][new_pos.1] {
            return LookResult::Food;
        } else if let BoardContent::Tasks(t) = &self.board[new_pos.0][new_pos.1] {
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
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for i in 0..self.board_size.0 {
            for j in 0..self.board_size.1 {
                match &self.board[i][j] {
                    BoardContent::None => write!(f, "_"),
                    BoardContent::Tasks(t) => write!(f, "{}", t.first().unwrap().0),
                    BoardContent::Food(fruit) => write!(f, "{}", fruit),
                }?;
            }
            write!(f, "\n")?;
        }

        Ok(())
    }
}

