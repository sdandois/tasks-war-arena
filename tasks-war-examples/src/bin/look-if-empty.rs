#[link(wasm_import_module = "")]
extern "C" {

    fn move_task(delta: u32, dir: u32) -> u32;
    fn look(delta_x: u32, delta_y: u32) -> u32;
    fn split() -> u32;

}


// Null,     // 0
// None,     // 1
// Player,   // 2
// Opponent, // 3
// Food,     // 4

/// Move directions:
/// Right = 1,
/// Down = 2,
/// Left = 3,
/// Up = 4.


fn main() {
    println!("main function");
    // for (key, value) in std::env::vars() {
    //     println!("{key}: {value}");
    // }

    unsafe {
        for i in 0..100 {
            println!(
                "{}: Round {}",
                std::env::var("PLAYER").unwrap_or(String::from("(unknown)")),
                i
            );
            
            let look_result = look(1, 1);

            match look_result {
                0 => move_task(2, 1),
                1 => move_task(1, 1),
                2 => move_task(1, 2),
                3 => move_task(1, 3),
                4 => move_task(1, 4),
                _ => panic!("invalid look result")
            };
        }
    }
}
