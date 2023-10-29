use serde::{Deserialize, Serialize};

use crate::game::*;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum Command {
    Move(usize, Direction),
    Look(isize, isize),
    Split,
    Pass,
}

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
            Command::Move(_, _) => 1024,
            Command::Look(_, _) => 128,
            Command::Split => 1,
            Command::Pass => 0,
        }
    }
}
