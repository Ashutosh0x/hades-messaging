use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Crypto error: {0}")]
    Crypto(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Session error: {0}")]
    Session(String),

    #[error("Identity error: {0}")]
    Identity(String),

    #[error("Not initialized: {0}")]
    NotInitialized(String),

    #[error("Database locked")]
    DatabaseLocked,

    #[error("Invalid passphrase")]
    InvalidPassphrase,

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("WebSocket error: {0}")]
    WebSocket(String),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("{0}")]
    Internal(String),
}

// Tauri requires Serialize for command return errors
impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
