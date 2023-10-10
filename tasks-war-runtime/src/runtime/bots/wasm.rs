use wasmtime::*;
use wasmtime_wasi::{sync::WasiCtxBuilder, WasiCtx};

use std::sync::Mutex;

use tokio::sync::mpsc;

struct WasmBot {
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
struct WasmBotFactory {
    engine: Engine,
}

struct Message;

use crate::game::Task;

use super::*;

impl WasmBotFactory {
    pub fn new() -> Result<WasmBotFactory> {
        let mut config = Config::default();

        config.async_support(true);

        let engine = Engine::new(&config)?;

        Ok(WasmBotFactory { engine })
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

                        s.tx_out.send(Command::Look(0, 0)).await?;
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

                        s.tx_out.send(Command::Move(1, Direction::Left)).await?;
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
        let module = Module::from_file(&self.engine, "wasm_modules/example-task.wasm").unwrap();
        WasmBot::spawn(self.engine.clone(), module, task_id)
            .await
            .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate::{game::TaskId, runtime::bots::BotFactory};

    use super::WasmBotFactory;

    #[test]
    fn wasm_bot_game() {
        let factory = WasmBotFactory::new().unwrap();

        let runner = crate::runtime::GameRunner::new(factory);

        let _result = runner.run_some_rounds(5);

    }

    #[test]
    fn full_game_finishes_with_fuel_error() {
        let factory = WasmBotFactory::new().unwrap();

        let runner = crate::runtime::GameRunner::new(factory);

        let _result = runner.run_game();

    }

}
