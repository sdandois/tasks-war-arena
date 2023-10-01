use super::commons::*;

#[derive(Clone)]
pub struct Task {
    pub player: Player,
    pub pos: Position,
    pub weight: usize,
    pub is_dead: bool,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct TaskId(pub usize, pub usize);

impl Task {
    pub fn new(player: Player, pos: Position) -> Task {
        Task {
            player,
            weight: 64,
            pos,
            is_dead: false,
        }
    }

    pub fn with_weight(player: Player, pos: Position, weight: usize) -> Task {
        Task {
            player,
            weight,
            pos,
            is_dead: false,
        }
    }

    pub fn look_distance(&self) -> usize {
        self.weight
    }
    pub fn move_distance(&self) -> usize {
        64 / self.weight
    }

    pub fn split(&mut self) -> Task {
        self.weight = self.weight / 2;

        Task::with_weight(self.player, self.pos, self.weight)
    }
}
