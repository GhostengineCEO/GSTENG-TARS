use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct Safety {
    last_move: Arc<Mutex<Instant>>,
    last_watchdog: Arc<Mutex<Instant>>,
    emergency: Arc<Mutex<bool>>,
    pub rate_limit: Duration,
    pub servo_min: f32,
    pub servo_max: f32,
    pub connection_timeout: Duration,
}

pub type SharedSafety = Arc<Safety>;

impl Safety {
    pub fn new() -> SharedSafety {
        Arc::new(Safety {
            last_move: Arc::new(Mutex::new(Instant::now())),
            last_watchdog: Arc::new(Mutex::new(Instant::now())),
            emergency: Arc::new(Mutex::new(false)),
            rate_limit: Duration::from_millis(100),
            servo_min: -1.57,
            servo_max: 1.57,
            connection_timeout: Duration::from_secs(3),
        })
    }

    pub async fn check_move_allowed(&self) -> bool {
        let mut last = self.last_move.lock().await;
        let now = Instant::now();
        if now.duration_since(*last) >= self.rate_limit {
            *last = now;
            true
        } else {
            false
        }
    }

    pub fn check_servo_bounds(&self, pos: f32) -> bool {
        pos >= self.servo_min && pos <= self.servo_max
    }

    pub async fn trigger_emergency(&self) {
        *self.emergency.lock().await = true;
    }

    pub async fn is_emergency(&self) -> bool {
        *self.emergency.lock().await
    }

    pub async fn feed_watchdog(&self) {
        *self.last_watchdog.lock().await = Instant::now();
    }
}

pub fn start_watchdog(safety: SharedSafety) {
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(safety.connection_timeout).await;
            let last = *safety.last_watchdog.lock().await;
            if Instant::now().duration_since(last) > safety.connection_timeout {
                safety.trigger_emergency().await;
            }
        }
    });
}
