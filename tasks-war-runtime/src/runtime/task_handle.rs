use super::*;
pub(super) struct TaskHandle {
    pub context: Arc<Mutex<TaskContext>>,
    pub task_id: TaskId,
    pub tx: mpsc::Sender<TaskRequest>,
    pub rx: mpsc::Receiver<TaskResponse>,
    pub handle: JoinHandle<()>,
    pub timestamp: usize,
}

impl core::fmt::Debug for TaskHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TaskHandle")
            .field("task_id", &self.task_id)
            .field("used_fuel", &self.context.lock().unwrap().used_fuel)
            .field("timestamp", &self.timestamp)
            .finish()
    }
}

#[derive(Clone)]
pub(super) struct TaskContext {
    pub used_fuel: isize,
    pub task_id: TaskId,
}

impl PartialOrd for TaskHandle {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TaskHandle {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let task_context = self.context.lock().unwrap();

        let ordering = task_context
            .used_fuel
            .cmp(&other.context.lock().unwrap().used_fuel);

        let comparison = match ordering {
            std::cmp::Ordering::Equal => self.timestamp.cmp(&other.timestamp),
            _ => ordering,
        };

        comparison.reverse()
    }
}

impl PartialEq for TaskHandle {
    fn eq(&self, other: &Self) -> bool {
        self.context.lock().unwrap().used_fuel == other.context.lock().unwrap().used_fuel
    }
}

impl Eq for TaskHandle {}
