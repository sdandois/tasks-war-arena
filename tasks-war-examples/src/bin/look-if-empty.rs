use tasks_war_bindings::*;

fn main() {
    println!("main function");

    for i in 0..100 {
        println!(
            "{}: Round {}",
            std::env::var("PLAYER").unwrap_or(String::from("(unknown)")),
            i
        );

        let look_result = look(1, 1);

        match look_result {
            LookResult::Null => move_task(Direction::Right),
            LookResult::None => move_task(Direction::Right),
            LookResult::Player => move_task(Direction::Down),
            LookResult::Opponent => move_task(Direction::Left),
            LookResult::Food => move_task(Direction::Up),
        };
    }
}
