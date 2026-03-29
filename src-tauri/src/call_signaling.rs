use crate::error::{AppError, AppResult};
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CallSignalingMessage {
    #[serde(rename = "call_offer")]
    CallOffer {
        call_id: String,
        caller_id: String,
        recipient_id: String,
        call_type: String,  // "voice" or "video"
        sdp: String,
        timestamp: u64,
    },
    #[serde(rename = "call_answer")]
    CallAnswer {
        call_id: String,
        callee_id: String,
        sdp: String,
        timestamp: u64,
    },
    #[serde(rename = "call_ice_candidate")]
    CallIceCandidate {
        call_id: String,
        sender_id: String,
        candidate: serde_json::Value,
        timestamp: u64,
    },
    #[serde(rename = "call_end")]
    CallEnd {
        call_id: String,
        sender_id: String,
        reason: String,  // "completed", "rejected", "failed", "timeout"
        duration_secs: u64,
        timestamp: u64,
    },
}

pub struct CallSignaling {
    state: Arc<RwLock<AppState>>,
}

impl CallSignaling {
    pub fn new(state: Arc<RwLock<AppState>>) -> Self {
        Self { state }
    }

    /// Send call offer to recipient
    pub async fn send_offer(
        &self,
        call_id: &str,
        recipient_id: &str,
        call_type: &str,
        sdp: &str,
    ) -> AppResult<()> {
        let s = self.state.read().await;

        let caller_id = s.messaging_keypair.as_ref()
            .ok_or(AppError::NotInitialized("No identity".into()))?
            .hades_id_hex();

        let relay = s.relay.as_ref()
            .ok_or(AppError::NotInitialized("Not connected to relay".into()))?;

        let msg = CallSignalingMessage::CallOffer {
            call_id: call_id.to_string(),
            caller_id: caller_id.clone(),
            recipient_id: recipient_id.to_string(),
            call_type: call_type.to_string(),
            sdp: sdp.to_string(),
            timestamp: unix_now(),
        };

        let serialized = serde_json::to_vec(&msg)?;

        // Send through relay (encrypted via existing message pipeline)
        relay.send(crate::websocket::RelayMessage::Send {
            recipient_id: recipient_id.to_string(),
            envelope: serialized,
            message_id: format!("call_{}", call_id),
        }).await?;

        // Store call in database
        if let Some(ref db) = s.db {
            self.store_call(db.conn(), call_id, recipient_id, &caller_id, call_type, "outgoing")?;
        }

        Ok(())
    }

    /// Send call answer
    pub async fn send_answer(
        &self,
        call_id: &str,
        caller_id: &str,
        sdp: &str,
    ) -> AppResult<()> {
        let s = self.state.read().await;

        let relay = s.relay.as_ref()
            .ok_or(AppError::NotInitialized("Not connected to relay".into()))?;

        let msg = CallSignalingMessage::CallAnswer {
            call_id: call_id.to_string(),
            callee_id: s.messaging_keypair.as_ref()
                .ok_or(AppError::NotInitialized("No identity".into()))?
                .hades_id_hex(),
            sdp: sdp.to_string(),
            timestamp: unix_now(),
        };

        let serialized = serde_json::to_vec(&msg)?;

        relay.send(crate::websocket::RelayMessage::Send {
            recipient_id: caller_id.to_string(),
            envelope: serialized,
            message_id: format!("call_{}", call_id),
        }).await?;

        Ok(())
    }

    /// Send ICE candidate
    pub async fn send_ice_candidate(
        &self,
        call_id: &str,
        recipient_id: &str,
        candidate: &serde_json::Value,
    ) -> AppResult<()> {
        let s = self.state.read().await;

        let relay = s.relay.as_ref()
            .ok_or(AppError::NotInitialized("Not connected to relay".into()))?;

        let msg = CallSignalingMessage::CallIceCandidate {
            call_id: call_id.to_string(),
            sender_id: s.messaging_keypair.as_ref()
                .ok_or(AppError::NotInitialized("No identity".into()))?
                .hades_id_hex(),
            candidate: candidate.clone(),
            timestamp: unix_now(),
        };

        let serialized = serde_json::to_vec(&msg)?;

        relay.send(crate::websocket::RelayMessage::Send {
            recipient_id: recipient_id.to_string(),
            envelope: serialized,
            message_id: format!("call_{}_ice", call_id),
        }).await?;

        Ok(())
    }

    /// End call
    pub async fn end_call(
        &self,
        call_id: &str,
        recipient_id: &str,
        reason: &str,
        duration_secs: u64,
    ) -> AppResult<()> {
        let s = self.state.read().await;

        let relay = s.relay.as_ref()
            .ok_or(AppError::NotInitialized("Not connected to relay".into()))?;

        let msg = CallSignalingMessage::CallEnd {
            call_id: call_id.to_string(),
            sender_id: s.messaging_keypair.as_ref()
                .ok_or(AppError::NotInitialized("No identity".into()))?
                .hades_id_hex(),
            reason: reason.to_string(),
            duration_secs,
            timestamp: unix_now(),
        };

        let serialized = serde_json::to_vec(&msg)?;

        relay.send(crate::websocket::RelayMessage::Send {
            recipient_id: recipient_id.to_string(),
            envelope: serialized,
            message_id: format!("call_{}_end", call_id),
        }).await?;

        // Update call status in database
        if let Some(ref db) = s.db {
            self.update_call_status(db.conn(), call_id, reason, duration_secs)?;
        }

        Ok(())
    }

    /// Process incoming signaling message
    pub async fn process_signaling_message(
        &self,
        data: &[u8],
    ) -> AppResult<CallSignalingMessage> {
        let msg: CallSignalingMessage = serde_json::from_slice(data)
            .map_err(|e| AppError::Internal(format!("Signaling parse error: {}", e)))?;

        match &msg {
            CallSignalingMessage::CallOffer { call_id, caller_id, call_type, .. } => {
                // Store incoming call in database
                let s = self.state.read().await;
                if let Some(ref db) = s.db {
                    self.store_call(db.conn(), call_id, caller_id, caller_id, call_type, "incoming")?;
                }
            }
            CallSignalingMessage::CallEnd { call_id, reason, duration_secs, .. } => {
                // Update call status
                let s = self.state.read().await;
                if let Some(ref db) = s.db {
                    self.update_call_status(db.conn(), call_id, reason, *duration_secs)?;
                }
            }
            _ => {}
        }

        Ok(msg)
    }

    fn store_call(
        &self,
        conn: &rusqlite::Connection,
        call_id: &str,
        contact_id: &str,
        other_party: &str,
        call_type: &str,
        direction: &str,
    ) -> AppResult<()> {
        conn.execute(
            r#"INSERT INTO call_history
               (id, contact_id, contact_name, call_type, direction, duration, timestamp, status)
               VALUES (?1, ?2, ?3, ?4, ?5, 0, datetime('now'), 'ringing')"#,
            rusqlite::params![call_id, contact_id, other_party, call_type, direction],
        )?;
        Ok(())
    }

    fn update_call_status(
        &self,
        conn: &rusqlite::Connection,
        call_id: &str,
        reason: &str,
        duration_secs: u64,
    ) -> AppResult<()> {
        let status = match reason {
            "completed" => "completed",
            "rejected" => "rejected",
            "failed" => "failed",
            _ => "missed",
        };

        conn.execute(
            r#"UPDATE call_history
               SET status = ?1, duration = ?2
               WHERE id = ?3"#,
            rusqlite::params![status, duration_secs, call_id],
        )?;
        Ok(())
    }
}

fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

// ── Tauri Commands ──

#[tauri::command]
pub async fn send_call_offer(
    state: tauri::State<'_, Arc<RwLock<AppState>>>,
    call_id: String,
    recipient_id: String,
    call_type: String,
    sdp: String,
) -> AppResult<()> {
    let signaling = CallSignaling::new(state.inner().clone());
    signaling.send_offer(&call_id, &recipient_id, &call_type, &sdp).await
}

#[tauri::command]
pub async fn send_call_answer(
    state: tauri::State<'_, Arc<RwLock<AppState>>>,
    call_id: String,
    caller_id: String,
    sdp: String,
) -> AppResult<()> {
    let signaling = CallSignaling::new(state.inner().clone());
    signaling.send_answer(&call_id, &caller_id, &sdp).await
}

#[tauri::command]
pub async fn send_call_ice_candidate(
    state: tauri::State<'_, Arc<RwLock<AppState>>>,
    call_id: String,
    recipient_id: String,
    candidate: serde_json::Value,
) -> AppResult<()> {
    let signaling = CallSignaling::new(state.inner().clone());
    signaling.send_ice_candidate(&call_id, &recipient_id, &candidate).await
}

// Wait, the user mentioned `delete_call_history` as part of call_signaling.rs, I should add it
#[tauri::command]
pub async fn delete_call_history(
    state: tauri::State<'_, Arc<RwLock<AppState>>>,
    call_id: String,
) -> AppResult<()> {
    let s = state.read().await;
    if let Some(ref db) = s.db {
        db.conn().execute(
            "DELETE FROM call_history WHERE id = ?1",
            rusqlite::params![call_id],
        )?;
    }
    Ok(())
}

#[tauri::command]
pub async fn get_call_history(
    state: tauri::State<'_, Arc<RwLock<AppState>>>
) -> AppResult<Vec<crate::db::call::CallHistoryRow>> {
    let s = state.read().await;
    let db = s.db.as_ref().ok_or(AppError::DatabaseLocked)?;
    crate::db::call::get_calls(db.conn())
}
