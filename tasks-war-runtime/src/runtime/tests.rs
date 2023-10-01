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
        }
    }
}

#[test]
fn start_runtime() {
    let runner = GameRunner::<RandomBot>::new();

    let _result = runner.run_game();
}

#[tokio::test]
async fn task_handle_comparison() {
    let t1 = test_factory::create_task_handle(TaskId(0, 1), 2);
    let t2 = test_factory::create_task_handle(TaskId(0, 1), 3);

    // Inverse order is needed for binary_heap.
    assert!(t1 > t2);
}
