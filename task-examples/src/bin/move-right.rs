#[link(wasm_import_module = "")]
extern "C" {

    fn move_task(delta: u32, dir: u32) -> u32;
    fn look(delta_x: u32, delta_y: u32) -> u32;
    fn split() -> u32;

}

#[no_mangle]
pub extern "C" fn run() {
    unsafe {
        println!("run function");
        for i in 0..20 {
            println!("Round {}", i);
            look(1, 1);
            move_task(1, 1);
            split();
        }
    }
}

fn main() {
    println!("main function");
    // for (key, value) in std::env::vars() {
    //     println!("{key}: {value}");
    // }

    unsafe {
        for i in 0..10 {
            println!(
                "{}: Round {}",
                std::env::var("PLAYER").unwrap_or(String::from("(unknown)")),
                i
            );
            // look(1, 1);
            // move_task(1, 1);
            move_task(17, 1);

            // split();
        }
    }
}
