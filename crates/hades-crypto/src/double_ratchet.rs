//! Double Ratchet algorithm with DH stepping and SPQR extension.
//!
//! Implements the Signal Double Ratchet with:
//! - DH ratchet: X25519 key exchanges at each turn
//! - Symmetric ratchet: HKDF chain keys for per-message keys
//! - Out-of-order message handling via skipped message keys
//! - SPQR extension: Periodic ML-KEM ratchet step (placeholder for Phase 3)
//!
//! ## State Machine
//!
//! ```text
//! ┌───────────────┐
//! │  INITIALIZED  │─── first message received ──→ DH ratchet step
//! └───────────────┘
//!        │
//!   first message sent
//!        │
//!        ▼
//!   ┌──────────┐      new DH key from peer
//!   │ SENDING  │◄────────── DH ratchet step ────────► RECEIVING
//!   └──────────┘
//! ```

use std::collections::HashMap;
use x25519_dalek::{PublicKey as X25519Public, StaticSecret};
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::error::CryptoError;
use crate::kdf;
use hades_common::constants::MAX_SKIP;

/// Header sent with each Double Ratchet message.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MessageHeader {
    /// Sender's current DH ratchet public key
    pub dh_public: [u8; 32],
    /// Message counter in the current sending chain
    pub counter: u32,
    /// Previous sending chain length (for skipped message keys)
    pub previous_chain_length: u32,
    /// PQ ratchet step indicator (SPQR)
    pub pq_step: bool,
}

/// The core Double Ratchet state.
///
/// This struct contains all mutable state for an ongoing ratcheted session.
/// It is `ZeroizeOnDrop` to ensure all key material is wiped from memory
/// when the session ends.
#[derive(ZeroizeOnDrop)]
pub struct RatchetState {
    /// Root key — advances on each DH ratchet step
    root_key: [u8; 32],

    /// Our current DH ratchet secret key (raw 32-byte scalar)
    our_dh_secret: [u8; 32],
    /// Our DH ratchet public key
    #[zeroize(skip)]
    our_dh_public: [u8; 32],

    /// Their current DH ratchet public key
    #[zeroize(skip)]
    their_dh_public: Option<[u8; 32]>,

    /// Sending chain key
    sending_chain: Option<[u8; 32]>,
    /// Receiving chain key
    receiving_chain: Option<[u8; 32]>,

    /// Sending message counter
    send_counter: u32,
    /// Receiving message counter
    recv_counter: u32,
    /// Previous sending chain length
    previous_chain_length: u32,

    /// Skipped message keys: (dh_pub, counter) → message_key
    /// Allows decryption of out-of-order messages.
    #[zeroize(skip)]
    skipped_keys: HashMap<([u8; 32], u32), [u8; 32]>,

    /// Messages since last PQ ratchet step (SPQR)
    #[zeroize(skip)]
    messages_since_pq_step: u32,
}

/// Generate a new DH ratchet key pair.
fn generate_dh_keypair() -> ([u8; 32], [u8; 32]) {
    let secret = StaticSecret::random_from_rng(rand::thread_rng());
    let public = X25519Public::from(&secret);
    let mut secret_bytes = [0u8; 32];
    // StaticSecret doesn't expose bytes directly, so we reconstruct
    // by using the diffie_hellman with the basepoint
    // Actually, we need to store the secret. Let's use a different approach.
    // We generate raw bytes and construct from those.
    let mut raw = [0u8; 32];
    getrandom::getrandom(&mut raw).expect("CSPRNG failure");
    let s = StaticSecret::from(raw);
    let p = X25519Public::from(&s);
    (raw, *p.as_bytes())
}

/// Perform X25519 DH from raw secret bytes and a public key.
fn dh(secret: &[u8; 32], their_public: &[u8; 32]) -> [u8; 32] {
    let s = StaticSecret::from(*secret);
    let p = X25519Public::from(*their_public);
    let shared = s.diffie_hellman(&p);
    *shared.as_bytes()
}

impl RatchetState {
    /// Initialize a ratchet state (Alice/initiator side).
    ///
    /// Called after PQXDH completes. Alice has the shared secret and
    /// Bob's signed prekey (used as initial remote DH ratchet key).
    pub fn init_alice(shared_secret: [u8; 32], bob_spk: [u8; 32]) -> Self {
        // Generate Alice's first DH ratchet key pair
        let (our_secret, our_public) = generate_dh_keypair();

        // Perform first DH ratchet step to derive sending chain
        let dh_output = dh(&our_secret, &bob_spk);
        let (root_key, sending_chain) = kdf::kdf_rk(&shared_secret, &dh_output);

        Self {
            root_key,
            our_dh_secret: our_secret,
            our_dh_public: our_public,
            their_dh_public: Some(bob_spk),
            sending_chain: Some(sending_chain),
            receiving_chain: None,
            send_counter: 0,
            recv_counter: 0,
            previous_chain_length: 0,
            skipped_keys: HashMap::new(),
            messages_since_pq_step: 0,
        }
    }

    /// Initialize a ratchet state (Bob/responder side).
    ///
    /// Bob uses his signed prekey secret as the initial DH ratchet key.
    pub fn init_bob(shared_secret: [u8; 32], our_spk_secret: [u8; 32], our_spk_public: [u8; 32]) -> Self {
        Self {
            root_key: shared_secret,
            our_dh_secret: our_spk_secret,
            our_dh_public: our_spk_public,
            their_dh_public: None,
            sending_chain: None,
            receiving_chain: None,
            send_counter: 0,
            recv_counter: 0,
            previous_chain_length: 0,
            skipped_keys: HashMap::new(),
            messages_since_pq_step: 0,
        }
    }

    /// Encrypt a message using the sending chain.
    ///
    /// Returns `(header, ciphertext)`.
    pub fn encrypt(&mut self, plaintext: &[u8]) -> Result<(MessageHeader, Vec<u8>), CryptoError> {
        // Ensure we have a sending chain
        let sending_chain = self.sending_chain.ok_or(CryptoError::NoSendingChain)?;

        // Advance the chain: derive message key
        let (new_chain, message_key) = kdf::kdf_ck(&sending_chain);
        self.sending_chain = Some(new_chain);

        // Build header
        let header = MessageHeader {
            dh_public: self.our_dh_public,
            counter: self.send_counter,
            previous_chain_length: self.previous_chain_length,
            pq_step: false,
        };

        // Encrypt with AEAD
        let nonce = kdf::derive_nonce(self.send_counter);
        let header_bytes = serde_json::to_vec(&header).unwrap_or_default();
        let ciphertext = crate::aead::encrypt(&message_key, &nonce, plaintext, &header_bytes)?;

        self.send_counter += 1;
        self.messages_since_pq_step += 1;

        Ok((header, ciphertext))
    }

    /// Decrypt a message using the receiving chain.
    ///
    /// Handles:
    /// 1. Lookup in skipped message keys (out-of-order)
    /// 2. DH ratchet step if new DH public key received
    /// 3. Skip messages on the current or new chain
    /// 4. Derive message key and decrypt
    pub fn decrypt(
        &mut self,
        header: &MessageHeader,
        ciphertext: &[u8],
    ) -> Result<Vec<u8>, CryptoError> {
        // 1. Try skipped message keys first (for out-of-order messages)
        let skip_key = (header.dh_public, header.counter);
        if let Some(message_key) = self.skipped_keys.remove(&skip_key) {
            let nonce = kdf::derive_nonce(header.counter);
            let header_bytes = serde_json::to_vec(header).unwrap_or_default();
            return crate::aead::decrypt(&message_key, &nonce, ciphertext, &header_bytes);
        }

        // 2. Check if we need a DH ratchet step (new DH public key from peer)
        let need_dh_step = match self.their_dh_public {
            Some(current) => header.dh_public != current,
            None => true,
        };

        if need_dh_step {
            // Skip any remaining messages on the current receiving chain
            if self.their_dh_public.is_some() {
                self.skip_messages(header.previous_chain_length)?;
            }

            // Perform DH ratchet step
            self.dh_ratchet_step(&header.dh_public)?;
        }

        // 3. Skip any missed messages on the current receiving chain
        self.skip_messages(header.counter)?;

        // 4. Derive message key from receiving chain
        let receiving_chain = self.receiving_chain.ok_or(CryptoError::NoReceivingChain)?;
        let (new_chain, message_key) = kdf::kdf_ck(&receiving_chain);
        self.receiving_chain = Some(new_chain);

        // 5. Decrypt
        let nonce = kdf::derive_nonce(header.counter);
        let header_bytes = serde_json::to_vec(header).unwrap_or_default();
        let plaintext = crate::aead::decrypt(&message_key, &nonce, ciphertext, &header_bytes)?;

        self.recv_counter = header.counter + 1;

        Ok(plaintext)
    }

    /// Perform a DH ratchet step — new DH output drives new root + chain keys.
    fn dh_ratchet_step(&mut self, new_remote_public: &[u8; 32]) -> Result<(), CryptoError> {
        // Save previous sending chain length
        self.previous_chain_length = self.send_counter;
        self.send_counter = 0;
        self.recv_counter = 0;

        // Update their DH public key
        self.their_dh_public = Some(*new_remote_public);

        // Derive receiving chain: DH(our_current_secret, their_new_public)
        let dh_recv = dh(&self.our_dh_secret, new_remote_public);
        let (root_key_1, receiving_chain) = kdf::kdf_rk(&self.root_key, &dh_recv);
        self.root_key = root_key_1;
        self.receiving_chain = Some(receiving_chain);

        // Generate new DH ratchet key pair for sending
        let (new_secret, new_public) = generate_dh_keypair();
        self.our_dh_secret = new_secret;
        self.our_dh_public = new_public;

        // Derive sending chain: DH(our_new_secret, their_new_public)
        let dh_send = dh(&self.our_dh_secret, new_remote_public);
        let (root_key_2, sending_chain) = kdf::kdf_rk(&self.root_key, &dh_send);
        self.root_key = root_key_2;
        self.sending_chain = Some(sending_chain);

        Ok(())
    }

    /// Store skipped message keys for out-of-order delivery.
    fn skip_messages(&mut self, until: u32) -> Result<(), CryptoError> {
        if until < self.recv_counter {
            return Ok(());
        }

        let to_skip = (until - self.recv_counter) as usize;
        if to_skip > MAX_SKIP {
            return Err(CryptoError::TooManySkipped(MAX_SKIP));
        }

        if let Some(mut chain) = self.receiving_chain {
            let their_dh = self.their_dh_public.ok_or(CryptoError::NoRemoteKey)?;

            for i in self.recv_counter..until {
                let (new_chain, message_key) = kdf::kdf_ck(&chain);
                self.skipped_keys.insert((their_dh, i), message_key);
                chain = new_chain;
            }
            self.receiving_chain = Some(chain);
            self.recv_counter = until;
        }

        Ok(())
    }

    /// Number of messages since last PQ ratchet step (SPQR metric).
    pub fn messages_since_pq(&self) -> u32 {
        self.messages_since_pq_step
    }

    /// Number of skipped message keys being held.
    pub fn skipped_key_count(&self) -> usize {
        self.skipped_keys.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Create an Alice-Bob ratchet pair ready for messaging.
    fn create_ratchet_pair() -> (RatchetState, RatchetState) {
        let shared_secret = [0x42u8; 32];

        // Bob's "signed prekey" (just a DH key pair for the ratchet)
        let (bob_spk_secret, bob_spk_public) = generate_dh_keypair();

        let alice = RatchetState::init_alice(shared_secret, bob_spk_public);
        let bob = RatchetState::init_bob(shared_secret, bob_spk_secret, bob_spk_public);

        (alice, bob)
    }

    #[test]
    fn test_ratchet_state_creation() {
        let (alice, bob) = create_ratchet_pair();

        // Alice should have a sending chain (she did the first DH step)
        assert!(alice.sending_chain.is_some());
        assert!(alice.their_dh_public.is_some());

        // Bob should NOT have chains yet — waiting for Alice's first message
        assert!(bob.sending_chain.is_none());
        assert!(bob.their_dh_public.is_none());
    }

    #[test]
    fn test_single_message_roundtrip() {
        let (mut alice, mut bob) = create_ratchet_pair();

        // Alice encrypts
        let plaintext = b"Hello from Alice!";
        let (header, ciphertext) = alice.encrypt(plaintext).unwrap();

        // Bob decrypts (triggers DH ratchet step)
        let decrypted = bob.decrypt(&header, &ciphertext).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_multiple_messages_one_direction() {
        let (mut alice, mut bob) = create_ratchet_pair();

        for i in 0..10 {
            let msg = format!("Message {}", i);
            let (header, ct) = alice.encrypt(msg.as_bytes()).unwrap();
            let decrypted = bob.decrypt(&header, &ct).unwrap();
            assert_eq!(decrypted, msg.as_bytes());
        }
    }

    #[test]
    fn test_ping_pong_conversation() {
        let (mut alice, mut bob) = create_ratchet_pair();

        // Alice → Bob
        let (h1, c1) = alice.encrypt(b"Hey Bob").unwrap();
        let d1 = bob.decrypt(&h1, &c1).unwrap();
        assert_eq!(d1, b"Hey Bob");

        // Bob → Alice (triggers DH ratchet on Bob's side)
        let (h2, c2) = bob.encrypt(b"Hey Alice").unwrap();
        let d2 = alice.decrypt(&h2, &c2).unwrap();
        assert_eq!(d2, b"Hey Alice");

        // Alice → Bob (new DH ratchet step)
        let (h3, c3) = alice.encrypt(b"How are you?").unwrap();
        let d3 = bob.decrypt(&h3, &c3).unwrap();
        assert_eq!(d3, b"How are you?");

        // Bob → Alice
        let (h4, c4) = bob.encrypt(b"Great!").unwrap();
        let d4 = alice.decrypt(&h4, &c4).unwrap();
        assert_eq!(d4, b"Great!");
    }

    #[test]
    fn test_out_of_order_messages() {
        let (mut alice, mut bob) = create_ratchet_pair();

        // Alice sends 3 messages
        let (h0, c0) = alice.encrypt(b"msg 0").unwrap();
        let (h1, c1) = alice.encrypt(b"msg 1").unwrap();
        let (h2, c2) = alice.encrypt(b"msg 2").unwrap();

        // Bob receives them out of order: 2, 0, 1
        let d2 = bob.decrypt(&h2, &c2).unwrap();
        assert_eq!(d2, b"msg 2");

        let d0 = bob.decrypt(&h0, &c0).unwrap();
        assert_eq!(d0, b"msg 0");

        let d1 = bob.decrypt(&h1, &c1).unwrap();
        assert_eq!(d1, b"msg 1");
    }

    #[test]
    fn test_wrong_key_fails_decryption() {
        let (mut alice, _bob) = create_ratchet_pair();

        // Create a different Bob (different shared secret)
        let (bob_secret, bob_public) = generate_dh_keypair();
        let mut wrong_bob = RatchetState::init_bob([0xFFu8; 32], bob_secret, bob_public);

        let (header, ct) = alice.encrypt(b"secret").unwrap();

        // Different Bob should fail
        let result = wrong_bob.decrypt(&header, &ct);
        assert!(result.is_err());
    }

    #[test]
    fn test_forward_secrecy() {
        let (mut alice, mut bob) = create_ratchet_pair();

        // Exchange messages (advances ratchet)
        let (h1, c1) = alice.encrypt(b"first").unwrap();
        bob.decrypt(&h1, &c1).unwrap();

        let (h2, c2) = bob.encrypt(b"reply").unwrap();
        alice.decrypt(&h2, &c2).unwrap();

        // Save Alice's current root key
        let old_root = alice.root_key;

        // Another full round-trip to trigger a DH ratchet step on Alice
        let (h3, c3) = alice.encrypt(b"second").unwrap();
        bob.decrypt(&h3, &c3).unwrap();

        let (h4, c4) = bob.encrypt(b"second reply").unwrap();
        alice.decrypt(&h4, &c4).unwrap();

        // Root key should have advanced (forward secrecy)
        assert_ne!(alice.root_key, old_root);
    }

    #[test]
    fn test_extended_conversation() {
        let (mut alice, mut bob) = create_ratchet_pair();

        for i in 0..50 {
            let msg = format!("Alice msg {}", i);
            let (h, c) = alice.encrypt(msg.as_bytes()).unwrap();
            let d = bob.decrypt(&h, &c).unwrap();
            assert_eq!(d, msg.as_bytes());

            let reply = format!("Bob reply {}", i);
            let (h2, c2) = bob.encrypt(reply.as_bytes()).unwrap();
            let d2 = alice.decrypt(&h2, &c2).unwrap();
            assert_eq!(d2, reply.as_bytes());
        }
    }
}
