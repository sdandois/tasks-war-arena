use serde::{Deserialize, Serialize};

use crate::game::*;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum Command {
    Move(Direction),
    Look(isize, isize),
    Split,
    Pass,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum CommandResponse {
    None,
    Look(LookResult),
    NewTask(TaskId),
}

impl Command {
    pub fn is_look(&self) -> bool {
        match self {
            Command::Look(_, _) => true,
            _ => false,
        }
    }

    pub fn extra_consumed_fuel(&self) -> u64 {
        match self {
            Command::Move(_) => 2048,
            Command::Look(_, _) => 256,
            Command::Split => 32,
            Command::Pass => 0,
        }
    }
}
