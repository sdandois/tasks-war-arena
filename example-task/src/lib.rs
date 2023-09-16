use std::vec;


#[link(wasm_import_module = "")]
extern "C" {
    // imports the name `foo` from `the-wasm-import-module`
    // fn hello_func();

    //
    // fn println(s: &str);

}

#[no_mangle]
pub extern "C" fn run() {
    // unsafe {

    //     hello_func();
    // }
}


#[no_mangle]
pub extern "C" fn create_vector() -> *const u8{
    
    let v : Vec<u8> = vec![1,2,3,4, 80, 90, 110];

    let slice = v.into_boxed_slice();

    let ptr = slice.as_ptr();

    std::mem::forget(slice);

    return ptr;
}