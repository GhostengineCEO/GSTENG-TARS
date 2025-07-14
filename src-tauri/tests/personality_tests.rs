use gsteng::commands::set_personality;
use gsteng::config::config::{Config, SharedConfig};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn personality_values_clamped() {
    let cfg = Arc::new(Mutex::new(Config::default()));
    let state = tauri::State::new(cfg.clone());
    set_personality(2.0, -1.0, 0.5, state).await.unwrap();
    let lock = cfg.lock().await;
    assert_eq!(lock.personality.humor, 1.0);
    assert_eq!(lock.personality.honesty, 0.0);
}
