#[link(wasm_import_module = "")]
extern "C" {

    fn move_task(delta: u32, dir: u32) -> u32;
    fn look(delta_x: u32, delta_y: u32) -> u32;
    fn split() -> u32;

}


fn main() {
    // println!("main function");
    // for (key, value) in std::env::vars() {
    //     println!("{key}: {value}");
    // }

    unsafe {
        for i in 0..10 {
            // println!(
            //     "{}: Round {}",
            //     std::env::var("PLAYER").unwrap_or(String::from("(unknown)")),
            //     i
            // );
            // look(1, 1);
            move_task(17, 2);
            // split();
        }
    }
}
