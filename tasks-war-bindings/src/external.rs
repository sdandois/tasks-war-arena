use std::ffi::c_char;

#[cfg(not(test))]
#[link(wasm_import_module = "")]
extern "C" {
    pub fn move_task(dir: i32) -> i32;
    pub fn look(delta_x: i32, delta_y: i32) -> i32;
    pub fn split() -> i32;
    pub fn debug(debug_str: *const c_char) -> i32;
    pub fn get_task_weight() -> i32;
}

#[cfg(test)]
pub unsafe fn move_task( dir: i32) -> i32 {
    return dir;
}

#[cfg(test)]
pub unsafe fn look(delta_x: i32, delta_y: i32) -> i32 {
    return delta_x * 10000 + delta_y;
}

#[cfg(test)]
pub unsafe fn split() -> i32 {
    return 0;
}

#[cfg(test)]
pub unsafe fn debug(_debug_str: *const c_char) -> i32 {
    return 0;
}

#[cfg(test)]
pub unsafe fn get_task_weight() -> i32 {
    64
}
