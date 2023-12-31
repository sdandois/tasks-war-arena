use crate::{game::TaskId, runtime::bots::BotFactory};

use super::*;

#[tokio::test]
async fn look_once() {
    let before = std::time::Instant::now();

    let factory = WasmBotFactory::fetch_from_cache("./wasm_cache/look-once", || {
        WasmBotFactory::same_module("wasm_modules/look-once.wasm")
    });

    let mut bot = factory.create_bot(TaskId(0, 0), 64).await;

    let p = bot.poll().await.unwrap().0;
    println!("{:?}", (std::time::Instant::now() - before).as_millis());

    assert_eq!(Command::Look(17, 23), p);
}

#[tokio::test]

async fn move_down() {
    let factory = WasmBotFactory::fetch_from_cache("./wasm_cache/move-down", || {
        WasmBotFactory::same_module("wasm_modules/move-down.wasm")
    });

    let mut bot = factory.create_bot(TaskId(0, 0), 64).await;

    let p = bot.poll().await.unwrap().0;

    assert_eq!(Command::Move(Direction::Down), p);
}

#[tokio::test]
async fn move_left() {
    let factory = WasmBotFactory::fetch_from_cache("./wasm_cache/move-left", || {
        WasmBotFactory::same_module("wasm_modules/move-left.wasm")
    });

    let mut bot = factory.create_bot(TaskId(0, 0), 64).await;

    let p = bot.poll().await.unwrap().0;

    assert_eq!(Command::Move(Direction::Left), p);
}

#[tokio::test]
async fn split() {
    let factory = WasmBotFactory::fetch_from_cache("./wasm_cache/split-once", || {
        WasmBotFactory::same_module("wasm_modules/split-once.wasm")
    });

    let mut bot = factory.create_bot(TaskId(0, 0), 64).await;

    let p = bot.poll().await.unwrap().0;

    assert_eq!(Command::Split, p);
}

#[tokio::test]
async fn different_modules_for_players() {
    let factory =
        WasmBotFactory::fetch_from_cache("./wasm_cache/different_modules_for_player", || {
            WasmBotFactory::new(
                "wasm_modules/split-once.wasm",
                "wasm_modules/look-once.wasm",
            )
        });

    let mut bot0 = factory.create_bot(TaskId(0, 0), 64).await;
    let mut bot1 = factory.create_bot(TaskId(1, 0), 64).await;

    let (c0, _) = bot0.poll().await.unwrap();
    let (c1, _) = bot1.poll().await.unwrap();

    assert_eq!(Command::Split, c0);
    assert_eq!(Command::Look(17, 23), c1);
}

#[tokio::test]
async fn action_depends_on_look_result() {
    let factory = WasmBotFactory::fetch_from_cache("./wasm_cache/look-if-empty", || {
        WasmBotFactory::same_module("wasm_modules/look-if-empty.wasm")
    });

    let mut bot = factory.create_bot(TaskId(0, 0), 64).await;

    let (c, _) = bot.poll().await.unwrap();

    assert_eq!(Command::Look(1, 1), c);
    bot.update(Some(LookResult::Null)).await;

    let (c, _) = bot.poll().await.unwrap();

    assert_eq!(Command::Move(Direction::Right), c);
    bot.update(None).await;

    let (c, _) = bot.poll().await.unwrap();

    assert_eq!(Command::Look(1, 1), c);
    bot.update(Some(LookResult::None)).await;

    let (c, _) = bot.poll().await.unwrap();

    assert_eq!(Command::Move(Direction::Right), c);
    bot.update(None).await;

    let (c, _) = bot.poll().await.unwrap();

    assert_eq!(Command::Look(1, 1), c);
    bot.update(Some(LookResult::Player)).await;

    let (c, _) = bot.poll().await.unwrap();

    assert_eq!(Command::Move(Direction::Down), c);
    bot.update(None).await;

    let (c, _) = bot.poll().await.unwrap();

    assert_eq!(Command::Look(1, 1), c);
    bot.update(Some(LookResult::Opponent)).await;

    let (c, _) = bot.poll().await.unwrap();

    assert_eq!(Command::Move(Direction::Left), c);
    bot.update(None).await;

    let (c, _) = bot.poll().await.unwrap();

    assert_eq!(Command::Look(1, 1), c);
    bot.update(Some(LookResult::Food)).await;

    let (c, _) = bot.poll().await.unwrap();

    assert_eq!(Command::Move(Direction::Up), c);
    bot.update(None).await;
}
