//! Cryptographic fingerprint derivation for Hades contact verification.
//!
//! Produces a deterministic "safety number" from two public keys using
//! BLAKE3. Both parties compute the same grid because keys are sorted
//! lexicographically before hashing.

use blake3;

/// Number of hex chunks in the fingerprint grid (5 rows x 2 columns).
const FINGERPRINT_CHUNKS: usize = 10;
/// Characters per chunk.
const CHUNK_SIZE: usize = 4;

/// Derive a contact fingerprint from two public keys.
///
/// Keys are sorted lexicographically so both participants produce the
/// same result regardless of who is "Alice" and who is "Bob".
///
/// Returns a `Vec<String>` of uppercase hex chunks (e.g. `["AE28", "89F1", ...]`).
pub fn derive_fingerprint(my_key: &[u8], their_key: &[u8]) -> Vec<String> {
    // 1. Sort keys to guarantee identical output on both devices.
    let (first, second) = if my_key <= their_key {
        (my_key, their_key)
    } else {
        (their_key, my_key)
    };

    // 2. BLAKE3 hash of concatenated keys.
    let mut hasher = blake3::Hasher::new();
    hasher.update(first);
    hasher.update(second);
    let hash = hasher.finalize();

    // 3. Format into uppercase hex chunks.
    let hex = hash.to_hex();
    let hex_str = hex.as_str().to_uppercase();
    hex_str
        .as_bytes()
        .chunks(CHUNK_SIZE)
        .take(FINGERPRINT_CHUNKS)
        .map(|chunk| String::from_utf8_lossy(chunk).to_string())
        .collect()
}

/// Derive a fingerprint from hex-encoded public key strings.
///
/// Convenience wrapper for the Tauri command layer.
pub fn derive_fingerprint_from_hex(my_key_hex: &str, their_key_hex: &str) -> Vec<String> {
    derive_fingerprint(my_key_hex.as_bytes(), their_key_hex.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fingerprint_is_deterministic() {
        let alice = b"alice_pub_key_x25519_00000000001";
        let bob = b"bob_pub_key_x25519_000000000002";

        let fp1 = derive_fingerprint(alice, bob);
        let fp2 = derive_fingerprint(bob, alice); // Reversed order

        assert_eq!(fp1, fp2, "Fingerprint must be the same regardless of key order");
    }

    #[test]
    fn fingerprint_has_correct_shape() {
        let alice = b"key_a";
        let bob = b"key_b";
        let fp = derive_fingerprint(alice, bob);

        assert_eq!(fp.len(), FINGERPRINT_CHUNKS);
        for chunk in &fp {
            assert_eq!(chunk.len(), CHUNK_SIZE);
            assert!(chunk.chars().all(|c| c.is_ascii_hexdigit()));
        }
    }

    #[test]
    fn different_keys_produce_different_fingerprints() {
        let fp1 = derive_fingerprint(b"key_a", b"key_b");
        let fp2 = derive_fingerprint(b"key_a", b"key_c");
        assert_ne!(fp1, fp2);
    }
}
