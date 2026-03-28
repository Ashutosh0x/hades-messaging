use crate::error::AppResult;
use crate::state::AppState;
use crate::websocket::RelayMessage;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadReceipt {
    pub message_ids: Vec<String>,
    pub conversation_id: String,
    pub read_at: String,
}

pub async fn send_read_receipts(
    state: &Arc<RwLock<AppState>>,
    conversation_id: &str,
    message_ids: &[String],
) -> AppResult<()> {
    let s = state.read().await;

    if let Some(ref db) = s.db {
        for id in message_ids {
            crate::db::messages::update_message_status(db.conn(), id, "read")?;
        }
    }

    if let Some(ref relay) = s.relay {
        let receipt = ReadReceipt {
            message_ids: message_ids.to_vec(),
            conversation_id: conversation_id.to_string(),
            read_at: unix_now(),
        };
        let receipt_json = serde_json::to_vec(&receipt).unwrap_or_default();

        relay.send(RelayMessage::Send {
            recipient_id: conversation_id.to_string(),
            envelope: receipt_json,
            message_id: String::new(),
        }).await?;
    }

    Ok(())
}

fn unix_now() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .to_string()
}
