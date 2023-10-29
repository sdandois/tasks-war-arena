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
}

impl GameMemento {
    #[cfg(test)]
    pub fn test_memento() -> GameMemento {
        GameMemento {
            config: GameConfig::default(),
            history: vec![
                HistoryEntry::new(TaskId(0, 0), Command::Move(1, Direction::Right)),
                HistoryEntry::new(TaskId(1, 0), Command::Move(1, Direction::Right)),
                HistoryEntry::new(TaskId(0, 0), Command::Move(2, Direction::Left)),
            ],
        }
    }
}

impl HistoryEntry {
    pub fn new(task_id: TaskId, command: Command) -> HistoryEntry {
        HistoryEntry { task_id, command }
    }
}

impl Display for HistoryEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} executed {:?}", self.task_id, self.command)
    }
}
