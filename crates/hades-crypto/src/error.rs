//! Crypto-specific error types.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("Key exchange computation failed: {0}")]
    KeyExchange(String),

    #[error("Signature verification failed")]
    SignatureVerification,

    #[error("Encryption failed")]
    Encryption,

    #[error("Decryption failed — invalid ciphertext or wrong key")]
    Decryption,

    #[error("Invalid key material: {0}")]
    InvalidKeyMaterial(String),

    #[error("No sending chain established")]
    NoSendingChain,

    #[error("No receiving chain established")]
    NoReceivingChain,

    #[error("Too many skipped messages (limit: {0})")]
    TooManySkipped(usize),

    #[error("No remote DH ratchet key")]
    NoRemoteKey,

    #[error("KDF output length error")]
    KdfLength,

    #[error("PQ KEM encapsulation failed")]
    KemEncapsulation,

    #[error("PQ KEM decapsulation failed")]
    KemDecapsulation,

    #[error("Sealed sender certificate invalid")]
    InvalidSenderCertificate,
}
