use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum WalletError {
    #[error("Invalid mnemonic: {0}")]
    InvalidMnemonic(String),

    #[error("Key derivation failed: {0}")]
    DerivationFailed(String),

    #[error("Unsupported chain: {0}")]
    UnsupportedChain(String),

    #[error("Insufficient balance: have {have}, need {need}")]
    InsufficientBalance { have: String, need: String },

    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    #[error("Signing failed: {0}")]
    SigningFailed(String),

    #[error("Broadcast failed: {0}")]
    BroadcastFailed(String),

    #[error("RPC error: {0}")]
    RpcError(String),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("{0}")]
    Internal(String),
}

impl Serialize for WalletError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
