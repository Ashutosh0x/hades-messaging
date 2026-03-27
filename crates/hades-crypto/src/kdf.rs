//! Key Derivation Functions for the Hades Double Ratchet.
//!
//! Provides domain-separated HKDF-based key derivations:
//! - Root KDF: advances root key on each DH ratchet step
//! - Chain KDF: derives per-message keys from chain key
//! - Nonce derivation: counter-based nonce for ChaCha20-Poly1305

use hkdf::Hkdf;
use sha2::Sha256;

use hades_common::constants::hkdf_info;

/// Root Key KDF — derives new root key + chain key from DH output.
///
/// Used during a DH ratchet step when a new DH key is received.
///
/// ```text
/// (root_key, chain_key) = HKDF(salt=old_root_key, ikm=dh_output)
/// ```
pub fn kdf_rk(root_key: &[u8; 32], dh_output: &[u8]) -> ([u8; 32], [u8; 32]) {
    let hk = Hkdf::<Sha256>::new(Some(root_key), dh_output);

    let mut new_root = [0u8; 32];
    hk.expand(hkdf_info::ROOT_KEY, &mut new_root)
        .expect("HKDF-expand for root key should not fail with 32-byte output");

    let mut chain_key = [0u8; 32];
    hk.expand(hkdf_info::CHAIN_KEY, &mut chain_key)
        .expect("HKDF-expand for chain key should not fail with 32-byte output");

    (new_root, chain_key)
}

/// Chain Key KDF — derives new chain key + message key.
///
/// Used for each message sent/received on a given chain.
///
/// ```text
/// new_chain_key = HMAC-SHA256(chain_key, 0x02)
/// message_key   = HMAC-SHA256(chain_key, 0x01)
/// ```
///
/// We use HKDF for consistency, with the chain key as IKM and
/// distinct info strings for domain separation.
pub fn kdf_ck(chain_key: &[u8; 32]) -> ([u8; 32], [u8; 32]) {
    let hk = Hkdf::<Sha256>::new(None, chain_key);

    let mut new_chain = [0u8; 32];
    hk.expand(hkdf_info::CHAIN_KEY, &mut new_chain)
        .expect("HKDF-expand for chain key should not fail");

    let mut message_key = [0u8; 32];
    hk.expand(hkdf_info::MESSAGE_KEY, &mut message_key)
        .expect("HKDF-expand for message key should not fail");

    (new_chain, message_key)
}

/// Derive a 12-byte nonce from a message counter.
///
/// The nonce is the counter encoded as little-endian u32, zero-padded to 12 bytes.
/// This is safe because each message key is used exactly once (the chain advances),
/// so the (key, nonce) pair is never reused.
pub fn derive_nonce(counter: u32) -> [u8; 12] {
    let mut nonce = [0u8; 12];
    nonce[..4].copy_from_slice(&counter.to_le_bytes());
    nonce
}

/// Session key derivation for PQXDH.
///
/// Combines all DH shared secrets (and optional PQ shared secret) into
/// a single 32-byte session key using HKDF.
pub fn derive_session_key(
    dh_secrets: &[&[u8]],
    pq_shared_secret: Option<&[u8]>,
) -> [u8; 32] {
    // Concatenate all input key material
    let total_len: usize = dh_secrets.iter().map(|s| s.len()).sum::<usize>()
        + pq_shared_secret.map_or(0, |s| s.len());
    let mut ikm = Vec::with_capacity(total_len);
    for secret in dh_secrets {
        ikm.extend_from_slice(secret);
    }
    if let Some(pq) = pq_shared_secret {
        ikm.extend_from_slice(pq);
    }

    let hk = Hkdf::<Sha256>::new(Some(hkdf_info::PQXDH_SESSION), &ikm);
    let mut session_key = [0u8; 32];
    hk.expand(b"HadesSessionKey", &mut session_key)
        .expect("HKDF-expand for session key should not fail");

    // Zero intermediate material
    ikm.fill(0);

    session_key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kdf_rk_deterministic() {
        let root = [0x42u8; 32];
        let dh_out = [0x13u8; 32];

        let (r1, c1) = kdf_rk(&root, &dh_out);
        let (r2, c2) = kdf_rk(&root, &dh_out);

        assert_eq!(r1, r2);
        assert_eq!(c1, c2);
        // Root and chain should differ
        assert_ne!(r1, c1);
    }

    #[test]
    fn test_kdf_rk_different_inputs() {
        let root = [0x42u8; 32];
        let (r1, c1) = kdf_rk(&root, &[0x01u8; 32]);
        let (r2, c2) = kdf_rk(&root, &[0x02u8; 32]);

        assert_ne!(r1, r2);
        assert_ne!(c1, c2);
    }

    #[test]
    fn test_kdf_ck_chain_advances() {
        let chain = [0xAAu8; 32];
        let (new_chain, msg_key) = kdf_ck(&chain);

        // New chain key should differ from old
        assert_ne!(new_chain, chain);
        // Message key should differ from chain key
        assert_ne!(msg_key, new_chain);
        assert_ne!(msg_key, chain);
    }

    #[test]
    fn test_kdf_ck_successive_steps() {
        let chain0 = [0xBBu8; 32];
        let (chain1, mk1) = kdf_ck(&chain0);
        let (chain2, mk2) = kdf_ck(&chain1);

        // All should be unique
        assert_ne!(mk1, mk2);
        assert_ne!(chain1, chain2);
    }

    #[test]
    fn test_derive_nonce() {
        let n0 = derive_nonce(0);
        assert_eq!(n0, [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);

        let n1 = derive_nonce(1);
        assert_eq!(n1, [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);

        let n_max = derive_nonce(u32::MAX);
        assert_eq!(n_max[..4], [0xFF, 0xFF, 0xFF, 0xFF]);
        assert_eq!(n_max[4..], [0, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_derive_session_key_deterministic() {
        let dh1 = [0x11u8; 32];
        let dh2 = [0x22u8; 32];
        let pq = [0x33u8; 32];

        let sk1 = derive_session_key(&[&dh1, &dh2], Some(&pq));
        let sk2 = derive_session_key(&[&dh1, &dh2], Some(&pq));
        assert_eq!(sk1, sk2);
    }

    #[test]
    fn test_derive_session_key_with_without_pq() {
        let dh1 = [0x11u8; 32];
        let sk_with = derive_session_key(&[&dh1], Some(&[0x33u8; 32]));
        let sk_without = derive_session_key(&[&dh1], None);
        assert_ne!(sk_with, sk_without);
    }
}
