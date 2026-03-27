//! Secure entropy generation for Hades protocol.
//!
//! Wraps the platform CSPRNG. All randomness in Hades flows through here
//! for auditability.

/// Generate `n` cryptographically secure random bytes.
pub fn random_bytes(n: usize) -> Vec<u8> {
    let mut buf = vec![0u8; n];
    getrandom::getrandom(&mut buf).expect("CSPRNG failure is fatal");
    buf
}

/// Generate a random 32-byte key.
pub fn random_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    getrandom::getrandom(&mut key).expect("CSPRNG failure is fatal");
    key
}

/// Generate a random 12-byte nonce.
pub fn random_nonce() -> [u8; 12] {
    let mut nonce = [0u8; 12];
    getrandom::getrandom(&mut nonce).expect("CSPRNG failure is fatal");
    nonce
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_bytes_length() {
        assert_eq!(random_bytes(32).len(), 32);
        assert_eq!(random_bytes(64).len(), 64);
        assert_eq!(random_bytes(0).len(), 0);
    }

    #[test]
    fn test_random_key_uniqueness() {
        let k1 = random_key();
        let k2 = random_key();
        assert_ne!(k1, k2);
    }
}
