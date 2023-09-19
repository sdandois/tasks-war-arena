use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;

use tokio::join;
use tokio::sync;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;

use crate::game::Game;
use crate::game::TaskId;

enum Message {
    None,
}

#[cfg(test)]
mod tests;

struct GameResult();

struct GameRunner {
    tokio_rt: tokio::runtime::Runtime,
}

type WrappedGame = Arc<Mutex<Game>>;

struct TaskRunner {
    task_id: TaskId,
    game: WrappedGame,
    rx: mpsc::Receiver<Message>,
    tx: mpsc::Sender<Message>,
}

impl TaskRunner {
    async fn run(&mut self) {
        while let Some(_content) = self.rx.recv().await {
            println!("{:?}", self.task_id);
            self.tx.send(Message::None).await.unwrap();
        }
    }
}

impl GameRunner {
    fn new() -> GameRunner {
        GameRunner {
            tokio_rt: tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap(),
        }
    }

    fn run_game(&self) -> GameResult {
        let game_result = self.tokio_rt.block_on(self.start_game());

        game_result
    }

    async fn start_game(&self) -> GameResult {
        let game: WrappedGame = Arc::new(Mutex::new(Game::new()));

        let tasks = game.lock().unwrap().get_all_task_ids();
        let mut handles = Vec::new();

        let mut channels = Vec::new();

        let (continue_game_tx, mut continue_game_rx): (
            mpsc::Sender<Message>,
            mpsc::Receiver<Message>,
        ) = mpsc::channel(32);

        for t in tasks {
            let (tx, rx): (mpsc::Sender<Message>, mpsc::Receiver<Message>) = mpsc::channel(32);
            let continue_tx = continue_game_tx.clone();
            let game_copy = game.clone();
            let handle = tokio::spawn(async move {
                let mut task_runner = TaskRunner {
                    task_id: t,
                    game: game_copy,
                    rx,
                    tx: continue_tx,
                };

                task_runner.run().await;
            });

            channels.push(tx);
            handles.push(handle)
        }

        continue_game_tx.send(Message::None).await.unwrap();
        let mut index = 0;
        while let Some(_content) = continue_game_rx.recv().await {
            if index >= channels.len() {
                break;
            }
            let m = channels[index].send(Message::None).await.unwrap();
            index = index + 1;
        }

        for i in channels {
            drop(i);
        }

        for i in handles {
            let _ = i.await;
        }

        GameResult()
    }
}
