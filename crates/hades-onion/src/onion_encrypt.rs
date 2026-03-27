use chacha20poly1305::{aead::Aead, ChaCha20Poly1305, KeyInit};
use crate::relay_node::RelayNode;

/// Onion-encrypt data through multiple relay hops.
///
/// Each layer is encrypted with the shared key of the corresponding
/// relay node, starting from the innermost (exit) node outward.
pub fn onion_encrypt(data: &[u8], relays: &[RelayNode]) -> Result<Vec<u8>, OnionError> {
    let mut result = data.to_vec();

    // Encrypt from innermost to outermost (reverse order)
    for relay in relays.iter().rev() {
        let cipher = ChaCha20Poly1305::new_from_slice(relay.key())
            .map_err(|_| OnionError::CipherInit)?;
        let nonce = chacha20poly1305::Nonce::default();
        result = cipher
            .encrypt(&nonce, result.as_ref())
            .map_err(|_| OnionError::EncryptionFailed)?;
    }

    Ok(result)
}

/// Peel one layer of onion encryption (performed by a relay node).
pub fn onion_decrypt_layer(data: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, OnionError> {
    let cipher = ChaCha20Poly1305::new_from_slice(key)
        .map_err(|_| OnionError::CipherInit)?;
    let nonce = chacha20poly1305::Nonce::default();
    cipher
        .decrypt(&nonce, data)
        .map_err(|_| OnionError::DecryptionFailed)
}

#[derive(Debug)]
pub enum OnionError {
    CipherInit,
    EncryptionFailed,
    DecryptionFailed,
}

#[cfg(test)]
mod tests {
    use super::*;
    use x25519_dalek::{PublicKey as X25519Public, StaticSecret};

    fn make_relay(name: &str) -> (StaticSecret, RelayNode) {
        let client_secret = StaticSecret::random_from_rng(rand::thread_rng());
        let node_secret = StaticSecret::random_from_rng(rand::thread_rng());
        let node_public = X25519Public::from(&node_secret);

        let relay = RelayNode::from_handshake(&client_secret, &node_public, name.to_string());
        // Return the node_secret so tests can decrypt
        (node_secret, relay)
    }

    #[test]
    fn test_single_layer_roundtrip() {
        let client_secret = StaticSecret::random_from_rng(rand::thread_rng());
        let node_secret = StaticSecret::random_from_rng(rand::thread_rng());
        let node_public = X25519Public::from(&node_secret);
        let client_public = X25519Public::from(&client_secret);

        let relay = RelayNode::from_handshake(&client_secret, &node_public, "node1".into());

        // Node computes same shared key
        let node_shared = node_secret.diffie_hellman(&client_public);
        let node_key = *blake3::hash(node_shared.as_bytes()).as_bytes();

        let plaintext = b"secret message";
        let encrypted = onion_encrypt(plaintext, &[relay]).unwrap();
        let decrypted = onion_decrypt_layer(&encrypted, &node_key).unwrap();
        assert_eq!(decrypted, plaintext);
    }
}
