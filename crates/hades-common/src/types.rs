//! Core types for the Hades protocol.
//!
//! All identifiers are opaque, cryptographically derived, and reveal nothing
//! about creation time or ordering to the server.

use serde::{Deserialize, Serialize};
use std::fmt;
use zeroize::Zeroize;

/// A Hades user identity — BLAKE3 hash of the Ed25519 public key.
/// This is the only identifier the server uses to reference users.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HadesId([u8; 32]);

impl HadesId {
    /// Derive a HadesId from an Ed25519 public key.
    pub fn from_identity_key(public_key: &[u8; 32]) -> Self {
        let hash = blake3::hash(public_key);
        Self(*hash.as_bytes())
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }
}

impl fmt::Debug for HadesId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HadesId({}..)", &self.to_hex()[..8])
    }
}

impl fmt::Display for HadesId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

/// Circuit identifier for transport-layer routing.
/// Opaque to the server — used to route messages without revealing user identity.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CircuitId([u8; 32]);

impl CircuitId {
    /// Generate a random circuit ID.
    pub fn generate() -> Self {
        let mut bytes = [0u8; 32];
        getrandom::getrandom(&mut bytes).expect("Failed to generate random bytes");
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn to_base64(&self) -> String {
        use base64::Engine;
        base64::engine::general_purpose::STANDARD.encode(self.0)
    }
}

impl fmt::Debug for CircuitId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Circuit({}..)", hex::encode(&self.0[..4]))
    }
}

/// Unique message identifier — ULID-based, time-sortable but opaque.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct MessageId(ulid::Ulid);

impl MessageId {
    pub fn new() -> Self {
        Self(ulid::Ulid::new())
    }

    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl Default for MessageId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for MessageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Msg({})", self.0)
    }
}

/// Group identifier — random ULID, client-generated.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GroupId(ulid::Ulid);

impl GroupId {
    pub fn new() -> Self {
        Self(ulid::Ulid::new())
    }
}

impl Default for GroupId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for GroupId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Group({})", self.0)
    }
}

/// Device identifier within a user's device set.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct DeviceId(pub u32);

impl DeviceId {
    pub const PRIMARY: Self = Self(1);

    pub fn new(id: u32) -> Self {
        Self(id)
    }
}

/// Protocol version for wire format compatibility.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolVersion {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
}

impl ProtocolVersion {
    pub const CURRENT: Self = Self {
        major: 1,
        minor: 0,
        patch: 0,
    };
}

impl fmt::Display for ProtocolVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "v{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Sealed envelope — the only thing the server sees transit through it.
/// Server cannot read content, sender, or anything beyond routing info.
#[derive(Clone, Serialize, Deserialize)]
pub struct SealedEnvelope {
    /// Protocol version
    pub version: ProtocolVersion,
    /// Destination circuit for routing
    pub destination: CircuitId,
    /// Encrypted payload — opaque to server
    pub ciphertext: Vec<u8>,
    /// Encrypted sender certificate — only recipient can decrypt
    pub sender_certificate: Vec<u8>,
    /// Server sets this on receipt (day precision only)
    pub server_timestamp: Option<u64>,
    /// Random padding to fixed bucket size
    pub padding: Vec<u8>,
}

impl Zeroize for SealedEnvelope {
    fn zeroize(&mut self) {
        self.ciphertext.zeroize();
        self.sender_certificate.zeroize();
        self.padding.zeroize();
    }
}

impl fmt::Debug for SealedEnvelope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SealedEnvelope")
            .field("version", &self.version)
            .field("destination", &self.destination)
            .field("ciphertext_len", &self.ciphertext.len())
            .finish()
    }
}
