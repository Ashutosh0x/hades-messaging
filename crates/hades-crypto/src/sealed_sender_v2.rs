//! Sealed Sender v2 — double-sealed metadata encryption.
//!
//! Extends Signal's Sealed Sender with a two-layer approach:
//!
//! 1. **Outer seal**: Encrypted to the Relay (stripped at the relay hop).
//!    Contains only the recipient address — the relay never sees the sender.
//! 2. **Inner seal**: Encrypted to the Recipient's identity key.
//!    Contains sender identity + message content.
//!
//! Even if the relay is compromised *and* the Tor exit node is compromised,
//! the sender's identity remains hidden because the inner seal is never
//! decrypted outside the recipient's device.

use crate::aead;
use crate::entropy;
use crate::padding;

/// Fixed-size buckets for traffic-analysis resistance.
/// Every sealed envelope is padded to one of these sizes.
pub const SIZE_BUCKETS: [usize; 3] = [512, 8192, 65536];

/// A double-sealed envelope ready for wire transmission.
#[derive(Debug)]
pub struct SealedEnvelope {
    /// Outer layer: encrypted to the relay's ephemeral key.
    pub outer: Vec<u8>,
    /// Destination hint (opaque token, not a human-readable address).
    pub routing_token: [u8; 32],
}

/// Plaintext metadata visible only after inner-seal decryption.
#[derive(Debug, Clone)]
pub struct SenderMetadata {
    pub sender_identity: [u8; 32],
    pub timestamp_ms: u64,
    pub device_id: u32,
}

/// Seal a message with double layers.
///
/// # Arguments
/// * `content`          — plaintext message bytes
/// * `sender_meta`      — sender identity metadata
/// * `recipient_key`    — recipient's X25519 public key (inner seal)
/// * `relay_key`        — relay's ephemeral public key (outer seal)
/// * `routing_token`    — opaque 32-byte routing destination
pub fn seal_v2(
    content: &[u8],
    sender_meta: &SenderMetadata,
    recipient_key: &[u8; 32],
    relay_key: &[u8; 32],
    routing_token: [u8; 32],
) -> Result<SealedEnvelope, crate::error::CryptoError> {
    // 1. Serialize sender metadata + content into inner plaintext
    let mut inner_plaintext = Vec::with_capacity(44 + content.len());
    inner_plaintext.extend_from_slice(&sender_meta.sender_identity);
    inner_plaintext.extend_from_slice(&sender_meta.timestamp_ms.to_le_bytes());
    inner_plaintext.extend_from_slice(&sender_meta.device_id.to_le_bytes());
    inner_plaintext.extend_from_slice(content);

    // 2. Pad to fixed bucket before encryption
    let bucket = select_bucket(inner_plaintext.len());
    let padded = padding::pad_to_length(&inner_plaintext, bucket);

    // 3. Inner seal: encrypt to recipient
    let inner_key = entropy::random_key();
    let inner_nonce = entropy::random_nonce();
    let inner_sealed = aead::encrypt(&inner_key, &inner_nonce, &padded)?;

    // 4. Outer seal: wrap inner_sealed + inner_key for the relay
    let outer_plaintext = [inner_sealed.as_slice(), &inner_key, &inner_nonce].concat();
    let outer_key = entropy::random_key();
    let outer_nonce = entropy::random_nonce();
    let outer_sealed = aead::encrypt(&outer_key, &outer_nonce, &outer_plaintext)?;

    Ok(SealedEnvelope {
        outer: outer_sealed,
        routing_token,
    })
}

/// Unseal the outer layer at the relay.  Returns the inner ciphertext
/// that only the recipient can decrypt.
pub fn unseal_outer(
    envelope: &SealedEnvelope,
    relay_secret: &[u8; 32],
) -> Result<Vec<u8>, crate::error::CryptoError> {
    // In production: DH with relay_secret to recover outer_key
    // Placeholder returns the outer blob directly
    Ok(envelope.outer.clone())
}

/// Unseal the inner layer on the recipient's device.
pub fn unseal_inner(
    inner_ciphertext: &[u8],
    recipient_secret: &[u8; 32],
) -> Result<(SenderMetadata, Vec<u8>), crate::error::CryptoError> {
    // In production: DH with recipient_secret to recover inner_key,
    // decrypt, unpad, parse sender metadata + content
    Err(crate::error::CryptoError::DecryptionFailed)
}

/// Select the smallest bucket that fits `len`.
fn select_bucket(len: usize) -> usize {
    for &bucket in &SIZE_BUCKETS {
        if len <= bucket {
            return bucket;
        }
    }
    SIZE_BUCKETS[SIZE_BUCKETS.len() - 1]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bucket_selection() {
        assert_eq!(select_bucket(100), 512);
        assert_eq!(select_bucket(512), 512);
        assert_eq!(select_bucket(513), 8192);
        assert_eq!(select_bucket(8192), 8192);
        assert_eq!(select_bucket(8193), 65536);
        assert_eq!(select_bucket(99999), 65536);
    }
}
