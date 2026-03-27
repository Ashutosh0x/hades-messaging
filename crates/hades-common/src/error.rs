//! Error types for the Hades protocol.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum HadesError {
    // ── Cryptographic errors ──
    #[error("Key exchange failed: {0}")]
    KeyExchange(String),

    #[error("Encryption failed")]
    EncryptionFailed,

    #[error("Decryption failed — invalid ciphertext or wrong key")]
    DecryptionFailed,

    #[error("Invalid signature")]
    InvalidSignature,

    #[error("Invalid public key")]
    InvalidPublicKey,

    // ── Ratchet errors ──
    #[error("No sending chain established")]
    NoSendingChain,

    #[error("No receiving chain established")]
    NoReceivingChain,

    #[error("Too many skipped messages (>{0})")]
    TooManySkippedMessages(usize),

    #[error("No remote DH key available")]
    NoRemoteKey,

    // ── Session errors ──
    #[error("Session not found for {0}")]
    SessionNotFound(String),

    #[error("No active session for device")]
    NoActiveSession,

    #[error("Prekey bundle verification failed: {0}")]
    PrekeyVerificationFailed(String),

    // ── Identity errors ──
    #[error("Identity not found: {0}")]
    IdentityNotFound(String),

    #[error("Device not authorized: {0}")]
    DeviceNotAuthorized(u32),

    // ── Transport errors ──
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Tor bootstrap failed: {0}")]
    TorBootstrapFailed(String),

    #[error("Circuit build failed")]
    CircuitBuildFailed,

    // ── Rate limiting ──
    #[error("Rate limited — retry after {0}s")]
    RateLimited(u32),

    // ── Storage ──
    #[error("Storage error: {0}")]
    Storage(String),

    // ── Protocol ──
    #[error("Protocol version mismatch: expected {expected}, got {got}")]
    VersionMismatch { expected: String, got: String },

    #[error("Envelope too large: {size} bytes (max {max})")]
    EnvelopeTooLarge { size: usize, max: usize },

    #[error("Invalid proof of work")]
    InvalidProofOfWork,

    // ── Group ──
    #[error("Group epoch mismatch: expected {expected}, got {got}")]
    EpochMismatch { expected: u64, got: u64 },

    // ── Key Transparency ──
    #[error("Key transparency proof verification failed")]
    KtProofFailed,

    #[error("Key mismatch detected — possible MITM")]
    KeyMismatch,
}

/// Server-facing error codes — generic enough to prevent info leakage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum ErrorCode {
    InvalidSignature = 4001,
    ExpiredTimestamp = 4002,
    UnknownIdentity = 4003,
    NotFound = 4004,
    RateLimited = 4005,
    InvalidPrekeyBundle = 4006,
    EpochMismatch = 4007,
    DeviceNotAuthorized = 4008,
    EnvelopeTooLarge = 4009,
    MissingHeader = 4010,
    InvalidProofOfWork = 4011,
    RegistrationConflict = 4012,
    InternalError = 5001,
    ServiceUnavailable = 5002,
    UpstreamTimeout = 5003,
}

impl ErrorCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::InvalidSignature => "E4001",
            Self::ExpiredTimestamp => "E4002",
            Self::UnknownIdentity => "E4003",
            Self::NotFound => "E4004",
            Self::RateLimited => "E4005",
            Self::InvalidPrekeyBundle => "E4006",
            Self::EpochMismatch => "E4007",
            Self::DeviceNotAuthorized => "E4008",
            Self::EnvelopeTooLarge => "E4009",
            Self::MissingHeader => "E4010",
            Self::InvalidProofOfWork => "E4011",
            Self::RegistrationConflict => "E4012",
            Self::InternalError => "E5001",
            Self::ServiceUnavailable => "E5002",
            Self::UpstreamTimeout => "E5003",
        }
    }
}
