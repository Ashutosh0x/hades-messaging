use crate::state::AppState;
use crate::websocket::RelayMessage;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

const TYPING_TIMEOUT_SECS: u64 = 5;
const TYPING_THROTTLE_MS: u64 = 3000;

pub struct TypingManager {
    remote_typing: Arc<RwLock<HashMap<String, Instant>>>,
    last_sent: Arc<RwLock<Instant>>,
}

impl TypingManager {
    pub fn new() -> Self {
        Self {
            remote_typing: Arc::new(RwLock::new(HashMap::new())),
            last_sent: Arc::new(RwLock::new(Instant::now() - Duration::from_secs(60))),
        }
    }

    pub async fn on_local_typing(&self, state: &Arc<RwLock<AppState>>, conversation_id: &str) {
        let mut last = self.last_sent.write().await;
        if last.elapsed() < Duration::from_millis(TYPING_THROTTLE_MS) { return; }
        *last = Instant::now();

        let s = state.read().await;
        if let Some(ref relay) = s.relay {
            let _ = relay.send(RelayMessage::Send {
                recipient_id: conversation_id.to_string(),
                envelope: b"__TYPING__".to_vec(),
                message_id: String::new(),
            }).await;
        }
    }

    pub async fn on_remote_typing(&self, contact_id: &str) {
        let mut map = self.remote_typing.write().await;
        map.insert(contact_id.to_string(), Instant::now());
    }

    pub async fn is_typing(&self, contact_id: &str) -> bool {
        let map = self.remote_typing.read().await;
        map.get(contact_id)
            .map(|t| t.elapsed() < Duration::from_secs(TYPING_TIMEOUT_SECS))
            .unwrap_or(false)
    }

    pub async fn cleanup(&self) {
        let mut map = self.remote_typing.write().await;
        let now = Instant::now();
        map.retain(|_, t| now.duration_since(*t) < Duration::from_secs(TYPING_TIMEOUT_SECS));
    }
}

impl Default for TypingManager {
    fn default() -> Self { Self::new() }
}
