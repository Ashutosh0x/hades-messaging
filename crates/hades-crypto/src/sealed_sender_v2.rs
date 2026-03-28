//! Sealed Sender v2: double-sealed envelope
//!
//! Outer layer: ephemeral DH with relay (hides sender from relay)
//! Inner layer: ephemeral DH with recipient (hides sender from everyone but recipient)

use crate::aead;
use crate::error::CryptoError;
use x25519_dalek::{PublicKey, StaticSecret};
use zeroize::Zeroize;

const BUCKET_SMALL: usize = 512;
const BUCKET_MEDIUM: usize = 8192;
const BUCKET_LARGE: usize = 65536;

fn select_bucket(len: usize) -> usize {
    if len <= BUCKET_SMALL {
        BUCKET_SMALL
    } else if len <= BUCKET_MEDIUM {
        BUCKET_MEDIUM
    } else {
        BUCKET_LARGE
    }
}

/// Seal a message with double-sealed sender.
///
/// - `plaintext`: the message payload
/// - `sender_identity`: 32-byte sender identity (hidden from relay)
/// - `recipient_identity_pub`: recipient's X25519 public key
/// - `relay_identity_pub`: relay server's X25519 public key
pub fn seal_v2(
    plaintext: &[u8],
    sender_identity: &[u8; 32],
    recipient_identity_pub: &PublicKey,
    relay_identity_pub: &PublicKey,
) -> Result<Vec<u8>, CryptoError> {
    // --- Inner seal: for recipient ---
    let inner_ek_secret = StaticSecret::random_from_rng(rand::thread_rng());
    let inner_ek_pub = PublicKey::from(&inner_ek_secret);

    let inner_shared = inner_ek_secret.diffie_hellman(recipient_identity_pub);
    let mut inner_key = [0u8; 32];
    let hk = hkdf::Hkdf::<sha2::Sha256>::new(None, inner_shared.as_bytes());
    hk.expand(b"HadesSealedSenderV2Inner", &mut inner_key)
        .map_err(|_| CryptoError::KdfError)?;

    // Inner plaintext = sender_identity (32) + plaintext
    let mut inner_plain = Vec::with_capacity(32 + plaintext.len());
    inner_plain.extend_from_slice(sender_identity);
    inner_plain.extend_from_slice(plaintext);

    let mut inner_nonce = [0u8; 12];
    getrandom::getrandom(&mut inner_nonce).map_err(|_| CryptoError::Encryption)?;

    let inner_ct = aead::encrypt(&inner_key, &inner_nonce, &inner_plain, b"")?;

    // Inner envelope = inner_ek_pub (32) + inner_nonce (12) + inner_ct
    let mut inner_envelope = Vec::with_capacity(32 + 12 + inner_ct.len());
    inner_envelope.extend_from_slice(inner_ek_pub.as_bytes());
    inner_envelope.extend_from_slice(&inner_nonce);
    inner_envelope.extend_from_slice(&inner_ct);

    // --- Pad to bucket ---
    let bucket = select_bucket(inner_envelope.len());
    let mut padded = vec![0u8; bucket];
    let len_bytes = (inner_envelope.len() as u32).to_be_bytes();
    padded[..4].copy_from_slice(&len_bytes);
    padded[4..4 + inner_envelope.len()].copy_from_slice(&inner_envelope);
    getrandom::getrandom(&mut padded[4 + inner_envelope.len()..])
        .map_err(|_| CryptoError::Encryption)?;

    // --- Outer seal: for relay ---
    let outer_ek_secret = StaticSecret::random_from_rng(rand::thread_rng());
    let outer_ek_pub = PublicKey::from(&outer_ek_secret);

    let outer_shared = outer_ek_secret.diffie_hellman(relay_identity_pub);
    let mut outer_key = [0u8; 32];
    let hk2 = hkdf::Hkdf::<sha2::Sha256>::new(None, outer_shared.as_bytes());
    hk2.expand(b"HadesSealedSenderV2Outer", &mut outer_key)
        .map_err(|_| CryptoError::KdfError)?;

    let mut outer_nonce = [0u8; 12];
    getrandom::getrandom(&mut outer_nonce).map_err(|_| CryptoError::Encryption)?;

    let outer_ct = aead::encrypt(&outer_key, &outer_nonce, &padded, b"")?;

    // Final output: outer_ek_pub (32) + outer_nonce (12) + outer_ct
    let mut output = Vec::with_capacity(32 + 12 + outer_ct.len());
    output.extend_from_slice(outer_ek_pub.as_bytes());
    output.extend_from_slice(&outer_nonce);
    output.extend_from_slice(&outer_ct);

    // Zeroize
    inner_key.zeroize();
    outer_key.zeroize();

    Ok(output)
}

/// Relay unseals outer layer (learns nothing about sender).
pub fn unseal_outer(
    envelope: &[u8],
    relay_secret: &StaticSecret,
) -> Result<Vec<u8>, CryptoError> {
    if envelope.len() < 44 + 16 {
        return Err(CryptoError::InvalidLength);
    }

    let mut ek_bytes = [0u8; 32];
    ek_bytes.copy_from_slice(&envelope[..32]);
    let ek_pub = PublicKey::from(ek_bytes);

    let mut nonce = [0u8; 12];
    nonce.copy_from_slice(&envelope[32..44]);

    let outer_shared = relay_secret.diffie_hellman(&ek_pub);
    let mut key = [0u8; 32];
    let hk = hkdf::Hkdf::<sha2::Sha256>::new(None, outer_shared.as_bytes());
    hk.expand(b"HadesSealedSenderV2Outer", &mut key)
        .map_err(|_| CryptoError::KdfError)?;

    let padded = aead::decrypt(&key, &nonce, &envelope[44..], b"")?;
    key.zeroize();

    // Remove padding
    if padded.len() < 4 {
        return Err(CryptoError::InvalidLength);
    }
    let inner_len =
        u32::from_be_bytes([padded[0], padded[1], padded[2], padded[3]]) as usize;
    if padded.len() < 4 + inner_len {
        return Err(CryptoError::InvalidLength);
    }

    Ok(padded[4..4 + inner_len].to_vec())
}

/// Recipient unseals inner layer (recovers sender identity + plaintext).
pub fn unseal_inner(
    inner_envelope: &[u8],
    recipient_secret: &StaticSecret,
) -> Result<([u8; 32], Vec<u8>), CryptoError> {
    if inner_envelope.len() < 44 + 16 + 32 {
        return Err(CryptoError::InvalidLength);
    }

    let mut ek_bytes = [0u8; 32];
    ek_bytes.copy_from_slice(&inner_envelope[..32]);
    let ek_pub = PublicKey::from(ek_bytes);

    let mut nonce = [0u8; 12];
    nonce.copy_from_slice(&inner_envelope[32..44]);

    let inner_shared = recipient_secret.diffie_hellman(&ek_pub);
    let mut key = [0u8; 32];
    let hk = hkdf::Hkdf::<sha2::Sha256>::new(None, inner_shared.as_bytes());
    hk.expand(b"HadesSealedSenderV2Inner", &mut key)
        .map_err(|_| CryptoError::KdfError)?;

    let plaintext = aead::decrypt(&key, &nonce, &inner_envelope[44..], b"")?;
    key.zeroize();

    if plaintext.len() < 32 {
        return Err(CryptoError::InvalidLength);
    }

    let mut sender_identity = [0u8; 32];
    sender_identity.copy_from_slice(&plaintext[..32]);
    let content = plaintext[32..].to_vec();

    Ok((sender_identity, content))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seal_unseal_roundtrip() {
        let sender_id = [0x01u8; 32];
        let recipient_sk = StaticSecret::random_from_rng(rand::thread_rng());
        let recipient_pk = PublicKey::from(&recipient_sk);
        let relay_sk = StaticSecret::random_from_rng(rand::thread_rng());
        let relay_pk = PublicKey::from(&relay_sk);

        let plaintext = b"Hello, Hades!";
        let sealed = seal_v2(plaintext, &sender_id, &recipient_pk, &relay_pk).unwrap();

        // Relay peels outer layer
        let inner = unseal_outer(&sealed, &relay_sk).unwrap();

        // Recipient peels inner layer
        let (recovered_sender, recovered_plaintext) =
            unseal_inner(&inner, &recipient_sk).unwrap();

        assert_eq!(recovered_sender, sender_id);
        assert_eq!(recovered_plaintext, plaintext);
    }

    #[test]
    fn test_wrong_key_fails() {
        let sender_id = [0x02u8; 32];
        let recipient_sk = StaticSecret::random_from_rng(rand::thread_rng());
        let recipient_pk = PublicKey::from(&recipient_sk);
        let relay_sk = StaticSecret::random_from_rng(rand::thread_rng());
        let relay_pk = PublicKey::from(&relay_sk);

        let sealed = seal_v2(b"secret", &sender_id, &recipient_pk, &relay_pk).unwrap();

        // Wrong relay key
        let wrong_sk = StaticSecret::random_from_rng(rand::thread_rng());
        assert!(unseal_outer(&sealed, &wrong_sk).is_err());
    }
}
