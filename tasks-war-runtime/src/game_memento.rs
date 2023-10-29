use serde::{Deserialize, Serialize};

use crate::game_with_history::Command;

use super::game::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub task_id: TaskId,
    pub command: Command,
}

#[derive(Serialize, Deserialize)]
pub struct GameMemento {
    pub config: GameConfig,
    pub history: Vec<HistoryEntry>, // Cambiar visibilidad e implementar iterador.
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
