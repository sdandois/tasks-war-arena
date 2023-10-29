#[link(wasm_import_module = "")]
extern "C" {

    fn move_task(delta: u32, dir: u32) -> u32;
    fn look(delta_x: u32, delta_y: u32) -> u32;
    fn split() -> u32;

}

fn main() {
    unsafe {
        for _i in 0..10 {
            look(17, 23);
        }
    }
}
