use wasmtime::*;
use wasmtime_wasi::{sync::WasiCtxBuilder, WasiCtx};

use std::sync::Mutex;

use tokio::sync::mpsc;

pub struct WasmBot {
    tx_in: Option<mpsc::Sender<Message>>,
    rx_out: mpsc::Receiver<Command>,
    handle: Option<tokio::task::JoinHandle<()>>,
}

struct ModuleState {
    wasi: WasiCtx,
    rx_in: mpsc::Receiver<Message>,
    tx_out: mpsc::Sender<Command>,
}

#[derive(Clone)]
pub struct WasmBotFactory {
    engine: Engine,
    module_path: &'static str,
}

struct Message;

use crate::game::Task;

use super::*;

impl WasmBotFactory {
    pub fn new(module_path: &'static str) -> Result<WasmBotFactory> {
        let mut config = Config::default();

        config.async_support(true);

        let engine = Engine::new(&config)?;

        Ok(WasmBotFactory {
            engine,
            module_path,
        })
    }
}

struct WasmRunner {
    engine: Engine,
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
        tx_out: mpsc::Sender<Command>,
        rx_in: mpsc::Receiver<Message>,
    ) -> Self {
        let linker: Linker<ModuleState> = Linker::new(&engine);

        let wasi = Self::create_wasi_ctx(task_id);

        let store = Store::new(
            &engine,
            ModuleState {
                wasi,
                rx_in,
                tx_out,
            },
        );

        WasmRunner {
            engine,
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
                        let s = caller.data_mut();

                        let delta_x = _params[0].unwrap_i32();
                        let delta_y = _params[1].unwrap_i32();

                        s.tx_out.send(Command::Look(delta_x as isize, delta_y as isize)).await?;

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
                        let s = caller.data_mut();

                        let delta = _params[0].unwrap_i32();
                        let dir : Direction =  _params[1].unwrap_i32().try_into().unwrap();


                        s.tx_out.send(Command::Move(delta as usize, dir)).await?;
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

        let out = self
            .linker
            .func_new_async("", "split", split_type, |mut caller, _params, results| {
                Box::new(async move {
                    let s = caller.data_mut();

                    s.tx_out.send(Command::Split).await?;
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

        self.link_module().await;
    }

    async fn run_default(&mut self) -> Result<()> {
        println!("{:?} Running run function...", self.task_id);

        self.linker
            .get_default(&mut (self.store), "")
            .unwrap()
            .typed::<(), ()>(&(self.store))
            .unwrap()
            .call_async(&mut (self.store), ())
            .await // Este await es problemático.
    }

    async fn run_run_fn(&mut self) {
        let _ = self
            .linker
            .get(&mut self.store, "", "run")
            .unwrap()
            .into_func()
            .unwrap()
            .typed::<(), ()>(&self.store)
            .unwrap()
            .call_async(&mut self.store, ())
            .await
            .unwrap();
    }
}

impl WasmBot {
    async fn spawn(engine: Engine, module: Module, task_id: TaskId) -> Result<WasmBot> {
        let (tx_in, rx_in) = tokio::sync::mpsc::channel::<Message>(2);
        let (tx_out, rx_out) = tokio::sync::mpsc::channel::<Command>(2);

        let handle = tokio::spawn(async move {
            let mut runner = WasmRunner::new(engine, module, task_id, tx_out, rx_in);

            runner.prepare().await;
            let res = runner.run_default().await;

            if res.is_err() {
                println!("{:?} Wasm default function finished with error.", task_id);
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
    async fn poll(&mut self) -> Option<Command> {
        self.rx_out.recv().await
    }

    async fn update(&mut self, result: Option<LookResult>) {
        if let Some(tx) = &self.tx_in {
            tx.send(Message).await.unwrap();
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
        let module = Module::from_file(&self.engine, self.module_path).unwrap();
        WasmBot::spawn(self.engine.clone(), module, task_id)
            .await
            .unwrap()
    }
}

#[cfg(test)]
mod tests;
