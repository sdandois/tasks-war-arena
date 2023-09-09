
#[link(wasm_import_module = "the-wasm-import-module")]
extern "C" {
    // imports the name `foo` from `the-wasm-import-module`
    fn hello_func();

}

#[no_mangle]
pub extern "C" fn run() {
    unsafe {

        hello_func();
    }
}
