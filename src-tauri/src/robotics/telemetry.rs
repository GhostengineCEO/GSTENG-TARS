//! Telemetry system for broadcasting robot state.

use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, Mutex};
use futures_util::{StreamExt, SinkExt};
use tokio_tungstenite::tungstenite::Message;

/// Container for telemetry operations.
pub struct Telemetry {
    tx: broadcast::Sender<String>,
    log: Arc<Mutex<Vec<String>>>,
}

impl Telemetry {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(16);
        Self { tx, log: Arc::new(Mutex::new(Vec::new())) }
    }

    /// Start the WebSocket server used by the dashboard.
    pub async fn start_server(&self, addr: &str) -> Result<(), String> {
        let listener = TcpListener::bind(addr).await.map_err(|e| e.to_string())?;
        while let Ok((stream, _)) = listener.accept().await {
            let tx = self.tx.clone();
            let log = self.log.clone();
            tokio::spawn(handle_connection(stream, tx, log));
        }
        Ok(())
    }

    /// Broadcast new telemetry data to listeners and log it.
    pub async fn broadcast(&self, data: String) {
        let _ = self.tx.send(data.clone());
        self.log.lock().await.push(data);
    }

    /// Replay logged data to a subscriber.
    pub async fn replay(&self) -> Vec<String> {
        self.log.lock().await.clone()
    }
}

async fn handle_connection(stream: TcpStream, tx: broadcast::Sender<String>, log: Arc<Mutex<Vec<String>>>) {
    if let Ok(ws_stream) = tokio_tungstenite::accept_async(stream).await {
        let (mut write, mut read) = ws_stream.split();
        let mut rx = tx.subscribe();
        // send historical data first
        for entry in log.lock().await.iter() {
            let _ = write.send(Message::Text(entry.clone())).await;
        }
        tokio::spawn(async move {
            while let Ok(msg) = rx.recv().await {
                let _ = write.send(Message::Text(msg)).await;
            }
        });
        // consume incoming messages, ignore them
        while let Some(Ok(_)) = read.next().await {}
    }
}

