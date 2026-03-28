use crate::db;
use crate::state::AppState;
use std::sync::Arc;
use tauri::Emitter;
use tokio::sync::RwLock;

/// Background task that deletes expired burn-after messages
pub async fn burn_timer_loop(state: Arc<RwLock<AppState>>, app_handle: tauri::AppHandle) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));

    loop {
        interval.tick().await;

        let s = state.read().await;
        if let Some(ref database) = s.db {
            match db::messages::delete_expired_burn_messages(database.conn()) {
                Ok(count) if count > 0 => {
                    log::info!("Burned {} expired messages", count);
                    let _ = app_handle.emit("messages-burned", serde_json::json!({"count": count}));
                }
                Err(e) => log::error!("Burn timer error: {}", e),
                _ => {}
            }
        }
    }
}
