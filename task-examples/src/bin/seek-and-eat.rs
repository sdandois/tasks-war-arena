#[link(wasm_import_module = "")]
extern "C" {

    fn move_task(delta: u32, dir: u32) -> u32;
    fn look(delta_x: u32, delta_y: u32) -> u32;
    fn split() -> u32;

}

fn seek_food() -> (i32, i32) {
    let mut distance = 1;
    let mut delta_x: i32 = 1;
    let mut delta_y: i32 = 0;

    let mut look_result = unsafe { look(delta_x as u32, delta_y as u32) };

    while look_result != 4 {
        delta_x -= 1;
        delta_y += 1;

        if delta_x < 0 {
            distance += 1;
            delta_x = distance;
            delta_y = 0;
        }

        look_result = unsafe { look(delta_x as u32, delta_y as u32) }; 
    }

    (delta_x, delta_y) 
}

fn long_move(target: (i32, i32)) {
    let mut delta_x = target.0;
    let mut delta_y = target.1;

    while delta_x != 0 || delta_y != 0 {
        if delta_x > 0 {
            unsafe {
                move_task(1, 1);
            }
            delta_x -= 1;
        } else if delta_x < 0 {
            unsafe {
                move_task(1, 3);
            }
            delta_x += 1;
        } else if delta_y > 0 {
            unsafe {
                move_task(1, 2);
            }
            delta_y -= 1;
        } else if delta_y < 0 {
            unsafe {
                move_task(1, 4);
            }
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
