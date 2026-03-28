use crate::error::{AppError, AppResult};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use tokio::sync::{mpsc, Mutex, Notify};

const MAX_RECONNECT_DELAY_SECS: u64 = 120;
const INITIAL_RECONNECT_DELAY_MS: u64 = 500;
const PING_INTERVAL_SECS: u64 = 30;
const PONG_TIMEOUT_SECS: u64 = 10;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RelayMessage {
    #[serde(rename = "send")]
    Send { recipient_id: String, envelope: Vec<u8>, message_id: String },
    #[serde(rename = "receive")]
    Receive { sender_id: Option<String>, envelope: Vec<u8>, timestamp: String },
    #[serde(rename = "receipt")]
    Receipt { message_id: String, status: String },
    #[serde(rename = "prekey_request")]
    PrekeyRequest { identity_id: String },
    #[serde(rename = "prekey_response")]
    PrekeyResponse { identity_id: String, bundle: Vec<u8> },
    #[serde(rename = "ping")]
    Ping,
    #[serde(rename = "pong")]
    Pong,
}

pub struct RelayConnection {
    outgoing_tx: mpsc::Sender<RelayMessage>,
    pub incoming_rx: Arc<Mutex<mpsc::Receiver<RelayMessage>>>,
    connected: Arc<AtomicBool>,
    reconnect_count: Arc<AtomicU32>,
    shutdown: Arc<Notify>,
}

impl RelayConnection {
    pub async fn connect(url: &str, identity_pubkey: &[u8]) -> AppResult<Self> {
        let (out_tx, out_rx) = mpsc::channel::<RelayMessage>(512);
        let (in_tx, in_rx) = mpsc::channel::<RelayMessage>(512);
        let connected = Arc::new(AtomicBool::new(false));
        let reconnect_count = Arc::new(AtomicU32::new(0));
        let shutdown = Arc::new(Notify::new());

        let conn = Self {
            outgoing_tx: out_tx,
            incoming_rx: Arc::new(Mutex::new(in_rx)),
            connected: connected.clone(),
            reconnect_count: reconnect_count.clone(),
            shutdown: shutdown.clone(),
        };

        let url_owned = url.to_string();
        let identity_owned = identity_pubkey.to_vec();

        tokio::spawn(Self::connection_loop(
            url_owned, identity_owned, out_rx, in_tx,
            connected, reconnect_count, shutdown,
        ));

        Ok(conn)
    }

    async fn connection_loop(
        url: String, identity: Vec<u8>,
        mut out_rx: mpsc::Receiver<RelayMessage>,
        in_tx: mpsc::Sender<RelayMessage>,
        connected: Arc<AtomicBool>,
        reconnect_count: Arc<AtomicU32>,
        shutdown: Arc<Notify>,
    ) {
        let mut delay_ms = INITIAL_RECONNECT_DELAY_MS;

        loop {
            log::info!("Connecting to relay: {}", url);

            match tokio_tungstenite::connect_async(&url).await {
                Ok((ws_stream, _)) => {
                    connected.store(true, Ordering::SeqCst);
                    reconnect_count.store(0, Ordering::SeqCst);
                    delay_ms = INITIAL_RECONNECT_DELAY_MS;
                    log::info!("Connected to relay");

                    let (mut ws_sink, mut ws_source) = ws_stream.split();

                    // Send registration
                    let auth = serde_json::json!({
                        "type": "register",
                        "identity": hex::encode(&identity),
                    });
                    if ws_sink.send(tokio_tungstenite::tungstenite::Message::Text(
                        auth.to_string()
                    )).await.is_err() {
                        connected.store(false, Ordering::SeqCst);
                        continue;
                    }

                    let mut ping_interval = tokio::time::interval(
                        tokio::time::Duration::from_secs(PING_INTERVAL_SECS)
                    );
                    let last_pong = Arc::new(Mutex::new(tokio::time::Instant::now()));
                    let last_pong_check = last_pong.clone();

                    loop {
                        tokio::select! {
                            Some(msg) = out_rx.recv() => {
                                let json = match serde_json::to_string(&msg) {
                                    Ok(j) => j,
                                    Err(_) => continue,
                                };
                                if ws_sink.send(
                                    tokio_tungstenite::tungstenite::Message::Text(json)
                                ).await.is_err() {
                                    break;
                                }
                            }
                            msg = ws_source.next() => {
                                match msg {
                                    Some(Ok(tokio_tungstenite::tungstenite::Message::Text(text))) => {
                                        if let Ok(relay_msg) = serde_json::from_str::<RelayMessage>(&text) {
                                            match &relay_msg {
                                                RelayMessage::Pong => {
                                                    *last_pong.lock().await = tokio::time::Instant::now();
                                                }
                                                _ => {
                                                    if in_tx.send(relay_msg).await.is_err() { break; }
                                                }
                                            }
                                        }
                                    }
                                    Some(Ok(tokio_tungstenite::tungstenite::Message::Binary(data))) => {
                                        let msg = RelayMessage::Receive {
                                            sender_id: None,
                                            envelope: data.to_vec(),
                                            timestamp: unix_now(),
                                        };
                                        if in_tx.send(msg).await.is_err() { break; }
                                    }
                                    Some(Ok(tokio_tungstenite::tungstenite::Message::Close(_))) | None => break,
                                    Some(Err(e)) => {
                                        log::error!("WebSocket error: {}", e);
                                        break;
                                    }
                                    _ => {}
                                }
                            }
                            _ = ping_interval.tick() => {
                                let elapsed = last_pong_check.lock().await.elapsed();
                                if elapsed > tokio::time::Duration::from_secs(
                                    PING_INTERVAL_SECS + PONG_TIMEOUT_SECS
                                ) {
                                    log::warn!("Pong timeout, reconnecting");
                                    break;
                                }
                                let ping = serde_json::json!({"type": "ping"});
                                if ws_sink.send(
                                    tokio_tungstenite::tungstenite::Message::Text(ping.to_string())
                                ).await.is_err() {
                                    break;
                                }
                            }
                            _ = shutdown.notified() => {
                                let _ = ws_sink.send(
                                    tokio_tungstenite::tungstenite::Message::Close(None)
                                ).await;
                                connected.store(false, Ordering::SeqCst);
                                return;
                            }
                        }
                    }
                    connected.store(false, Ordering::SeqCst);
                }
                Err(e) => {
                    log::warn!("Connection failed: {}", e);
                }
            }

            let count = reconnect_count.fetch_add(1, Ordering::SeqCst);
            let jitter = rand::random::<u64>() % (delay_ms / 4 + 1);
            let wait = std::cmp::min(delay_ms + jitter, MAX_RECONNECT_DELAY_SECS * 1000);
            log::info!("Reconnecting in {}ms (attempt {})", wait, count + 1);

            tokio::select! {
                _ = tokio::time::sleep(tokio::time::Duration::from_millis(wait)) => {}
                _ = shutdown.notified() => return,
            }
            delay_ms = std::cmp::min(delay_ms * 2, MAX_RECONNECT_DELAY_SECS * 1000);
        }
    }

    pub async fn send(&self, msg: RelayMessage) -> AppResult<()> {
        self.outgoing_tx.send(msg).await
            .map_err(|_| AppError::WebSocket("Send channel closed".into()))
    }

    pub fn is_connected(&self) -> bool { self.connected.load(Ordering::SeqCst) }

    pub fn reconnect_count(&self) -> u32 { self.reconnect_count.load(Ordering::SeqCst) }

    pub fn disconnect(&self) { self.shutdown.notify_one(); }
}

impl Drop for RelayConnection {
    fn drop(&mut self) { self.shutdown.notify_one(); }
}

fn unix_now() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .to_string()
}
