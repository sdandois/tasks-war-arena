#[cfg(not(test))]
#[link(wasm_import_module = "")]
extern "C" {
    pub fn move_task(delta: i32, dir: i32) -> i32;
    pub fn look(delta_x: i32, delta_y: i32) -> i32;
    pub fn split() -> i32;
}

#[cfg(test)]
pub unsafe fn move_task(delta: i32, dir: i32) -> i32 {
    return delta * 10000 + dir;
}

#[cfg(test)]
pub unsafe fn look(delta_x: i32, delta_y: i32) -> i32 {
    return delta_x * 10000 + delta_y;
}

#[cfg(test)]
pub unsafe fn split() -> i32 {
    return 0;
}
