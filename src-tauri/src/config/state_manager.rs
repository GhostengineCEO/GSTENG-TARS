use std::{cmp::Ordering, collections::BinaryHeap, sync::Arc};
use tokio::sync::{broadcast, Mutex};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RobotState {
    Idle,
    Listening,
    Thinking,
    Moving,
}

#[derive(Debug, Eq)]
pub struct Command {
    pub priority: u8,
    pub action: String,
}

impl Ord for Command {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority.cmp(&other.priority)
    }
}

impl PartialOrd for Command {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Command {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.action == other.action
    }
}

pub struct StateManager {
    state: Arc<Mutex<RobotState>>,
    queue: Arc<Mutex<BinaryHeap<Command>>>,
    event_tx: broadcast::Sender<RobotState>,
}

impl StateManager {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(16);
        Self {
            state: Arc::new(Mutex::new(RobotState::Idle)),
            queue: Arc::new(Mutex::new(BinaryHeap::new())),
            event_tx: tx,
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<RobotState> {
        self.event_tx.subscribe()
    }

    pub async fn current_state(&self) -> RobotState {
        *self.state.lock().await
    }

    pub async fn set_state(&self, new: RobotState) {
        *self.state.lock().await = new;
        let _ = self.event_tx.send(new);
    }

    pub async fn enqueue_command(&self, cmd: Command) {
        self.queue.lock().await.push(cmd);
    }

    pub async fn next_command(&self) -> Option<Command> {
        self.queue.lock().await.pop()
    }
}
