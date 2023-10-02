use super::*;

mod test_factory {
    use super::*;

    pub(super) fn create_task_handle(task_id: TaskId, used_fuel: isize) -> TaskHandle {
        let (tx, _rx) = tokio::sync::mpsc::channel(32);
        let h = tokio::spawn(async {});

        TaskHandle {
            context: Arc::new(Mutex::new(TaskContext { used_fuel, task_id })),
            task_id,
            tx,
            handle: h,
            timestamp: 0,
        }
    }
}

#[test]
fn start_runtime() {
    let factory = bots::RandomBotFactory::new();
    let runner = GameRunner::new(factory);

    let _result = runner.run_game();
}

#[test]
fn first_move_is_split() {
    let factory = bots::MockedBotFactory::new()
        .mock(TaskId(0, 0), bots::Command::Split)
        .mock(TaskId(1, 0), bots::Command::Move(1, Direction::Left));

    let runner = GameRunner::new(factory);

    let _result = runner.run_game();
}

#[tokio::test]
async fn task_handle_comparison() {
    let t1 = test_factory::create_task_handle(TaskId(0, 1), 2);
    let t2 = test_factory::create_task_handle(TaskId(0, 1), 3);

    // Inverse order is needed for binary_heap.
    assert!(t1 > t2);
}
