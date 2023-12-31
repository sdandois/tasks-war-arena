use std::{
    ffi::CString,
    fs::{DirBuilder, File},
    io::Write,
    path::{Path, PathBuf},
};

use tracing::{event, Level};

use wasmtime::*;

use tokio::sync::mpsc;

use super::*;

const MAX_DEBUG_MESSAGE_SIZE: usize = 256;

pub struct WasmBot {
    tx_in: Option<mpsc::Sender<Option<LookResult>>>,
    rx_out: mpsc::Receiver<(Command, usize)>,
    handle: Option<tokio::task::JoinHandle<()>>,
}

struct ModuleState {
    rx_in: mpsc::Receiver<Option<LookResult>>,
    tx_out: mpsc::Sender<(Command, usize)>,
    previous_fuel: u64,
}

#[derive(Clone)]
pub struct WasmBotFactory {
    engine: Engine,
    module_player0: Module,
    module_player1: Module,
}

impl WasmBotFactory {
    pub fn same_module(module_path_player0: impl AsRef<Path>) -> Result<WasmBotFactory> {
        Self::new(module_path_player0.as_ref(), module_path_player0.as_ref())
    }

    pub fn from_cache(cache_prefix: &Path) -> Result<WasmBotFactory> {
        let engine = Self::create_engine()?;

        let mut path_base_0 = PathBuf::from(cache_prefix);
        let mut path_base_1 = PathBuf::from(cache_prefix);

        path_base_0.push("player0");
        path_base_1.push("player1");

        unsafe {
            let module0 = Module::deserialize_file(&engine, path_base_0)?;
            let module1 = Module::deserialize_file(&engine, path_base_1)?;

            Ok(WasmBotFactory {
                engine: engine.clone(),
                module_player0: module0,
                module_player1: module1,
            })
        }
    }

    pub fn fetch_from_cache(
        cache_prefix: impl AsRef<Path>,
        creator: impl Fn() -> Result<WasmBotFactory>,
    ) -> WasmBotFactory {
        WasmBotFactory::from_cache(cache_prefix.as_ref()).unwrap_or_else(|_| {
            let f = creator().unwrap();
            f.dump(cache_prefix.as_ref());
            f
        })
    }

    fn create_engine() -> Result<Engine> {
        let mut config = Config::default();

        config.async_support(true).consume_fuel(true);

        Engine::new(&config)
    }

    pub fn new(
        module_path_player0: impl AsRef<Path>,
        module_path_player1: impl AsRef<Path>,
    ) -> Result<WasmBotFactory> {
        let engine = Self::create_engine()?;
        Ok(WasmBotFactory {
            engine: engine.clone(),
            module_player0: Module::from_file(&engine, module_path_player0)?,
            module_player1: Module::from_file(&engine, module_path_player1)?,
        })
    }

    pub fn dump(&self, directory: &Path) {
        let mut path_base_0 = PathBuf::from(directory);
        let mut path_base_1 = PathBuf::from(directory);

        DirBuilder::new().recursive(true).create(directory).unwrap();

        path_base_0.push("player0");
        path_base_1.push("player1");

        let v = self.module_player0.serialize().unwrap();

        let mut file = File::create(path_base_0).unwrap();

        file.write_all(&v).unwrap();

        let v = self.module_player1.serialize().unwrap();

        let mut file = File::create(path_base_1).unwrap();

        file.write_all(&v).unwrap();
    }
}

struct WasmRunner {
    module: Module,
    task_id: TaskId,
    task_weight: i32,
    linker: Linker<ModuleState>,
    store: Store<ModuleState>,
}

impl WasmRunner {
    fn new(
        engine: Engine,
        module: Module,
        task_id: TaskId,
        task_weight: i32,
        tx_out: mpsc::Sender<(Command, usize)>,
        rx_in: mpsc::Receiver<Option<LookResult>>,
    ) -> Self {
        let linker: Linker<ModuleState> = Linker::new(&engine);

        let store = Store::new(
            &engine,
            ModuleState {
                rx_in,
                tx_out,
                previous_fuel: 0,
            },
        );

        WasmRunner {
            linker,
            store,
            task_id,
            task_weight,
            module,
        }
    }

    fn link_look_fn(&mut self) {
        let look_type = wasmtime::FuncType::new(
            [wasmtime::ValType::I32, wasmtime::ValType::I32],
            Some(wasmtime::ValType::I32),
        );
        self.linker
            .func_new_async(
                "",
                "look",
                look_type,
                move |mut caller: Caller<'_, ModuleState>, _params, results| {
                    Box::new(async move {
                        let delta_x = _params[0].unwrap_i32();
                        let delta_y = _params[1].unwrap_i32();

                        let command = Command::Look(delta_x as isize, delta_y as isize);
                        let _ = caller.consume_fuel(command.extra_consumed_fuel());

                        let consumed_fuel =
                            caller.fuel_consumed().unwrap() - caller.data().previous_fuel;
                        caller.data_mut().previous_fuel = caller.fuel_consumed().unwrap();

                        let s = caller.data_mut();
                        s.tx_out.send((command, consumed_fuel as usize)).await?;

                        let look_result = s
                            .rx_in
                            .recv()
                            .await
                            .ok_or(wasmtime::Error::msg("nothing else to receive"))?
                            .ok_or(wasmtime::Error::msg("expected a look result"))?;

                        results[0] = Val::I32(look_result.into());
                        Ok(())
                    })
                },
            )
            .unwrap();
    }

    fn link_move_task_fn(&mut self) {
        let move_task_type =
            wasmtime::FuncType::new([wasmtime::ValType::I32], Some(wasmtime::ValType::I32));

        let task_weight = self.task_weight;
        self.linker
            .func_new_async(
                "",
                "move_task",
                move_task_type,
                move |mut caller, _params, results| {
                    Box::new(async move {
                        let dir: Direction = _params[0].unwrap_i32().try_into().unwrap();
                        let command = Command::Move(dir);

                        let fuel_delta = command.extra_consumed_fuel() * task_weight as u64;

                        let _ = caller.consume_fuel(fuel_delta);
                        let consumed_fuel =
                            caller.fuel_consumed().unwrap() - caller.data().previous_fuel;
                        caller.data_mut().previous_fuel = caller.fuel_consumed().unwrap();

                        let s = caller.data_mut();

                        s.tx_out.send((command, consumed_fuel as usize)).await?;
                        let _m = s
                            .rx_in
                            .recv()
                            .await
                            .ok_or(wasmtime::Error::msg("nothing else to receive"))?;

                        results[0] = Val::I32(0 as i32);
                        Ok(())
                    })
                },
            )
            .unwrap();
    }

    fn link_split_fn(&mut self) {
        let split_type = wasmtime::FuncType::new([], Some(wasmtime::ValType::I32));

        self.linker
            .func_new_async("", "split", split_type, |mut caller, _params, results| {
                Box::new(async move {
                    let command = Command::Split;
                    let _ = caller.consume_fuel(command.extra_consumed_fuel());

                    let consumed_fuel =
                        caller.fuel_consumed().unwrap() - caller.data().previous_fuel;
                    caller.data_mut().previous_fuel = caller.fuel_consumed().unwrap();

                    let s = caller.data_mut();

                    s.tx_out.send((command, consumed_fuel as usize)).await?;
                    let _m = s
                        .rx_in
                        .recv()
                        .await
                        .ok_or(wasmtime::Error::msg("nothing else to receive"))?;
                    results[0] = Val::I32(0 as i32);

                    Ok(())
                })
            })
            .unwrap();
    }

    fn link_weight(&mut self) {
        let get_weight_type = wasmtime::FuncType::new([], Some(wasmtime::ValType::I32));

        let weight = self.task_weight;

        self.linker
            .func_new_async(
                "",
                "get_task_weight",
                get_weight_type,
                move |mut _caller, _params, results| {
                    Box::new(async move {
                        results[0] = Val::I32(weight);

                        Ok(())
                    })
                },
            )
            .unwrap();
    }

    fn link_debug_fn(&mut self) {
        let debug_type = wasmtime::FuncType::new([ValType::I32], Some(wasmtime::ValType::I32));

        let task_id = self.task_id;
        self.linker
            .func_new_async(
                "",
                "debug",
                debug_type,
                move |mut caller, _params, results| {
                    Box::new(async move {
                        let pointer = _params[0].unwrap_i32() as usize;

                        let memory = caller.get_export("memory").unwrap().into_memory().unwrap();

                        let mem_data = memory.data(caller.as_context());

                        let find_0 = mem_data[pointer..]
                            .iter()
                            .take(MAX_DEBUG_MESSAGE_SIZE + 1)
                            .position(|v| *v == 0)
                            .unwrap_or(MAX_DEBUG_MESSAGE_SIZE);

                        let mut v: Vec<u8> = mem_data[pointer..(pointer + find_0 + 1)]
                            .iter()
                            .map(|v| v.clone())
                            .collect();

                        *v.last_mut().unwrap() = 0;
                        let c_str = CString::from_vec_with_nul(v)
                            .unwrap()
                            .into_string()
                            .unwrap();

                        event!(Level::INFO, "{:?}: {}", task_id, c_str);

                        results[0] = Val::I32(0 as i32);

                        Ok(())
                    })
                },
            )
            .unwrap();
    }

    async fn link_module(&mut self) {
        event!(Level::INFO, "Linking our module...");
        self.linker
            .module_async(&mut self.store, "", &self.module)
            .await
            .unwrap();
    }

    async fn prepare(&mut self) {
        self.link_look_fn();
        self.link_move_task_fn();
        self.link_split_fn();
        self.link_debug_fn();
        self.link_weight();

        self.store.add_fuel(u64::MAX).unwrap(); // TODO: Add correct amount of fuel.
        self.link_module().await;
    }

    #[allow(dead_code)]
    async fn run_default(&mut self) -> Result<()> {
        event!(
            Level::INFO,
            "{:?} Running default function...",
            self.task_id
        );

        self.linker
            .get_default(&mut (self.store), "")
            .unwrap()
            .typed::<(), ()>(&(self.store))
            .unwrap()
            .call_async(&mut (self.store), ())
            .await
    }

    #[allow(dead_code)]
    async fn run_run_fn(&mut self) -> Result<()> {
        self.linker
            .get(&mut self.store, "", "run")
            .unwrap()
            .into_func()
            .unwrap()
            .typed::<(), ()>(&self.store)
            .unwrap()
            .call_async(&mut self.store, ())
            .await
    }

    async fn run_main_fn(&mut self) -> Result<()> {
        self.linker
            .get(&mut self.store, "", "main")
            .unwrap()
            .into_func()
            .unwrap()
            .typed::<(i32, i32), i32>(&self.store)
            .unwrap()
            .call_async(&mut self.store, (0, 0))
            .await
            .map(|_| ())
    }
}

impl WasmBot {
    async fn spawn(
        engine: Engine,
        module: Module,
        task_id: TaskId,
        task_weight: i32,
    ) -> Result<WasmBot> {
        let (tx_in, rx_in) = tokio::sync::mpsc::channel::<Option<LookResult>>(2);
        let (tx_out, rx_out) = tokio::sync::mpsc::channel::<(Command, usize)>(2);

        let handle = tokio::spawn(async move {
            let mut runner = WasmRunner::new(engine, module, task_id, task_weight, tx_out, rx_in);

            runner.prepare().await;
            let res: Result<_> = runner.run_main_fn().await;

            if res.is_err() {
                let err = res.unwrap_err();
                event!(
                    Level::INFO,
                    "{:?} Wasm default function finished with error: {:?}",
                    task_id,
                    err
                );
            } else {
                event!(
                    Level::INFO,
                    "{:?} Wasm default function finished ok.",
                    task_id
                );
            }
        });

        Ok(WasmBot {
            tx_in: Some(tx_in),
            rx_out,
            handle: Some(handle),
        })
    }
}

#[async_trait]
impl Bot for WasmBot {
    async fn poll(&mut self) -> Option<(Command, usize)> {
        self.rx_out.recv().await
    }

    async fn update(&mut self, result: Option<LookResult>) {
        if let Some(tx) = &self.tx_in {
            tx.send(result).await.unwrap();
        }
    }
    async fn wait(&mut self) {
        self.tx_in.take();
        self.rx_out.close();
        if let Some(h) = self.handle.take() {
            h.await.unwrap();
        }
    }
}

#[async_trait]
impl BotFactory for WasmBotFactory {
    type B = WasmBot;

    async fn create_bot(&self, task_id: TaskId, task_weight: i32) -> Self::B {
        let module = match task_id {
            TaskId(0, _) => self.module_player0.clone(),
            TaskId(1, _) => self.module_player1.clone(),
            _ => panic!("invalid player"),
        };
        WasmBot::spawn(self.engine.clone(), module, task_id, task_weight)
            .await
            .unwrap()
    }
}

#[cfg(test)]
mod tests;
