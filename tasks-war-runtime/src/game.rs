use std::fmt;
use std::vec;

#[cfg(test)]
mod tests;

#[derive(Clone)]
struct Task {
    player: Player,
    pos: Position,
    weight: usize,
}

enum Direction {
    Up,
    Down,
    Left,
    Right,
}

type Position = (usize, usize);

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

impl Task {
    fn new(player: Player, pos: Position) -> Task {
        Task {
            player,
            weight: 64,
            pos,
        }
    }

    fn with_weight(player: Player, pos: Position, weight: usize) -> Task {
        Task {
            player,
            weight,
            pos,
        }
    }

    fn look_distance(&self) -> usize {
        self.weight
    }
    fn move_distance(&self) -> usize {
        64 / self.weight
    }
}

type Player = usize;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct TaskId(usize, usize);

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
enum Fruit {
    Grape,
    Banana,
    Strawberry,
}

impl Fruit {
    fn points(&self) -> usize {
        match &self {
            Fruit::Grape => 16,
            Fruit::Banana => 32,
            Fruit::Strawberry => 64,
        }
    }
}

struct FruitPos {
    fruit: Fruit,
    pos: Position,
}

pub struct Game {
    tasks: Vec<Vec<Task>>,
    board_size: BoardSize,
    board: Board,
    player_points: Vec<usize>,
}

#[derive(PartialEq, Eq, Debug)]
enum BoardContent {
    None,
    Task(TaskId),
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
                board[t.pos.0][t.pos.1] = BoardContent::Task(TaskId(i, j));
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

    fn get_tasks(&self, player: Player) -> &[Task] {
        self.tasks[player].as_slice()
    }

    fn move_task(&mut self, task_id: TaskId, mut delta: usize, dir: Direction) -> Position {
        let (old_x, old_y) = {
            let task: &Task = self.get_task(task_id);

            if delta > task.move_distance() {
                delta = task.move_distance()
            }

            task.pos
        };

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

        if let BoardContent::Food(f) = self.board[new_pos.0][new_pos.1] {
            self.player_points[task_id.0] += f.points();
        }

        if let BoardContent::Task(other_task_id) = self.board[new_pos.0][new_pos.1] {

            let winner_id = {
                let task = self.get_task(task_id);
                let other_task = self.get_task(other_task_id);
    
                if task.weight > other_task.weight {
                    task_id
                } else {
                    other_task_id
                }
            };
          
            self.board[old_x][old_y] = BoardContent::None;
            self.board[new_pos.0][new_pos.1] = BoardContent::Task(winner_id);
        } else {
            self.board[old_x][old_y] = BoardContent::None;
            self.board[new_pos.0][new_pos.1] = BoardContent::Task(task_id);
        }

        self.get_task_mut(task_id).pos = new_pos;

        return new_pos;
    }

    fn look(&self, task_id: TaskId, delta_x: isize, delta_y: isize) -> LookResult {
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
        } else if let BoardContent::Task(t) = self.board[new_pos.0][new_pos.1] {
            return match task.player == self.get_task(t).player {
                false => LookResult::Opponent,
                true => LookResult::Player,
            };
        }

        LookResult::None
    }

    fn get_task(&self, task_id: TaskId) -> &Task {
        &self.tasks[task_id.0][task_id.1]
    }

    fn get_task_mut<'a>(&'a mut self, task_id: TaskId) -> &'a mut Task {
        &mut self.tasks[task_id.0][task_id.1]
    }

    fn points(&self, player: Player) -> usize {
        self.player_points[player]
    }
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for i in 0..self.board_size.0 {
            for j in 0..self.board_size.1 {
                match &self.board[i][j] {
                    BoardContent::None => write!(f, "_"),
                    BoardContent::Task(t) => write!(f, "{}", t.0),
                    BoardContent::Food(fruit) => write!(f, "{}", fruit),
                }?;
            }
            write!(f, "\n")?;
        }

        Ok(())
    }
}

impl fmt::Display for Fruit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Fruit::Grape => write!(f, "G"),
            Fruit::Banana => write!(f, "B"),
            Fruit::Strawberry => write!(f, "F"),
        }
    }
}
