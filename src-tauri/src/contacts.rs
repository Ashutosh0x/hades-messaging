//! Contact Identity Exchange Protocol
//!
//! Handles the creation, sharing, and verification of contact identity bundles.
//! Bundles are self-signed with Ed25519, enabling tamper-proof exchange via
//! hades:// links, QR codes, or direct key exchange inside E2EE channels.

use crate::error::{AppError, AppResult};
use base64::Engine;
use serde::{Deserialize, Serialize};

/// A contact identity bundle — everything needed to establish an E2EE session.
///
/// Contains both messaging identity and optional wallet addresses,
/// all signed by the owner's Ed25519 key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactIdentityBundle {
    /// The contact's Hades ID (blake3 of Ed25519 pubkey)
    pub hades_id: String,

    /// Ed25519 public key for verification (hex)
    pub ed25519_public_key: String,

    /// X25519 public key for key exchange (hex)
    pub x25519_public_key: String,

    /// Optional display name
    pub display_name: Option<String>,

    /// Optional wallet addresses (chain → address)
    /// Shared INSIDE the E2EE channel, never visible to relay
    pub wallet_addresses: Option<std::collections::HashMap<String, String>>,

    /// Signature proving this bundle is self-signed (hex)
    pub self_signature: String,
}

impl ContactIdentityBundle {
    /// Create a shareable identity bundle from our keypair
    pub fn from_keypair(
        keypair: &hades_identity::seed::MessagingKeypair,
        display_name: Option<&str>,
        wallet_addresses: Option<std::collections::HashMap<String, String>>,
    ) -> Self {
        // Sign the bundle content for tamper-proofing
        let mut sign_data = Vec::new();
        sign_data.extend_from_slice(keypair.ed25519_public.as_bytes());
        sign_data.extend_from_slice(keypair.x25519_public.as_bytes());
        if let Some(ref name) = display_name {
            sign_data.extend_from_slice(name.as_bytes());
        }

        let signature = keypair.sign(&sign_data);

        Self {
            hades_id: keypair.hades_id_hex(),
            ed25519_public_key: hex::encode(keypair.ed25519_public.as_bytes()),
            x25519_public_key: hex::encode(keypair.x25519_public.as_bytes()),
            display_name: display_name.map(|s| s.to_string()),
            wallet_addresses,
            self_signature: hex::encode(&signature),
        }
    }

    /// Verify this bundle is legitimately self-signed
    pub fn verify(&self) -> bool {
        let pubkey_bytes = match hex::decode(&self.ed25519_public_key) {
            Ok(b) if b.len() == 32 => b,
            _ => return false,
        };

        let mut pk_arr = [0u8; 32];
        pk_arr.copy_from_slice(&pubkey_bytes);

        let verifying_key = match ed25519_dalek::VerifyingKey::from_bytes(&pk_arr) {
            Ok(k) => k,
            Err(_) => return false,
        };

        let x25519_bytes = match hex::decode(&self.x25519_public_key) {
            Ok(b) => b,
            Err(_) => return false,
        };

        let sig_bytes = match hex::decode(&self.self_signature) {
            Ok(b) if b.len() == 64 => b,
            _ => return false,
        };

        let signature = match ed25519_dalek::Signature::from_slice(&sig_bytes) {
            Ok(s) => s,
            Err(_) => return false,
        };

        let mut sign_data = Vec::new();
        sign_data.extend_from_slice(&pubkey_bytes);
        sign_data.extend_from_slice(&x25519_bytes);
        if let Some(ref name) = self.display_name {
            sign_data.extend_from_slice(name.as_bytes());
        }

        use ed25519_dalek::Verifier;
        verifying_key.verify(&sign_data, &signature).is_ok()
    }

    /// Verify that hades_id matches the public key
    pub fn verify_hades_id(&self) -> bool {
        if let Ok(pubkey_bytes) = hex::decode(&self.ed25519_public_key) {
            let expected_id = blake3::hash(&pubkey_bytes);
            hex::encode(expected_id.as_bytes()) == self.hades_id
        } else {
            false
        }
    }

    /// Encode as a shareable hades:// link
    pub fn to_link(&self) -> String {
        let json = serde_json::to_string(self).unwrap_or_default();
        let encoded = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(json.as_bytes());
        format!("hades://contact/{}", encoded)
    }

    /// Parse from a shareable hades:// link
    pub fn from_link(link: &str) -> AppResult<Self> {
        let data = link
            .trim_start_matches("hades://contact/")
            .trim_start_matches("hades://add/");

        let json_bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(data)
            .map_err(|_| AppError::Identity("Invalid contact link encoding".into()))?;

        let bundle: Self = serde_json::from_slice(&json_bytes)
            .map_err(|_| AppError::Identity("Invalid contact bundle data".into()))?;

        // Verify bundle integrity
        if !bundle.verify() {
            return Err(AppError::Identity(
                "Contact bundle signature invalid".into(),
            ));
        }

        if !bundle.verify_hades_id() {
            return Err(AppError::Identity(
                "Hades ID does not match public key".into(),
            ));
        }

        Ok(bundle)
    }

    /// Encode as QR code content (JSON)
    pub fn to_qr_data(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    /// Parse from QR code scan data
    pub fn from_qr_data(data: &str) -> AppResult<Self> {
        // Try as JSON first
        if let Ok(bundle) = serde_json::from_str::<Self>(data) {
            if bundle.verify() && bundle.verify_hades_id() {
                return Ok(bundle);
            }
        }

        // Try as hades:// link
        if data.starts_with("hades://") {
            return Self::from_link(data);
        }

        Err(AppError::Identity("Unrecognized QR code format".into()))
    }
}

/// Wallet address exchange — sent INSIDE the E2EE channel
/// after session is established with PQXDH.
///
/// Addresses are cryptographically bound to the sender's identity
/// via Ed25519 signature, preventing man-in-the-middle address substitution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletAddressExchange {
    /// Message type identifier
    pub msg_type: String,

    /// Chain → address map
    pub addresses: std::collections::HashMap<String, String>,

    /// Ed25519 signature of sorted(chain:address) pairs
    pub signature: String,
}

impl WalletAddressExchange {
    /// Create a signed wallet address exchange message
    pub fn create(
        keypair: &hades_identity::seed::MessagingKeypair,
        addresses: std::collections::HashMap<String, String>,
    ) -> Self {
        let mut sign_data = Vec::new();
        let mut sorted_keys: Vec<&String> = addresses.keys().collect();
        sorted_keys.sort();
        for key in sorted_keys {
            sign_data.extend_from_slice(key.as_bytes());
            sign_data.push(b':');
            sign_data.extend_from_slice(addresses[key].as_bytes());
            sign_data.push(b'\n');
        }

        let signature = keypair.sign(&sign_data);

        Self {
            msg_type: "wallet_address_share".to_string(),
            addresses,
            signature: hex::encode(&signature),
        }
    }

    /// Verify the address exchange was signed by the claimed identity
    pub fn verify(&self, ed25519_public: &ed25519_dalek::VerifyingKey) -> bool {
        let mut sign_data = Vec::new();
        let mut sorted_keys: Vec<&String> = self.addresses.keys().collect();
        sorted_keys.sort();
        for key in sorted_keys {
            sign_data.extend_from_slice(key.as_bytes());
            sign_data.push(b':');
            sign_data.extend_from_slice(self.addresses[key].as_bytes());
            sign_data.push(b'\n');
        }

        let sig_bytes = match hex::decode(&self.signature) {
            Ok(b) if b.len() == 64 => b,
            _ => return false,
        };

        let sig = match ed25519_dalek::Signature::from_slice(&sig_bytes) {
            Ok(s) => s,
            Err(_) => return false,
        };

        use ed25519_dalek::Verifier;
        ed25519_public.verify(&sign_data, &sig).is_ok()
    }
}
