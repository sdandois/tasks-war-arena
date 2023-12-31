use tasks_war_bindings::*;
fn main() {
    for _ in 0..1_000_000_000 {
        move_task(Direction::Right);
    }
}
