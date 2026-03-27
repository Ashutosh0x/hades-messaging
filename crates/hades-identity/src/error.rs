use thiserror::Error;

#[derive(Error, Debug)]
pub enum IdentityError {
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Key store error: {0}")]
    StoreError(String),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Invalid key bundle")]
    InvalidBundle,
}
