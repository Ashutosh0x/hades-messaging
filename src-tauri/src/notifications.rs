use crate::error::AppResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationPrivacy {
    Full, SenderOnly, Hidden, Silent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPayload {
    pub title: String,
    pub body: String,
    pub conversation_id: String,
    pub message_id: String,
    pub privacy: NotificationPrivacy,
}

pub fn build_notification(
    sender_name: &str, message_preview: &str,
    conversation_id: &str, message_id: &str,
    privacy: &NotificationPrivacy,
) -> NotificationPayload {
    match privacy {
        NotificationPrivacy::Full => NotificationPayload {
            title: sender_name.to_string(),
            body: truncate(message_preview, 100),
            conversation_id: conversation_id.to_string(),
            message_id: message_id.to_string(),
            privacy: NotificationPrivacy::Full,
        },
        NotificationPrivacy::SenderOnly => NotificationPayload {
            title: "Hades".into(),
            body: format!("Message from {}", sender_name),
            conversation_id: conversation_id.to_string(),
            message_id: message_id.to_string(),
            privacy: NotificationPrivacy::SenderOnly,
        },
        NotificationPrivacy::Hidden => NotificationPayload {
            title: "Hades".into(), body: "New message".into(),
            conversation_id: conversation_id.to_string(),
            message_id: message_id.to_string(),
            privacy: NotificationPrivacy::Hidden,
        },
        NotificationPrivacy::Silent => NotificationPayload {
            title: String::new(), body: String::new(),
            conversation_id: conversation_id.to_string(),
            message_id: message_id.to_string(),
            privacy: NotificationPrivacy::Silent,
        },
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max { s.to_string() }
    else { format!("{}...", &s[..max.saturating_sub(3)]) }
}

/// Register for UnifiedPush (privacy-respecting push service)
#[tauri::command]
pub async fn register_push(endpoint: String) -> AppResult<()> {
    log::info!("Push registered: {}", &endpoint[..endpoint.len().min(40)]);
    Ok(())
}
