use tasks_war_bindings::*;

fn main() {
    for _ in 0..10 {
        move_task(Direction::Up);
    }
}
