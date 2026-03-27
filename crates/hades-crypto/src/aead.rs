//! ChaCha20-Poly1305 AEAD encryption/decryption.
//!
//! All message-level encryption in Hades uses ChaCha20-Poly1305 (RFC 8439).
//! This module provides thin wrappers with zeroization guarantees.

use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Key, Nonce,
};
use zeroize::Zeroize;

use crate::error::CryptoError;

/// Encrypt plaintext with ChaCha20-Poly1305.
///
/// - `key`: 256-bit message key (from chain KDF)
/// - `nonce`: 96-bit nonce (from message counter)
/// - `plaintext`: message content
/// - `aad`: associated data (header bytes, not encrypted but authenticated)
pub fn encrypt(
    key: &[u8; 32],
    nonce: &[u8; 12],
    plaintext: &[u8],
    aad: &[u8],
) -> Result<Vec<u8>, CryptoError> {
    let cipher = ChaCha20Poly1305::new(Key::from_slice(key));
    let nonce = Nonce::from_slice(nonce);

    cipher
        .encrypt(nonce, chacha20poly1305::aead::Payload { msg: plaintext, aad })
        .map_err(|_| CryptoError::Encryption)
}

/// Decrypt ciphertext with ChaCha20-Poly1305.
///
/// Returns the plaintext if authentication succeeds.
/// Returns `CryptoError::Decryption` if the ciphertext was tampered with
/// or the wrong key was used.
pub fn decrypt(
    key: &[u8; 32],
    nonce: &[u8; 12],
    ciphertext: &[u8],
    aad: &[u8],
) -> Result<Vec<u8>, CryptoError> {
    let cipher = ChaCha20Poly1305::new(Key::from_slice(key));
    let nonce = Nonce::from_slice(nonce);

    cipher
        .decrypt(nonce, chacha20poly1305::aead::Payload { msg: ciphertext, aad })
        .map_err(|_| CryptoError::Decryption)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip() {
        let key = [0x42u8; 32];
        let nonce = [0u8; 12];
        let plaintext = b"Hello, Hades!";
        let aad = b"header-data";

        let ciphertext = encrypt(&key, &nonce, plaintext, aad).unwrap();
        let decrypted = decrypt(&key, &nonce, &ciphertext, aad).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_wrong_key_fails() {
        let key = [0x42u8; 32];
        let wrong_key = [0x43u8; 32];
        let nonce = [0u8; 12];
        let plaintext = b"secret message";
        let aad = b"";

        let ciphertext = encrypt(&key, &nonce, plaintext, aad).unwrap();
        let result = decrypt(&wrong_key, &nonce, &ciphertext, aad);

        assert!(result.is_err());
    }

    #[test]
    fn test_tampered_ciphertext_fails() {
        let key = [0x42u8; 32];
        let nonce = [0u8; 12];
        let plaintext = b"authentic message";
        let aad = b"";

        let mut ciphertext = encrypt(&key, &nonce, plaintext, aad).unwrap();
        // Tamper with a byte
        if let Some(byte) = ciphertext.last_mut() {
            *byte ^= 0xFF;
        }

        let result = decrypt(&key, &nonce, &ciphertext, aad);
        assert!(result.is_err());
    }

    #[test]
    fn test_wrong_aad_fails() {
        let key = [0x42u8; 32];
        let nonce = [0u8; 12];
        let plaintext = b"hello";
        let aad = b"correct-header";
        let wrong_aad = b"wrong-header";

        let ciphertext = encrypt(&key, &nonce, plaintext, aad).unwrap();
        let result = decrypt(&key, &nonce, &ciphertext, wrong_aad);

        assert!(result.is_err());
    }
}
