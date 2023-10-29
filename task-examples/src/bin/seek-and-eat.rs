use tasks_war_bindings::*;

fn seek_food() -> (i32, i32) {
    let mut distance = 1;
    let mut delta_x: i32 = 1;
    let mut delta_y: i32 = 0;

    let mut look_result = look(delta_x, delta_y);

    while look_result != LookResult::Food {
        delta_x -= 1;
        delta_y += 1;

        if delta_x < -distance {
            distance += 1;
            delta_x = distance;
            delta_y = 0;
        }

        look_result = look(delta_x, delta_y);
    }

    (delta_x, delta_y)
}

fn long_move(target: (i32, i32)) {
    let mut delta_x = target.0;
    let mut delta_y = target.1;

    while delta_x != 0 || delta_y != 0 {
        if delta_x > 0 {
            move_task(1, Direction::Right);
            delta_x -= 1;
        } else if delta_x < 0 {
            move_task(1, Direction::Left);
            delta_x += 1;
        } else if delta_y > 0 {
            move_task(1, Direction::Down);
            delta_y -= 1;
        } else if delta_y < 0 {
            move_task(1, Direction::Up);
            delta_y += 1;
        }
    }
}

fn main() {
    loop {
        let target = seek_food();

        long_move(target);
    }
}
