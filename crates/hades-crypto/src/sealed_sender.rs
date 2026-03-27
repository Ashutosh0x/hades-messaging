//! Sealed Sender — hide sender identity from the relay server.
//!
//! The relay server routes messages using circuit IDs but should never learn
//! who is sending a message. Sealed sender encrypts the sender's identity
//! inside the envelope so only the recipient can learn who sent it.
//!
//! ## Protocol
//!
//! 1. Sender generates ephemeral X25519 key pair
//! 2. Sender performs DH with recipient's identity public key
//! 3. Derive encryption key via HKDF
//! 4. Encrypt (sender_identity || message) with ChaCha20-Poly1305
//! 5. Attach ephemeral public key to the sealed envelope
//! 6. Recipient uses their identity secret to DH with ephemeral public key
//! 7. Derive same key, decrypt, learn sender identity

use ed25519_dalek::VerifyingKey;
use x25519_dalek::{EphemeralSecret, PublicKey as X25519Public, StaticSecret};
use zeroize::Zeroize;

use crate::aead;
use crate::error::CryptoError;
use crate::kdf;
use hades_common::constants::hkdf_info;

/// A sealed sender certificate — the sender's identity encrypted to the recipient.
#[derive(Clone)]
pub struct SealedSenderCertificate {
    /// Ephemeral X25519 public key (visible to server, but meaningless without recipient's key)
    pub ephemeral_public: [u8; 32],
    /// Encrypted sender identity + content
    pub encrypted_payload: Vec<u8>,
}

/// Content inside the sealed envelope (after decryption).
pub struct UnsealedContent {
    /// Sender's Ed25519 identity public key
    pub sender_identity: [u8; 32],
    /// The actual message ciphertext (still E2EE encrypted via Double Ratchet)
    pub message: Vec<u8>,
}

/// Create a sealed sender envelope.
///
/// Encrypts the sender's identity key and message payload so that only
/// the recipient (who holds `recipient_identity_dh_secret`) can learn
/// who sent the message.
///
/// # Arguments
/// - `sender_identity_key`: Sender's Ed25519 public key (32 bytes)
/// - `recipient_dh_public`: Recipient's X25519 identity public key
/// - `message`: Already-encrypted message payload (Double Ratchet ciphertext)
pub fn seal(
    sender_identity_key: &[u8; 32],
    recipient_dh_public: &X25519Public,
    message: &[u8],
) -> SealedSenderCertificate {
    // 1. Generate ephemeral X25519 key pair
    let ephemeral_secret = EphemeralSecret::random_from_rng(rand::thread_rng());
    let ephemeral_public = X25519Public::from(&ephemeral_secret);

    // 2. DH with recipient's identity key
    let shared_secret = ephemeral_secret.diffie_hellman(recipient_dh_public);

    // 3. Derive encryption key via HKDF
    let mut encryption_key = [0u8; 32];
    let hk = hkdf::Hkdf::<sha2::Sha256>::new(
        Some(hkdf_info::SEALED_SENDER),
        shared_secret.as_bytes(),
    );
    hk.expand(b"HadesSealedSenderKey", &mut encryption_key)
        .expect("HKDF expand should not fail for 32-byte output");

    // 4. Build plaintext: sender_identity (32 bytes) || message
    let mut plaintext = Vec::with_capacity(32 + message.len());
    plaintext.extend_from_slice(sender_identity_key);
    plaintext.extend_from_slice(message);

    // 5. Encrypt with ChaCha20-Poly1305
    let nonce = [0u8; 12]; // Safe: key is ephemeral, used exactly once
    let aad = ephemeral_public.as_bytes(); // Bind to the ephemeral key
    let encrypted_payload = aead::encrypt(&encryption_key, &nonce, &plaintext, aad)
        .expect("AEAD encryption should not fail with valid key");

    // Zero sensitive material
    plaintext.fill(0);
    encryption_key.zeroize();

    SealedSenderCertificate {
        ephemeral_public: *ephemeral_public.as_bytes(),
        encrypted_payload,
    }
}

/// Open a sealed sender envelope.
///
/// Decrypts the sender's identity and message payload using the
/// recipient's X25519 identity secret key.
///
/// # Arguments
/// - `recipient_dh_secret`: Recipient's X25519 identity secret key
/// - `certificate`: The sealed sender certificate to open
pub fn unseal(
    recipient_dh_secret: &StaticSecret,
    certificate: &SealedSenderCertificate,
) -> Result<UnsealedContent, CryptoError> {
    // 1. Reconstruct the DH shared secret
    let ephemeral_public = X25519Public::from(certificate.ephemeral_public);
    let shared_secret = recipient_dh_secret.diffie_hellman(&ephemeral_public);

    // 2. Derive the same encryption key
    let mut encryption_key = [0u8; 32];
    let hk = hkdf::Hkdf::<sha2::Sha256>::new(
        Some(hkdf_info::SEALED_SENDER),
        shared_secret.as_bytes(),
    );
    hk.expand(b"HadesSealedSenderKey", &mut encryption_key)
        .expect("HKDF expand should not fail for 32-byte output");

    // 3. Decrypt
    let nonce = [0u8; 12];
    let aad = &certificate.ephemeral_public;
    let plaintext = aead::decrypt(&encryption_key, &nonce, &certificate.encrypted_payload, aad)?;

    encryption_key.zeroize();

    // 4. Parse: first 32 bytes = sender identity, rest = message
    if plaintext.len() < 32 {
        return Err(CryptoError::InvalidSenderCertificate);
    }

    let mut sender_identity = [0u8; 32];
    sender_identity.copy_from_slice(&plaintext[..32]);
    let message = plaintext[32..].to_vec();

    Ok(UnsealedContent {
        sender_identity,
        message,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use x25519_dalek::StaticSecret;

    #[test]
    fn test_sealed_sender_roundtrip() {
        // Sender identity
        let sender_id = [0x42u8; 32];

        // Recipient key pair
        let recipient_secret = StaticSecret::random_from_rng(rand::thread_rng());
        let recipient_public = X25519Public::from(&recipient_secret);

        let message = b"Hello from sealed sender!";

        // Seal
        let certificate = seal(&sender_id, &recipient_public, message);

        // Unseal
        let unsealed = unseal(&recipient_secret, &certificate).unwrap();

        assert_eq!(unsealed.sender_identity, sender_id);
        assert_eq!(unsealed.message, message);
    }

    #[test]
    fn test_sealed_sender_wrong_recipient() {
        let sender_id = [0x42u8; 32];

        let recipient_secret = StaticSecret::random_from_rng(rand::thread_rng());
        let recipient_public = X25519Public::from(&recipient_secret);

        let wrong_recipient_secret = StaticSecret::random_from_rng(rand::thread_rng());

        let certificate = seal(&sender_id, &recipient_public, b"secret");

        // Wrong recipient should fail decryption
        let result = unseal(&wrong_recipient_secret, &certificate);
        assert!(result.is_err());
    }

    #[test]
    fn test_sealed_sender_tampered_payload() {
        let sender_id = [0x42u8; 32];
        let recipient_secret = StaticSecret::random_from_rng(rand::thread_rng());
        let recipient_public = X25519Public::from(&recipient_secret);

        let mut certificate = seal(&sender_id, &recipient_public, b"secret");

        // Tamper with encrypted payload
        if let Some(byte) = certificate.encrypted_payload.last_mut() {
            *byte ^= 0xFF;
        }

        let result = unseal(&recipient_secret, &certificate);
        assert!(result.is_err());
    }

    #[test]
    fn test_sealed_sender_different_messages() {
        let sender_id = [0x42u8; 32];
        let recipient_secret = StaticSecret::random_from_rng(rand::thread_rng());
        let recipient_public = X25519Public::from(&recipient_secret);

        let cert1 = seal(&sender_id, &recipient_public, b"message 1");
        let cert2 = seal(&sender_id, &recipient_public, b"message 2");

        // Different ephemeral keys each time
        assert_ne!(cert1.ephemeral_public, cert2.ephemeral_public);

        // Both should decrypt correctly
        let u1 = unseal(&recipient_secret, &cert1).unwrap();
        let u2 = unseal(&recipient_secret, &cert2).unwrap();
        assert_eq!(u1.message, b"message 1");
        assert_eq!(u2.message, b"message 2");
    }
}