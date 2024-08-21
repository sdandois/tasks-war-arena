use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::command::{Command, CommandResponse};

use super::game::*;

pub mod avro;

#[cfg(test)]
mod tests;

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct GameMemento {
    pub config: GameConfig,
    pub history: Vec<HistoryEntry>, // Cambiar visibilidad e implementar iterador.
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ActionEntry {
    pub task_id: TaskId,
    pub command: Command,
    pub used_fuel: isize,
    pub command_response: CommandResponse,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum HistoryEntry {
    Action(ActionEntry),
    KillEvent(TaskId, String),
}

impl GameMemento {
    #[cfg(test)]
    pub fn test_memento() -> GameMemento {
        GameMemento {
            config: GameConfig::default(),
            history: vec![
                HistoryEntry::create_action(
                    TaskId(1, 0),
                    Command::Move(Direction::Right),
                    2,
                    CommandResponse::None,
                ),
                HistoryEntry::create_action(
                    TaskId(0, 0),
                    Command::Move(Direction::Left),
                    3,
                    CommandResponse::None,
                ),
                HistoryEntry::create_action(
                    TaskId(0, 0),
                    Command::Move(Direction::Right),
                    1,
                    CommandResponse::None,
                ),
            ],
        }
    }
}

impl HistoryEntry {
    pub fn create_action(
        task_id: TaskId,
        command: Command,
        used_fuel: isize,
        command_response: CommandResponse,
    ) -> HistoryEntry {
        HistoryEntry::Action(ActionEntry {
            task_id,
            command,
            used_fuel,
            command_response,
        })
    }

    pub fn create_kill_event(task_id: TaskId, reason: Option<String>) -> HistoryEntry {
        HistoryEntry::KillEvent(
            task_id,
            reason.unwrap_or_else(|| String::from("out of bad luck")),
        )
    }

    pub fn is_look(&self) -> bool {
        match self {
            HistoryEntry::Action(a) => a.command.is_look(),
            HistoryEntry::KillEvent(_, _) => false,
        }
    }
}

impl Display for ActionEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?}: executed {:?} having used {} fuel in total.",
            self.task_id, self.command, self.used_fuel
        )?;

        if let CommandResponse::NewTask(tid) = self.command_response {
            write!(f, " {:?} has spawned", tid)?;
        };

        if let CommandResponse::Look(result) = self.command_response {
            write!(f, " Has seen {:?}", result)?;
        }

        Ok(())
    }
}

impl Display for HistoryEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HistoryEntry::Action(action) => write!(f, "{}", action),
            HistoryEntry::KillEvent(tid, reason) => write!(f, "{:?}: died {}", tid, reason),
        }
    }
}


