use crate::db;
use crate::message_queue;
use crate::state::AppState;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Background sync loop — retries pending messages and drains the queue.
/// Runs alongside burn_timer_loop in Tauri setup.
pub async fn sync_loop(state: Arc<RwLock<AppState>>, app_handle: tauri::AppHandle) {
    // Check every 5 seconds for messages to retry
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));

    loop {
        interval.tick().await;

        let mut s = state.write().await;

        // Skip if vault is locked
        if !s.vault_unlocked {
            continue;
        }

        // 1. Drain ready messages from the queue and attempt to send
        let now = message_queue::now_ms();
        let mut retry_later = Vec::new();

        while let Some(msg) = s.message_queue.dequeue_ready(now) {
            // Attempt to send via relay
            let send_ok = if let Some(ref relay) = s.relay {
                use crate::websocket::RelayMessage;
                let relay_msg = RelayMessage::Send {
                    recipient_id: msg.recipient_id.clone(),
                    envelope: msg.envelope.clone(),
                    message_id: msg.id.clone(),
                };
                relay.send(relay_msg).await.is_ok()
            } else {
                false
            };

            if send_ok {
                log::info!("Background sync: sent message {}", msg.id);
                // Update status in DB
                if let Some(ref database) = s.db {
                    let _ = db::messages::update_message_status(database.conn(), &msg.id, "sent");
                }
                // Notify frontend
                let _ = app_handle.emit(
                    "message-status",
                    serde_json::json!({"id": msg.id, "status": "sent"}),
                );
            } else {
                // Re-queue for retry
                retry_later.push(msg);
            }
        }

        // Re-enqueue failed messages with backoff
        for msg in retry_later {
            let msg_id = msg.id.clone();
            if !s.message_queue.retry(msg) {
                // Max retries exceeded — mark as permanently failed
                log::warn!("Message {} permanently failed after max retries", msg_id);
                if let Some(ref database) = s.db {
                    let _ =
                        db::messages::update_message_status(database.conn(), &msg_id, "failed");
                }
                let _ = app_handle.emit(
                    "message-status",
                    serde_json::json!({"id": msg_id, "status": "failed"}),
                );
            }
        }

        // 2. Check for stuck "sending" messages in DB (older than 30s) and enqueue them
        if let Some(ref database) = s.db {
            if let Ok(pending) = db::messages::get_pending_messages(database.conn()) {
                for msg in pending {
                    // Only re-enqueue if not already in the queue
                    let queued = message_queue::QueuedMessage {
                        id: msg.id.clone(),
                        recipient_id: msg.conversation_id.clone(),
                        envelope: msg.content_encrypted.clone(),
                        priority: message_queue::MessagePriority::High,
                        created_at: message_queue::now_ms(),
                        retry_count: 0,
                        next_retry_at: 0,
                    };
                    s.message_queue.enqueue(queued);
                }
            }
        }

        let queue_len = s.message_queue.len();
        if queue_len > 0 {
            log::debug!("Background sync: {} messages in queue", queue_len);
        }
    }
}
