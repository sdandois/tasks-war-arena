use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::command::Command;

use super::game::*;

#[derive(Serialize, Deserialize)]
pub struct GameMemento {
    pub config: GameConfig,
    pub history: Vec<HistoryEntry>, // Cambiar visibilidad e implementar iterador.
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub task_id: TaskId,
    pub command: Command,
    pub used_fuel: isize,
}

impl GameMemento {
    #[cfg(test)]
    pub fn test_memento() -> GameMemento {
        GameMemento {
            config: GameConfig::default(),
            history: vec![
                HistoryEntry::new(TaskId(0, 0), Command::Move(1, Direction::Right), 1),
                HistoryEntry::new(TaskId(1, 0), Command::Move(1, Direction::Right), 2),
                HistoryEntry::new(TaskId(0, 0), Command::Move(2, Direction::Left), 3),
            ],
        }
    }
}

impl HistoryEntry {
    pub fn new(task_id: TaskId, command: Command, used_fuel: isize) -> HistoryEntry {
        HistoryEntry { task_id, command, used_fuel }
    }
}

impl Display for HistoryEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} executed {:?} having used {} fuel in total", self.task_id, self.command, self.used_fuel)
    }
}
