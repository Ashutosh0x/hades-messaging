//! Signal Sender Keys protocol for group E2EE messaging.
//!
//! Each group member generates a SenderKeyState and distributes it.
//! Messages are encrypted with the sender's chain key, which ratchets forward.

use crate::error::CryptoError;
use serde::{Deserialize, Serialize};
use zeroize::ZeroizeOnDrop;

#[derive(Debug, Clone, Serialize, Deserialize, ZeroizeOnDrop)]
pub struct SenderKeyState {
    #[zeroize(skip)]
    pub chain_id: u32,
    #[zeroize(skip)]
    pub iteration: u32,
    pub chain_key: [u8; 32],
    pub signing_key: [u8; 32],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SenderKeyMessage {
    pub chain_id: u32,
    pub iteration: u32,
    pub ciphertext: Vec<u8>,
    pub signature: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SenderKeyDistribution {
    pub chain_id: u32,
    pub iteration: u32,
    pub chain_key: Vec<u8>,
    pub signing_public_key: Vec<u8>,
}

impl SenderKeyState {
    pub fn generate() -> Self {
        let mut chain_key = [0u8; 32];
        let mut signing_key = [0u8; 32];
        getrandom::getrandom(&mut chain_key).unwrap();
        getrandom::getrandom(&mut signing_key).unwrap();
        Self { chain_id: rand::random(), iteration: 0, chain_key, signing_key }
    }

    fn ratchet(&mut self) {
        self.chain_key = derive_chain_key(&self.chain_key);
        self.iteration += 1;
    }

    fn message_key(&self) -> [u8; 32] { derive_message_key(&self.chain_key) }

    pub fn encrypt(&mut self, plaintext: &[u8]) -> Result<SenderKeyMessage, CryptoError> {
        let msg_key = self.message_key();
        let mut nonce = [0u8; 12];
        nonce[0..4].copy_from_slice(&self.chain_id.to_be_bytes());
        nonce[4..8].copy_from_slice(&self.iteration.to_be_bytes());

        use chacha20poly1305::{aead::Aead, KeyInit, ChaCha20Poly1305};
        let cipher = ChaCha20Poly1305::new_from_slice(&msg_key)
            .map_err(|_| CryptoError::Encryption)?;
        let ciphertext = cipher.encrypt(&nonce.into(), plaintext)
            .map_err(|_| CryptoError::Encryption)?;

        let signature = blake3::keyed_hash(&self.signing_key, &ciphertext);
        let msg = SenderKeyMessage {
            chain_id: self.chain_id, iteration: self.iteration,
            ciphertext, signature: signature.as_bytes().to_vec(),
        };
        self.ratchet();
        Ok(msg)
    }

    pub fn create_distribution(&self) -> SenderKeyDistribution {
        let signing_public = blake3::hash(&self.signing_key);
        SenderKeyDistribution {
            chain_id: self.chain_id, iteration: self.iteration,
            chain_key: self.chain_key.to_vec(),
            signing_public_key: signing_public.as_bytes().to_vec(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceivedSenderKey {
    pub sender_id: String,
    pub chain_id: u32,
    pub current_iteration: u32,
    pub chain_key: [u8; 32],
    pub signing_public_key: [u8; 32],
}

impl ReceivedSenderKey {
    pub fn from_distribution(sender_id: &str, dist: &SenderKeyDistribution) -> Self {
        let mut chain_key = [0u8; 32];
        chain_key.copy_from_slice(&dist.chain_key);
        let mut signing_pk = [0u8; 32];
        signing_pk.copy_from_slice(&dist.signing_public_key);
        Self {
            sender_id: sender_id.to_string(), chain_id: dist.chain_id,
            current_iteration: dist.iteration, chain_key, signing_public_key: signing_pk,
        }
    }

    pub fn decrypt(&mut self, msg: &SenderKeyMessage) -> Result<Vec<u8>, CryptoError> {
        if msg.chain_id != self.chain_id { return Err(CryptoError::DecryptionFailed); }

        // Fast-forward chain key
        while self.current_iteration < msg.iteration {
            self.chain_key = derive_chain_key(&self.chain_key);
            self.current_iteration += 1;
        }
        if self.current_iteration != msg.iteration { return Err(CryptoError::DecryptionFailed); }

        let msg_key = derive_message_key(&self.chain_key);
        let mut nonce = [0u8; 12];
        nonce[0..4].copy_from_slice(&self.chain_id.to_be_bytes());
        nonce[4..8].copy_from_slice(&self.current_iteration.to_be_bytes());

        use chacha20poly1305::{aead::Aead, KeyInit, ChaCha20Poly1305};
        let cipher = ChaCha20Poly1305::new_from_slice(&msg_key)
            .map_err(|_| CryptoError::DecryptionFailed)?;
        let plaintext = cipher.decrypt(&nonce.into(), msg.ciphertext.as_ref())
            .map_err(|_| CryptoError::DecryptionFailed)?;

        self.chain_key = derive_chain_key(&self.chain_key);
        self.current_iteration += 1;
        Ok(plaintext)
    }
}

fn derive_chain_key(key: &[u8; 32]) -> [u8; 32] {
    *blake3::keyed_hash(key, b"SenderKeyChain").as_bytes()
}

fn derive_message_key(key: &[u8; 32]) -> [u8; 32] {
    *blake3::keyed_hash(key, b"SenderKeyMessage").as_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sender_key_roundtrip() {
        let mut alice_key = SenderKeyState::generate();
        let dist = alice_key.create_distribution();
        let mut bob_received = ReceivedSenderKey::from_distribution("alice", &dist);

        let msg1 = alice_key.encrypt(b"Hello group!").unwrap();
        let msg2 = alice_key.encrypt(b"Second message").unwrap();

        let plain1 = bob_received.decrypt(&msg1).unwrap();
        let plain2 = bob_received.decrypt(&msg2).unwrap();

        assert_eq!(plain1, b"Hello group!");
        assert_eq!(plain2, b"Second message");
    }

    #[test]
    fn test_skip_messages() {
        let mut alice = SenderKeyState::generate();
        let dist = alice.create_distribution();
        let mut bob = ReceivedSenderKey::from_distribution("alice", &dist);

        let msg1 = alice.encrypt(b"First").unwrap();
        let _msg2 = alice.encrypt(b"Second").unwrap(); // skipped
        let msg3 = alice.encrypt(b"Third").unwrap();

        bob.decrypt(&msg1).unwrap();
        let plain3 = bob.decrypt(&msg3).unwrap();
        assert_eq!(plain3, b"Third");
    }
}
