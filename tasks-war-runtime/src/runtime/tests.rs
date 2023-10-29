use crate::{
    game::{BoardSize, Direction},
    game_with_history::Command,
};

use super::*;

mod test_factory {
    use super::*;

    pub(super) fn create_task_handle(task_id: TaskId, used_fuel: isize) -> TaskHandle {
        let (tx, _rx) = tokio::sync::mpsc::channel(32);
        let (_tx, rx) = tokio::sync::mpsc::channel(32);
        let h = tokio::spawn(async {});

        TaskHandle {
            context: Arc::new(Mutex::new(TaskContext { used_fuel, task_id })),
            task_id,
            tx,
            rx,
            handle: h,
            timestamp: 0,
        }
    }
}

#[test]
#[ignore = "too slow"]
fn start_runtime() {
    let factory = bots::RandomBotFactory::new();
    let runner = GameRunner::new(factory);

    let _result = runner.run_game();
}

#[test]
fn first_move_is_split() {
    let factory = bots::MockedBotFactory::new()
        .mock(TaskId(0, 0), Command::Split)
        .mock(TaskId(1, 0), Command::Move(1, Direction::Left))
        .mock(TaskId(0, 1), Command::Move(1, Direction::Left));

    let runner = GameRunner::new(factory);

    let result = runner.run_some_rounds(15);

    assert_eq!(result.get_task(TaskId(1, 0)).pos, (34, 20));
    assert_eq!(result.get_task(TaskId(0, 0)).pos, (10, 25));
    assert_eq!(result.get_task(TaskId(0, 1)).pos, (10, 24));

    assert_eq!(3, result.get_all_task_ids().len());
}

#[test]
fn if_look_result_is_empty_move_right() {
    let factory = bots::MockedBotFactory::new()
        .mock(TaskId(0, 0), Command::Look(1, 0))
        .mock_fn(TaskId(0, 0), |r| {
            if let crate::game::LookResult::Food = r.unwrap() {
                Command::Move(1, Direction::Left)
            } else {
                Command::Move(1, Direction::Right)
            }
        });

    let runner = GameRunner::new(factory);

    let result = runner.run_some_rounds(15);

    assert_eq!(result.get_task(TaskId(0, 0)).pos, (10, 26));
}

#[test]
fn if_look_result_is_empty_move_left() {
    let factory = bots::MockedBotFactory::new()
        .mock(TaskId(0, 0), Command::Look(0, 2))
        .mock_fn(TaskId(0, 0), |r| {
            if let crate::game::LookResult::Food = r.unwrap() {
                Command::Move(1, Direction::Left)
            } else {
                Command::Move(1, Direction::Right)
            }
        });

    let runner = GameRunner::new(factory);

    let result = runner.run_some_rounds(15);

    assert_eq!(result.get_task(TaskId(0, 0)).pos, (10, 24));
}

#[test]
fn dont_play_if_killed() {
    let mut factory = bots::MockedBotFactory::new()
        .mock(TaskId(0, 0), Command::Pass)
        .mock(TaskId(1, 0), Command::Split)
        .mock(TaskId(1, 1), Command::Move(1, Direction::Down));

    for _i in 0..24 {
        factory = factory.mock(TaskId(0, 0), Command::Move(1, Direction::Down));
        factory = factory.mock(TaskId(1, 0), Command::Pass);
    }

    for _i in 0..3 {
        factory = factory.mock(TaskId(0, 0), Command::Move(1, Direction::Left));
        factory = factory.mock(TaskId(1, 0), Command::Pass);
    }

    factory = factory.mock(TaskId(0, 0), Command::Move(1, Direction::Left));
    factory = factory.mock(TaskId(1, 0), Command::Move(1, Direction::Up));

    let runner = GameRunner::new(factory);

    let result = runner.run_some_rounds(100);

    assert!(result.get_task(TaskId(1, 0)).is_dead);

    println!("{:?}", result.get_task(TaskId(0, 0)).pos);
    println!("{:?}", result.get_task(TaskId(1, 0)).pos);
    println!("{}", result);
}

#[tokio::test]
async fn task_handle_comparison() {
    let t1 = test_factory::create_task_handle(TaskId(0, 1), 2);
    let t2 = test_factory::create_task_handle(TaskId(0, 1), 3);

    // Inverse order is needed for binary_heap.
    assert!(t1 > t2);
}

#[test]
fn if_bot_panicks_runner_should_finish() {
    let factory = bots::MockedBotFactory::new()
        .mock(TaskId(0, 0), Command::Look(0, 2))
        .mock_fn(TaskId(0, 0), |_r| panic!("my bot implementation panics"));

    let runner = GameRunner::new(factory);

    let _result = runner.run_some_rounds(15);
}

#[test]
fn wasm_bot_game() {
    let factory =
        bots::WasmBotFactory::same_module("wasm_modules/move-right-forever.wasm").unwrap();

    let runner = crate::runtime::GameRunner::new(factory);

    let _result = runner.run_some_rounds(10);
}

#[test]
#[ignore = "too slow"]
fn full_game_finishes_with_fuel_error() {
    let factory =
        bots::WasmBotFactory::same_module("wasm_modules/move-right-forever.wasm").unwrap();

    let runner = crate::runtime::GameRunner::new(factory);

    let _result = runner.run_game();
}

#[tokio::test]
async fn cloned_tasks_have_same_used_fuel() {
    let factory = bots::MockedBotFactory::new()
        .mock(TaskId(0, 0), Command::Move(1, Direction::Left))
        .mock(TaskId(0, 0), Command::Move(1, Direction::Left))
        .mock(TaskId(0, 0), Command::Move(1, Direction::Left))
        .mock(TaskId(0, 0), Command::Move(1, Direction::Left))
        .mock(TaskId(0, 0), Command::Split);

    let mut runner = RunnerContext::new(factory, 9, GameConfig::default());

    let _result = runner.spawn_all_tasks().await;
    let _result = runner.play_rounds().await;

    let vec: Vec<_> = runner.handles.iter().map(|th| th.context.clone()).collect();

    let used_fuel_00 = vec
        .iter()
        .find(|c| c.lock().unwrap().task_id == TaskId(0, 0))
        .map(|c| c.lock().unwrap().used_fuel)
        .unwrap();
    let used_fuel_01 = vec
        .iter()
        .find(|c| c.lock().unwrap().task_id == TaskId(0, 1))
        .map(|c| c.lock().unwrap().used_fuel)
        .unwrap();

    assert!(used_fuel_00 > 2);
    assert_eq!(used_fuel_00, used_fuel_01);
}

#[test]
fn create_runner_with_config() {
    let factory = bots::MockedBotFactory::new()
        .mock(TaskId(0, 0), Command::Split)
        .mock(TaskId(1, 0), Command::Move(1, Direction::Left))
        .mock(TaskId(0, 1), Command::Move(1, Direction::Left));

    let config = GameConfig {
        board_size: BoardSize(10, 10),
        seed: 800,
    };
    let runner = GameRunner::with_config(factory, config);

    let result = runner.run_some_rounds(15);

    let obtained_config = result.get_config();

    assert_eq!(obtained_config.board_size, BoardSize(10, 10));
    assert_eq!(obtained_config.seed, 800);
}
