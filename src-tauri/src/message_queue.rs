use serde::{Deserialize, Serialize};
use std::collections::BinaryHeap;
use std::cmp::Ordering;

/// Message priority levels for the outbound queue.
/// Higher priority messages are sent first.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub enum MessagePriority {
    Low = 0,        // Typing indicators, presence
    Normal = 1,     // Status updates, read receipts
    High = 2,       // Regular user messages
    Critical = 3,   // Key exchange, auth, session setup
}

/// A message waiting to be sent (or retried) via the relay.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct QueuedMessage {
    pub id: String,
    pub recipient_id: String,
    pub envelope: Vec<u8>,
    pub priority: MessagePriority,
    pub created_at: u64,
    pub retry_count: u32,
    pub next_retry_at: u64,
}

impl Ord for QueuedMessage {
    fn cmp(&self, other: &Self) -> Ordering {
        // Higher priority first, then older messages first, then earlier retry
        self.priority
            .cmp(&other.priority)
            .then_with(|| other.created_at.cmp(&self.created_at))
            .then_with(|| other.next_retry_at.cmp(&self.next_retry_at))
    }
}

impl PartialOrd for QueuedMessage {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Priority message queue with exponential backoff retry.
pub struct MessageQueue {
    heap: BinaryHeap<QueuedMessage>,
    max_retries: u32,
    backoff_ms: Vec<u64>,
}

impl Default for MessageQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl MessageQueue {
    pub fn new() -> Self {
        Self {
            heap: BinaryHeap::new(),
            max_retries: 5,
            // Exponential backoff: 1s, 2s, 4s, 8s, 16s
            backoff_ms: vec![1_000, 2_000, 4_000, 8_000, 16_000],
        }
    }

    /// Enqueue a new message for sending.
    pub fn enqueue(&mut self, msg: QueuedMessage) {
        self.heap.push(msg);
    }

    /// Remove and return the highest-priority message that is ready to send.
    /// Returns None if the queue is empty or all messages are waiting for retry.
    pub fn dequeue_ready(&mut self, now_ms: u64) -> Option<QueuedMessage> {
        // Peek to check if the top message is ready
        if let Some(top) = self.heap.peek() {
            if top.next_retry_at <= now_ms {
                return self.heap.pop();
            }
        }
        None
    }

    /// Re-enqueue a message for retry with exponential backoff.
    /// Returns false if the message has exceeded max retries.
    pub fn retry(&mut self, mut msg: QueuedMessage) -> bool {
        msg.retry_count += 1;
        if msg.retry_count > self.max_retries {
            return false; // Caller should mark as permanently failed
        }

        let delay = self
            .backoff_ms
            .get((msg.retry_count - 1) as usize)
            .copied()
            .unwrap_or(32_000);

        msg.next_retry_at = now_ms() + delay;
        self.heap.push(msg);
        true
    }

    /// Number of messages in the queue.
    pub fn len(&self) -> usize {
        self.heap.len()
    }

    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }

    /// Drain all messages (e.g. for persistence on shutdown).
    pub fn drain(&mut self) -> Vec<QueuedMessage> {
        let mut out = Vec::with_capacity(self.heap.len());
        while let Some(msg) = self.heap.pop() {
            out.push(msg);
        }
        out
    }
}

/// Current Unix timestamp in milliseconds.
pub fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_ordering() {
        let mut q = MessageQueue::new();
        let now = now_ms();

        q.enqueue(QueuedMessage {
            id: "low".into(),
            recipient_id: "bob".into(),
            envelope: vec![],
            priority: MessagePriority::Low,
            created_at: now,
            retry_count: 0,
            next_retry_at: 0,
        });
        q.enqueue(QueuedMessage {
            id: "critical".into(),
            recipient_id: "bob".into(),
            envelope: vec![],
            priority: MessagePriority::Critical,
            created_at: now,
            retry_count: 0,
            next_retry_at: 0,
        });
        q.enqueue(QueuedMessage {
            id: "high".into(),
            recipient_id: "bob".into(),
            envelope: vec![],
            priority: MessagePriority::High,
            created_at: now,
            retry_count: 0,
            next_retry_at: 0,
        });

        assert_eq!(q.dequeue_ready(now).unwrap().id, "critical");
        assert_eq!(q.dequeue_ready(now).unwrap().id, "high");
        assert_eq!(q.dequeue_ready(now).unwrap().id, "low");
    }

    #[test]
    fn test_retry_backoff() {
        let mut q = MessageQueue::new();
        let now = now_ms();

        let msg = QueuedMessage {
            id: "test".into(),
            recipient_id: "bob".into(),
            envelope: vec![1, 2, 3],
            priority: MessagePriority::High,
            created_at: now,
            retry_count: 0,
            next_retry_at: 0,
        };

        // First retry should succeed
        assert!(q.retry(msg.clone()));
        assert_eq!(q.len(), 1);

        // Should not be ready immediately (backoff)
        assert!(q.dequeue_ready(now).is_none());

        // Should be ready after delay
        assert!(q.dequeue_ready(now + 2_000).is_some());
    }

    #[test]
    fn test_max_retries_exceeded() {
        let mut q = MessageQueue::new();
        let msg = QueuedMessage {
            id: "test".into(),
            recipient_id: "bob".into(),
            envelope: vec![],
            priority: MessagePriority::High,
            created_at: 0,
            retry_count: 5, // Already at max
            next_retry_at: 0,
        };

        assert!(!q.retry(msg)); // Should fail
        assert!(q.is_empty());
    }
}
