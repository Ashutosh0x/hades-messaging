//! Protocol constants for Hades Messaging.
//!
//! These values define the security parameters and operational limits
//! of the Hades protocol. Changing these requires careful security analysis.

/// Maximum number of message keys to skip (out-of-order tolerance).
/// Signal uses 2000, we use 256 as a tighter bound.
pub const MAX_SKIP: usize = 256;

/// Number of one-time prekeys to upload on registration.
pub const INITIAL_PREKEY_COUNT: usize = 100;

/// Threshold below which the server warns the client to upload more prekeys.
pub const PREKEY_LOW_WATER_MARK: usize = 10;

/// Maximum prekeys that can be uploaded in a single request.
pub const MAX_PREKEYS_PER_UPLOAD: usize = 100;

/// Signed prekey rotation interval in seconds (7 days).
pub const SIGNED_PREKEY_ROTATION_SECS: u64 = 7 * 24 * 3600;

/// Maximum age of a signed prekey before forced rotation (14 days).
pub const SIGNED_PREKEY_MAX_AGE_SECS: u64 = 14 * 24 * 3600;

/// Message padding bucket sizes in bytes.
/// Every message is padded to the nearest bucket to prevent length analysis.
pub const PADDING_BUCKETS: &[usize] = &[256, 1024, 4096, 16384];

/// Maximum sealed envelope size (256 KB).
pub const MAX_ENVELOPE_SIZE: usize = 256 * 1024;

/// Maximum attachment size (100 MB).
pub const MAX_ATTACHMENT_SIZE: usize = 100 * 1024 * 1024;

/// Offline message queue TTL in seconds (30 days).
pub const MESSAGE_QUEUE_TTL_SECS: u64 = 30 * 24 * 3600;

/// Attachment expiry in seconds (30 days).
pub const ATTACHMENT_TTL_SECS: u64 = 30 * 24 * 3600;

/// Authentication timestamp window in seconds.
/// Requests older than this are rejected (replay protection).
pub const AUTH_TIMESTAMP_WINDOW_SECS: u64 = 300;

/// WebSocket heartbeat interval in seconds.
pub const HEARTBEAT_INTERVAL_SECS: u64 = 45;

/// Idle connection timeout in seconds (5 minutes).
pub const IDLE_TIMEOUT_SECS: u64 = 300;

/// Maximum missed pong frames before disconnect.
pub const MAX_MISSED_PONGS: u8 = 3;

/// Maximum connections per edge node.
pub const MAX_CONNECTIONS_PER_NODE: usize = 200_000;

/// SPQR: Post-quantum ratchet step interval (every N messages).
pub const PQ_RATCHET_INTERVAL: u32 = 50;

/// Tor circuit rotation interval range in seconds (10-20 minutes).
pub const CIRCUIT_ROTATION_MIN_SECS: u64 = 600;
pub const CIRCUIT_ROTATION_MAX_SECS: u64 = 1200;

/// Maximum messages per Tor circuit before rotation.
pub const MAX_MESSAGES_PER_CIRCUIT: u32 = 500;

/// Cover traffic jitter range in milliseconds.
pub const TIMING_JITTER_MIN_MS: u64 = 5;
pub const TIMING_JITTER_MAX_MS: u64 = 200;

/// Proof-of-work difficulty for registration (20 bits ≈ 1s on modern phone).
pub const REGISTRATION_POW_DIFFICULTY: u8 = 20;

/// Protocol version string.
pub const PROTOCOL_VERSION: &str = "HADES/1.0";

/// HKDF info strings — domain separation for all key derivations.
pub mod hkdf_info {
    pub const PQXDH_SESSION: &[u8] = b"HadesProtocol_PQXDH";
    pub const ROOT_KEY: &[u8] = b"HadesRatchetRootKey";
    pub const CHAIN_KEY: &[u8] = b"HadesRatchetChainKey";
    pub const MESSAGE_KEY: &[u8] = b"HadesMessageKey";
    pub const SEALED_SENDER: &[u8] = b"HadesSealedSender";
    pub const FINGERPRINT: &[u8] = b"HadesFingerprint";
}
