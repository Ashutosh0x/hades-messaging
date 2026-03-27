//! Wire protocol message types for Hades Messaging.
//!
//! These types mirror the protobuf schema in `proto/hades.proto`
//! but use serde for serialization, avoiding the `protoc` dependency.

use serde::{Deserialize, Serialize};

// ── Envelope ──

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SealedEnvelope {
    pub version: u32,
    pub destination_circuit: Vec<u8>,
    pub ciphertext: Vec<u8>,
    pub sender_certificate: Vec<u8>,
    pub server_timestamp: Option<u64>,
    pub padding: Vec<u8>,
}

// ── Key Exchange ──

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PrekeyBundle {
    pub identity_key: Vec<u8>,
    pub identity_dh: Vec<u8>,
    pub signed_prekey: Vec<u8>,
    pub signed_prekey_signature: Vec<u8>,
    pub signed_prekey_id: u32,
    pub one_time_prekey: Option<OneTimePrekey>,
    pub pq_prekey: Option<PqPrekey>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OneTimePrekey {
    pub key_id: u32,
    pub public_key: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PqPrekey {
    pub key_id: u32,
    pub encapsulation_key: Vec<u8>,
    pub signature: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PqxdhInitialMessage {
    pub alice_identity_key: Vec<u8>,
    pub alice_identity_dh: Vec<u8>,
    pub alice_ephemeral: Vec<u8>,
    pub bob_signed_prekey_id: u32,
    pub bob_one_time_prekey_id: Option<u32>,
    pub pq_ciphertext: Option<Vec<u8>>,
}

// ── Double Ratchet ──

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RatchetHeader {
    pub dh_public: Vec<u8>,
    pub counter: u32,
    pub previous_chain_length: u32,
    pub pq_step: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EncryptedMessage {
    pub header: RatchetHeader,
    pub ciphertext: Vec<u8>,
}

// ── Relay Messages ──

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClientHello {
    pub identity_proof: Vec<u8>,
    pub proof_of_work: Vec<u8>,
    pub protocol_version: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServerHello {
    pub accepted: bool,
    pub error_code: Option<String>,
    pub server_time: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeliverEnvelope {
    pub envelope: SealedEnvelope,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AckDelivery {
    pub message_id: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FetchPrekeys {
    pub target_identity: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PrekeyResponse {
    pub bundle: PrekeyBundle,
    pub remaining_one_time_prekeys: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UploadPrekeys {
    pub one_time_prekeys: Vec<OneTimePrekey>,
    pub pq_prekey: Option<PqPrekey>,
}

// ── Server Push ──

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerPush {
    Deliver(DeliverEnvelope),
    PrekeyLow { remaining: u32 },
    DeviceEvent { device_id: u32, event_type: String },
}
