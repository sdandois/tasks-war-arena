use std::{
    fs::{DirBuilder, File},
    io::Write,
    path::{Path, PathBuf},
};

use wasmtime::*;
use wasmtime_wasi::{sync::WasiCtxBuilder, WasiCtx};

use tokio::sync::mpsc;

use super::*;

pub struct WasmBot {
    tx_in: Option<mpsc::Sender<Option<LookResult>>>,
    rx_out: mpsc::Receiver<(Command, usize)>,
    handle: Option<tokio::task::JoinHandle<()>>,
}

struct ModuleState {
    wasi: WasiCtx,
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
    pub fn same_module(module_path_player0: &'static str) -> Result<WasmBotFactory> {
        Self::new(module_path_player0, module_path_player0)
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
        creator: impl Fn() -> Result<WasmBotFactory>
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
        module_path_player0: &'static str,
        module_path_player1: &'static str,
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
    linker: Linker<ModuleState>,
    store: Store<ModuleState>,
}

impl WasmRunner {
    fn new(
        engine: Engine,
        module: Module,
        task_id: TaskId,
        tx_out: mpsc::Sender<(Command, usize)>,
        rx_in: mpsc::Receiver<Option<LookResult>>,
    ) -> Self {
        let linker: Linker<ModuleState> = Linker::new(&engine);

        let wasi = Self::create_wasi_ctx(task_id);

        let store = Store::new(
            &engine,
            ModuleState {
                wasi,
                rx_in,
                tx_out,
                previous_fuel: 0,
            },
        );

        WasmRunner {
            linker,
            store,
            task_id,
            module,
        }
    }

    fn create_wasi_ctx(task_id: TaskId) -> WasiCtx {
        WasiCtxBuilder::new()
            .inherit_stdio()
            .env("PLAYER", format!("{}", task_id.0).as_str())
            .unwrap()
            .env("SEQUENCE", format!("{}", task_id.1).as_str())
            .unwrap()
            .build()
    }

    fn link_wasi(&mut self) {
        wasmtime_wasi::add_to_linker(&mut self.linker, |state: &mut ModuleState| &mut state.wasi)
            .unwrap();
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
                move |mut caller, _params, results| {
                    Box::new(async move {
                        let delta_x = _params[0].unwrap_i32();
                        let delta_y = _params[1].unwrap_i32();

                        let consumed_fuel =
                            caller.fuel_consumed().unwrap() - caller.data().previous_fuel;
                        caller.data_mut().previous_fuel = caller.fuel_consumed().unwrap();

                        let s = caller.data_mut();
                        s.tx_out
                            .send((
                                Command::Look(delta_x as isize, delta_y as isize),
                                consumed_fuel as usize,
                            ))
                            .await?;

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

    /// Move directions:
    /// Right = 1,
    /// Down = 2,
    /// Left = 3,
    /// Up = 4.
    fn link_move_task_fn(&mut self) {
        let move_task_type = wasmtime::FuncType::new(
            [wasmtime::ValType::I32, wasmtime::ValType::I32],
            Some(wasmtime::ValType::I32),
        );

        self.linker
            .func_new_async(
                "",
                "move_task",
                move_task_type,
                |mut caller, _params, results| {
                    Box::new(async move {
                        let delta = _params[0].unwrap_i32();
                        let dir: Direction = _params[1].unwrap_i32().try_into().unwrap();

                        let consumed_fuel =
                            caller.fuel_consumed().unwrap() - caller.data().previous_fuel;
                        caller.data_mut().previous_fuel = caller.fuel_consumed().unwrap();

                        let s = caller.data_mut();

                        s.tx_out
                            .send((Command::Move(delta as usize, dir), consumed_fuel as usize))
                            .await?;
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
                    let consumed_fuel =
                        caller.fuel_consumed().unwrap() - caller.data().previous_fuel;
                    caller.data_mut().previous_fuel = caller.fuel_consumed().unwrap();

                    let s = caller.data_mut();

                    s.tx_out
                        .send((Command::Split, consumed_fuel as usize))
                        .await?;
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

    async fn link_module(&mut self) {
        println!("Linking our module...");
        self.linker
            .module_async(&mut self.store, "", &self.module)
            .await
            .unwrap();
    }

    async fn prepare(&mut self) {
        self.link_wasi();
        self.link_look_fn();
        self.link_move_task_fn();
        self.link_split_fn();

        self.store.add_fuel(u64::MAX).unwrap();
        self.link_module().await;
    }

    #[allow(dead_code)]
    async fn run_default(&mut self) -> Result<()> {
        println!("{:?} Running default function...", self.task_id);

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
    async fn spawn(engine: Engine, module: Module, task_id: TaskId) -> Result<WasmBot> {
        let (tx_in, rx_in) = tokio::sync::mpsc::channel::<Option<LookResult>>(2);
        let (tx_out, rx_out) = tokio::sync::mpsc::channel::<(Command, usize)>(2);

        let handle = tokio::spawn(async move {
            let mut runner = WasmRunner::new(engine, module, task_id, tx_out, rx_in);

            runner.prepare().await;
            let res: Result<_> = runner.run_main_fn().await;

            if res.is_err() {
                let err = res.unwrap_err();
                println!(
                    "{:?} Wasm default function finished with error: {:?}",
                    task_id, err
                );
            } else {
                println!("{:?} Wasm default function finished ok.", task_id);
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

    async fn create_bot(&self, task_id: TaskId) -> Self::B {
        let module = match task_id {
            TaskId(0, _) => self.module_player0.clone(),
            TaskId(1, _) => self.module_player1.clone(),
            _ => panic!("invalid player"),
        };
        WasmBot::spawn(self.engine.clone(), module, task_id)
            .await
            .unwrap()
    }
}

#[cfg(test)]
mod tests;
