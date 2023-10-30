use std::fmt;
use std::fmt::Display;

use crate::command::{Command, CommandResponse};
use crate::game::*;
use crate::game_memento::*;

#[derive(Clone)]
pub struct GameWithHistory {
    game: Game,
    history: Vec<HistoryEntry>,
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

    pub fn accept(&mut self, task_id: TaskId, command: &Command, used_fuel: isize) -> CommandResponse {
        self.history.push(HistoryEntry::new(task_id, command.clone(), used_fuel));
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

    pub fn get_config(&self) -> &GameConfig {
        self.game.get_config()
    }
}

impl Display for GameWithHistory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.game)
    }
}
