use crate::{game::TaskId, runtime::bots::BotFactory};

use super::*;

#[tokio::test]
async fn look_once() {
    let factory = WasmBotFactory::new("wasm_modules/look-once.wasm").unwrap();

    let mut bot = factory.create_bot(TaskId(0, 0)).await;

    let p = bot.poll().await.unwrap();

    assert_eq!(Command::Look(17, 23), p);

}

#[tokio::test]
async fn move_down() {
    let factory = WasmBotFactory::new("wasm_modules/move-down.wasm").unwrap();

    let mut bot = factory.create_bot(TaskId(0, 0)).await;

    let p = bot.poll().await.unwrap();

    assert_eq!(Command::Move(17, Direction::Down), p);

}

#[tokio::test]
async fn move_left() {
    let factory = WasmBotFactory::new("wasm_modules/move-left.wasm").unwrap();

    let mut bot = factory.create_bot(TaskId(0, 0)).await;

    let p = bot.poll().await.unwrap();

    assert_eq!(Command::Move(17, Direction::Left), p);

}

#[tokio::test]
async fn split() {
    let factory = WasmBotFactory::new("wasm_modules/split-once.wasm").unwrap();

    let mut bot = factory.create_bot(TaskId(0, 0)).await;

    let p = bot.poll().await.unwrap();

    assert_eq!(Command::Split, p);

}