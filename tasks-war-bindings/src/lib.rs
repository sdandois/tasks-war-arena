mod external;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LookResult {
    Null,     // 0
    None,     // 1
    Player,   // 2
    Opponent, // 3
    Food,     // 4
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

pub fn look(delta_x: i32, delta_y: i32) -> LookResult {
    unsafe { external::look(delta_x as i32, delta_y as i32).into() }
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
