mod external;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

/// This struct represents the possible outcomes of the
/// `look` command.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LookResult {
    /// Null means 'unknown', you've tried to look to far and couldn't get an answer.
    Null, // 0
    /// The slot is empty.
    None, // 1
    /// There is a task of the same player at the slot.
    Player, // 2
    /// There is a task of the opponent at the slot.
    Opponent, // 3
    /// There is food at the slot.
    Food, // 4
}

impl From<Direction> for i32 {
    fn from(value: Direction) -> Self {
        match value {
            Direction::Right => 1,
            Direction::Down => 2,
            Direction::Left => 3,
            Direction::Up => 4,
        }
    }
}

impl From<i32> for LookResult {
    fn from(value: i32) -> Self {
        match value {
            0 => LookResult::Null,
            1 => LookResult::None,
            2 => LookResult::Player,
            3 => LookResult::Opponent,
            4 => LookResult::Food,
            _ => panic!("invalid i32 for look result"),
        }
    }
}

pub fn move_task(delta: u32, dir: Direction) {
    unsafe {
        external::move_task(delta as i32, dir.into());
    }
}

/// The look command lets you seed what is present in a board tile at distance.
///
/// The board is indexed like matrixes, (1, 1) means one row down, and one column left.xx
pub fn look(vertical_delta: i32, horizontal_delta: i32) -> LookResult {
    unsafe { external::look(vertical_delta as i32, horizontal_delta as i32).into() }
}

pub fn split() {
    unsafe {
        external::split();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn call_move_unsafe() {
        unsafe {
            let result = external::move_task(2, 3);
            assert_eq!(result, 20003);
        }
    }
}
