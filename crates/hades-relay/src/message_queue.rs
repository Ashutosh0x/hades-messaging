use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;

const MAX_QUEUE_PER_USER: usize = 10_000;
const MAX_MESSAGE_AGE_SECS: u64 = 7 * 24 * 3600; // 7 days

#[derive(Debug, Clone)]
pub struct QueuedMessage {
    pub sender_id: String,
    pub envelope: Vec<u8>,
    pub timestamp: u64,
    pub message_id: String,
}

/// In-memory message queue for offline users.
/// Messages are Sealed Sender encrypted — relay can't read them.
pub struct MessageQueue {
    queues: Arc<RwLock<HashMap<String, VecDeque<QueuedMessage>>>>,
}

impl MessageQueue {
    pub fn new() -> Self {
        Self { queues: Arc::new(RwLock::new(HashMap::new())) }
    }

    pub async fn enqueue(&self, recipient_id: &str, msg: QueuedMessage) -> bool {
        let mut queues = self.queues.write().await;
        let queue = queues.entry(recipient_id.to_string()).or_default();
        if queue.len() >= MAX_QUEUE_PER_USER { queue.pop_front(); }
        queue.push_back(msg);
        true
    }

    pub async fn drain(&self, recipient_id: &str) -> Vec<QueuedMessage> {
        let mut queues = self.queues.write().await;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
        match queues.remove(recipient_id) {
            Some(queue) => queue.into_iter()
                .filter(|m| now - m.timestamp < MAX_MESSAGE_AGE_SECS).collect(),
            None => vec![],
        }
    }

    pub async fn queue_size(&self, recipient_id: &str) -> usize {
        let queues = self.queues.read().await;
        queues.get(recipient_id).map(|q| q.len()).unwrap_or(0)
    }

    pub async fn cleanup_expired(&self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
        let mut queues = self.queues.write().await;
        for queue in queues.values_mut() {
            queue.retain(|m| now - m.timestamp < MAX_MESSAGE_AGE_SECS);
        }
        queues.retain(|_, q| !q.is_empty());
    }
}

impl Default for MessageQueue {
    fn default() -> Self { Self::new() }
}
