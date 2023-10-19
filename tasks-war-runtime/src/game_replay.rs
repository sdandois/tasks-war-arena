use std::fmt;
use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::runtime::bots::Command;

use super::game::*;

pub enum CommandResponse {
    None,
    Look(LookResult),
    NewTask(TaskId),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct HistoryEntry {
    task_id: TaskId,
    command: Command,
}

#[derive(Clone)]
pub struct GameWithHistory {
    game: Game,
    history: Vec<HistoryEntry>,
}

#[derive(Serialize, Deserialize)]
pub struct GameMemento {
    pub config: GameConfig,
    history: Vec<HistoryEntry>,
}

impl GameMemento {
    pub fn test_memento() -> GameMemento {
        GameMemento {
            config: GameConfig::default(),
            history: vec![
                HistoryEntry {
                    task_id: TaskId(0, 0),
                    command: Command::Move(1, Direction::Right),
                },
                HistoryEntry {
                    task_id: TaskId(1, 0),
                    command: Command::Move(1, Direction::Right),
                },
                HistoryEntry {
                    task_id: TaskId(0, 0),
                    command: Command::Move(2, Direction::Left),
                },
            ],
        }
    }
}

pub struct GameReplay {
    game: Game,
    history: Vec<HistoryEntry>,
    cursor: usize,
}

impl GameWithHistory {
    pub fn from_config(config: GameConfig) -> GameWithHistory {
        GameWithHistory {
            game: Game::from_config(config.clone()),
            history: Vec::default(),
        }
    }

    pub fn get_task(&self, task_id: TaskId) -> &Task {
        self.game.get_task(task_id)
    }

    pub fn points(&self, player: Player) -> usize {
        self.game.points(player)
    }

    pub fn accept(&mut self, task_id: TaskId, command: &Command) -> CommandResponse {
        self.history.push(HistoryEntry {
            task_id,
            command: command.clone(),
        });
        match command {
            Command::Move(random_delta, random_dir) => {
                self.game.move_task(task_id, *random_delta, *random_dir);

                CommandResponse::None
            }
            Command::Split => {
                let res = self.game.split(task_id);

                match res {
                    Ok(tid) => CommandResponse::NewTask(tid),
                    _ => CommandResponse::None,
                }
            }
            Command::Look(delta_x, delta_y) => {
                let res = self.game.look(task_id, *delta_x, *delta_y);

                CommandResponse::Look(res)
            }
            Command::Pass => CommandResponse::None,
        }
    }

    pub fn get_all_task_ids(&self) -> Vec<TaskId> {
        self.game.get_all_task_ids()
    }

    pub fn is_finished(&self) -> bool {
        self.game.is_finished()
    }

    pub fn kill(&mut self, tid: TaskId) {
        self.game.kill(tid)
    }

    pub fn dump(self) -> GameMemento {
        GameMemento {
            history: self.history,
            config: self.game.get_config().clone(),
        }
    }
}

impl From<GameMemento> for GameReplay {
    fn from(value: GameMemento) -> Self {
        GameReplay {
            game: Game::from_config(value.config.clone()),
            history: value.history,
            cursor: 0,
        }
    }
}

impl Display for GameReplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.game)
    }
}

impl Display for GameWithHistory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.game)
    }
}

impl GameReplay {
    fn advance(&mut self) {
        let HistoryEntry { task_id, command } = self.history[self.cursor].clone();
        self.cursor += 1;

        self.accept(task_id, &command);
    }

    pub fn accept(&mut self, task_id: TaskId, command: &Command) {
        match command {
            Command::Move(random_delta, random_dir) => {
                self.game.move_task(task_id, *random_delta, *random_dir);
            }
            Command::Split => {
                let res = self.game.split(task_id);
            }
            Command::Look(delta_x, delta_y) => {
                let res = self.game.look(task_id, *delta_x, *delta_y);
            }
            Command::Pass => (),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replay_history() {
        let mut game = GameWithHistory::from_config(GameConfig {
            seed: 1,
            board_size: BoardSize(5, 5),
        });

        game.accept(TaskId(0, 0), &Command::Move(1, Direction::Right));
        game.accept(TaskId(1, 0), &Command::Move(1, Direction::Left));

        let memento = game.dump();

        let mut game_replay = GameReplay::from(memento);

        let exp_str = r"Board size: 5 5
Player 0 points: 0
Player 1 points: 0

Player 0 tasks: A
Player 1 tasks: A
_____
0____
_G1__
_____
__G__
";

        assert_eq!(exp_str, game_replay.to_string());

        game_replay.advance();

        let exp_str = r"Board size: 5 5
Player 0 points: 0
Player 1 points: 0

Player 0 tasks: A
Player 1 tasks: A
_____
_0___
_G1__
_____
__G__
";

        assert_eq!(exp_str, game_replay.to_string());

        game_replay.advance();

        let exp_str = r"Board size: 5 5
Player 0 points: 0
Player 1 points: 16

Player 0 tasks: A
Player 1 tasks: A
_____
_0___
_1___
_____
__G__
";
        print!("{game_replay}");
        assert_eq!(exp_str, game_replay.to_string());
    }
}
