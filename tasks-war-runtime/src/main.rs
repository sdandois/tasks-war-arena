//! Small example of how to instantiate a wasm module that imports one function,
//! showing how you can fill in host functionality for a wasm module.

// You can execute this example with `cargo run --example hello`

use wasmtime::*;

mod game;
mod runtime;
struct MyState {
    name: String,
    count: usize,
}

fn main() -> Result<()> {
    // First the wasm module needs to be compiled. This is done with a global
    // "compilation environment" within an `Engine`. Note that engines can be
    // further configured through `Config` if desired instead of using the
    // default like this is here.
    println!("Compiling module...");
    let engine = Engine::default();
    let module = Module::from_file(&engine, "examples/example_task.wasm")?;

    // After a module is compiled we create a `Store` which will contain
    // instantiated modules and other items like host functions. A Store
    // contains an arbitrary piece of host information, and we use `MyState`
    // here.
    println!("Initializing...");
    let mut store = Store::new(
        &engine,
        MyState {
            name: "hello, world!".to_string(),
            count: 0,
        },
    );

    // Our wasm module we'll be instantiating requires one imported function.
    // the function takes no parameters and returns no results. We create a host
    // implementation of that function here, and the `caller` parameter here is
    // used to get access to our original `MyState` value.
    println!("Creating callback...");
    // let println = Func::wrap(&mut store, |s: &str| {
    //     println!("Calling back...");
    //     println!(s);
    //     caller.data_mut().count += 1;
    // });

    // Once we've got that all set up we can then move to the instantiation
    // phase, pairing together a compiled module as well as a set of imports.
    // Note that this is where the wasm `start` function, if any, would run.
    println!("Instantiating module...");
    let imports = [];
    let instance = Instance::new(&mut store, &module, &imports)?;

    // Next we poke around a bit to extract the `run` function from the module.
    // println!("Extracting export...");
    // let run = instance.get_typed_func::<(), ()>(&mut store, "print")?;

    // And last but not least we can call it!
    // println!("Calling export...");
    // run.call(&mut store, ())?;

    let mem: Memory = instance.get_memory(&mut store, "memory").unwrap();

    let create_vector = instance.get_typed_func::<(), i32>(&mut store, "create_vector")?;

    let vec_result: i32 = create_vector.call(&mut store, ())?;

    println!("vec results {:?}", vec_result);

    let buf: &[u8] = mem.data(&store);

    let inf_limit: usize = vec_result as usize;
    let sup_limit: usize = (vec_result + 4 * 16) as usize;

    let short_slice = &buf[inf_limit..sup_limit];

    println!("vec results {:?}", short_slice);

    let game = game::Game::new();

    println!("{}", game);

    println!("Done.");
    Ok(())
}
