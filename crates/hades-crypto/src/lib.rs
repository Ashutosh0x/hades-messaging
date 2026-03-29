//! # hades-crypto
//!
//! Cryptographic core for Hades Messaging.
//!
//! Provides:
//! - **PQXDH**: Hybrid X25519 + ML-KEM-768 post-quantum key exchange
//! - **Double Ratchet**: Per-message forward secrecy with SPQR extension
//! - **AEAD**: ChaCha20-Poly1305 authenticated encryption
//! - **KDF**: HKDF-based key derivation (root chain, message chain)
//! - **Sealed Sender**: Hide sender identity from the relay server
//! - **Padding**: Fixed-bucket message padding against length analysis
//! - **Entropy**: Secure random number generation

pub mod aead;
pub mod anti_forensics;
pub mod double_ratchet;
pub mod entropy;
pub mod error;
pub mod fingerprint;
pub mod kdf;
pub mod padding;
pub mod pqxdh;
pub mod sealed_sender;
pub mod sealed_sender_v2;

pub mod notifications;
pub mod audio;
pub mod calls;
pub mod search;
pub mod sender_keys;

pub use error::CryptoError;
