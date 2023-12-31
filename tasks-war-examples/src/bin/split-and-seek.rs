use std::cmp::min;

use tasks_war_bindings::*;

static mut WEIGHT: i32 = 0;

#[inline]
fn get_weight() -> i32 {
    unsafe { WEIGHT }
}

fn max_leap() -> i32 {
    return 64 / get_weight();
}

fn seek_food() -> (i32, i32) {
    let mut distance = 1;
    let mut delta_x: i32 = 1;
    let mut delta_y: i32 = 0;
    let mut quadrant = 0;

    let mut look_result = look(delta_x, delta_y);

    while look_result != LookResult::Food {
        match quadrant {
            0 => {
                // x+, y+
                delta_x -= 1;
                delta_y += 1;

                if delta_x < 0 {
                    quadrant = 1;
                    delta_x = -1;
                    delta_y = distance - 1;
                }
            }
            1 => {
                // x-, y+
                delta_x -= 1;
                delta_y -= 1;

                if delta_y < 0 {
                    quadrant = 2;
                    delta_x = -(distance - 1);
                    delta_y = -1;
                }
            }
            2 => {
                // x-, y-
                delta_x += 1;
                delta_y -= 1;

                if delta_x > 0 {
                    quadrant = 3;
                    delta_x = distance;
                    delta_y = 0;
                }
            }
            3 => {
                // x+, y-
                delta_x -= 1;
                delta_y -= 1;

                if delta_x < 0 {
                    quadrant = 0;
                    distance += 1;
                    delta_x = distance;
                    delta_y = 0;
                }
            }
            _ => panic!("invalid quadrant"),
        };

        look_result = look(delta_x, delta_y);
    }

    (delta_x, delta_y)
}

fn long_move(target: (i32, i32)) {
    let mut delta_x = target.0;
    let mut delta_y = target.1;

    while delta_x != 0 || delta_y != 0 {
        let leap = min(delta_x.abs(), max_leap());

        if delta_x > 0 {
            move_task(leap as u32, Direction::Down);
            delta_x -= leap;
        } else if delta_x < 0 {
            let leap = min(delta_x.abs(), max_leap());
            move_task(leap as u32, Direction::Up);
            delta_x += leap;
        } else if delta_y > 0 {
            let leap = min(delta_y.abs(), max_leap());
            move_task(leap as u32, Direction::Right);
            delta_y -= leap;
        } else if delta_y < 0 {
            let leap = min(delta_y.abs(), max_leap());
            move_task(leap as u32, Direction::Left);
            delta_y += leap;
        }
    }
}

fn main() {
    unsafe {
        WEIGHT = get_task_weight();
    }
    
    debug(&format!("Current weight is from inside wasm: {}", get_weight()));

    split();

    loop {
        let target = seek_food();

        long_move(target);
    }
}
