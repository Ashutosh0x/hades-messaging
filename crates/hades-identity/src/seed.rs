//! Unified Seed Manager for Hades Identity
//!
//! One BIP-39 mnemonic seed → messaging identity + all wallet keys.
//! Possession of the seed = possession of the account.
//!
//! ## Derivation Paths
//!
//! - Messaging identity: `m/13'/0'/0'` (purpose 13 = Hades messaging)
//! - Wallet keys:        `m/44'/{coin_type}'/0'/0/0` (BIP-44 standard)

use bip39::{Language, Mnemonic};
use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

/// The master seed from which ALL keys derive.
/// This is the single secret that controls the entire account.
#[derive(ZeroizeOnDrop)]
pub struct MasterSeed {
    mnemonic: String,
    seed: [u8; 64],
}

/// What gets persisted (encrypted by SQLCipher)
#[derive(Serialize, Deserialize)]
pub struct PersistedSeed {
    pub mnemonic_encrypted: Vec<u8>,
    pub mnemonic_nonce: [u8; 12],
    pub seed_salt: [u8; 16],
    pub created_at: u64,
}

impl MasterSeed {
    /// Generate a brand-new identity + wallet seed
    pub fn generate() -> Result<Self, SeedError> {
        let mnemonic = Mnemonic::generate_in(Language::English, 24)
            .map_err(|e| SeedError::Generation(e.to_string()))?;

        let seed_bytes = mnemonic.to_seed("");
        let mut seed = [0u8; 64];
        seed.copy_from_slice(&seed_bytes[..64]);

        Ok(Self {
            mnemonic: mnemonic.to_string(),
            seed,
        })
    }

    /// Restore from mnemonic phrase (recovery or import)
    pub fn from_mnemonic(phrase: &str) -> Result<Self, SeedError> {
        let mnemonic = Mnemonic::parse_in(Language::English, phrase)
            .map_err(|e| SeedError::InvalidMnemonic(e.to_string()))?;

        let seed_bytes = mnemonic.to_seed("");
        let mut seed = [0u8; 64];
        seed.copy_from_slice(&seed_bytes[..64]);

        Ok(Self {
            mnemonic: mnemonic.to_string(),
            seed,
        })
    }

    pub fn mnemonic(&self) -> &str {
        &self.mnemonic
    }

    pub fn seed_bytes(&self) -> &[u8; 64] {
        &self.seed
    }

    /// Derive the messaging identity keypair.
    ///
    /// Path: `m/13'/0'/0'` (purpose 13 = Hades messaging, not colliding with BIP-44/49/84)
    ///
    /// Returns an Ed25519 signing key, X25519 key exchange key, and Hades ID.
    pub fn derive_messaging_keypair(&self) -> Result<MessagingKeypair, SeedError> {
        use bip32::{DerivationPath, XPrv};
        use std::str::FromStr;

        let path = DerivationPath::from_str("m/13'/0'/0'")
            .map_err(|e| SeedError::Derivation(e.to_string()))?;

        let child = XPrv::derive_from_path(&self.seed, &path)
            .map_err(|e| SeedError::Derivation(e.to_string()))?;

        let secret_bytes = child.to_bytes();

        // Use first 32 bytes as Ed25519 seed
        let mut ed25519_seed = [0u8; 32];
        ed25519_seed.copy_from_slice(&secret_bytes[..32]);

        let signing_key = ed25519_dalek::SigningKey::from_bytes(&ed25519_seed);
        let verifying_key = signing_key.verifying_key();

        // Derive X25519 key from Ed25519 key (standard conversion)
        // SHA-512 the Ed25519 seed, take first 32 bytes, clamp
        let hash = sha2::Sha512::digest(&ed25519_seed);
        let mut x25519_secret = [0u8; 32];
        x25519_secret.copy_from_slice(&hash[..32]);
        // Clamp per curve25519 spec
        x25519_secret[0] &= 248;
        x25519_secret[31] &= 127;
        x25519_secret[31] |= 64;

        let x25519_sk = x25519_dalek::StaticSecret::from(x25519_secret);
        let x25519_pk = x25519_dalek::PublicKey::from(&x25519_sk);

        // Compute Hades ID = blake3(ed25519_public_key)
        let hades_id = blake3::hash(verifying_key.as_bytes());

        // Zeroize intermediate secrets
        ed25519_seed.zeroize();
        x25519_secret.zeroize();

        Ok(MessagingKeypair {
            ed25519_signing: signing_key,
            ed25519_public: verifying_key,
            x25519_secret: x25519_sk,
            x25519_public: x25519_pk,
            hades_id: hades_id.as_bytes().to_vec(),
        })
    }

    /// Derive wallet HD key for a specific chain.
    ///
    /// Path: `m/44'/{coin_type}'/{account}'/0/0`
    pub fn derive_wallet_key(
        &self,
        coin_type: u32,
        account: u32,
    ) -> Result<WalletDerivedKey, SeedError> {
        use bip32::{DerivationPath, XPrv};
        use std::str::FromStr;

        let path_str = format!("m/44'/{}'/{}'/0/0", coin_type, account);
        let path = DerivationPath::from_str(&path_str)
            .map_err(|e| SeedError::Derivation(e.to_string()))?;

        let child = XPrv::derive_from_path(&self.seed, &path)
            .map_err(|e| SeedError::Derivation(e.to_string()))?;

        Ok(WalletDerivedKey {
            secret_bytes: child.to_bytes().to_vec(),
            derivation_path: path_str,
        })
    }
}

use sha2::Digest;

/// Messaging keypair (Ed25519 for signing + X25519 for DH key exchange)
pub struct MessagingKeypair {
    pub ed25519_signing: ed25519_dalek::SigningKey,
    pub ed25519_public: ed25519_dalek::VerifyingKey,
    pub x25519_secret: x25519_dalek::StaticSecret,
    pub x25519_public: x25519_dalek::PublicKey,
    pub hades_id: Vec<u8>,
}

impl MessagingKeypair {
    /// The Hades ID displayed as hex. Example: `7f3a9b2c...`
    pub fn hades_id_hex(&self) -> String {
        hex::encode(&self.hades_id)
    }

    /// Short display ID: first 4 + last 4 bytes. Example: `7f3a9b2c...deadbeef`
    pub fn hades_id_short(&self) -> String {
        let hex = self.hades_id_hex();
        if hex.len() >= 16 {
            format!("{}...{}", &hex[..8], &hex[hex.len() - 8..])
        } else {
            hex
        }
    }

    /// Sign arbitrary data with Ed25519
    pub fn sign(&self, data: &[u8]) -> Vec<u8> {
        use ed25519_dalek::Signer;
        self.ed25519_signing.sign(data).to_bytes().to_vec()
    }

    /// Verify a signature against our public key
    pub fn verify(&self, data: &[u8], signature: &[u8]) -> bool {
        use ed25519_dalek::Verifier;
        if signature.len() != 64 {
            return false;
        }
        let sig = match ed25519_dalek::Signature::from_slice(signature) {
            Ok(s) => s,
            Err(_) => return false,
        };
        self.ed25519_public.verify(data, &sig).is_ok()
    }

    /// Compute safety number with a contact (Signal-style, for verification).
    ///
    /// The safety number is deterministic and identical regardless of which
    /// party computes it (keys are sorted before hashing).
    pub fn safety_number_with(&self, their_ed25519_public: &[u8; 32]) -> String {
        let mut input = Vec::new();
        let our_bytes = self.ed25519_public.as_bytes();

        // Sort keys for deterministic ordering
        if our_bytes < their_ed25519_public {
            input.extend_from_slice(our_bytes);
            input.extend_from_slice(their_ed25519_public);
        } else {
            input.extend_from_slice(their_ed25519_public);
            input.extend_from_slice(our_bytes);
        }

        let hash = blake3::hash(&input);
        let bytes = hash.as_bytes();

        // Format as groups of 5 digits (Signal-style)
        let mut number = String::new();
        for chunk in bytes.chunks(5) {
            let val = u64::from_be_bytes({
                let mut arr = [0u8; 8];
                let start = 8 - chunk.len();
                arr[start..].copy_from_slice(chunk);
                arr
            }) % 100000;
            if !number.is_empty() {
                number.push(' ');
            }
            number.push_str(&format!("{:05}", val));
        }
        number
    }
}

/// Derived wallet key with zeroization
#[derive(ZeroizeOnDrop)]
pub struct WalletDerivedKey {
    pub secret_bytes: Vec<u8>,
    #[zeroize(skip)]
    pub derivation_path: String,
}

#[derive(Debug, thiserror::Error)]
pub enum SeedError {
    #[error("Seed generation failed: {0}")]
    Generation(String),
    #[error("Invalid mnemonic: {0}")]
    InvalidMnemonic(String),
    #[error("Key derivation failed: {0}")]
    Derivation(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_derivation() {
        let seed1 = MasterSeed::generate().unwrap();
        let phrase = seed1.mnemonic().to_string();

        let seed2 = MasterSeed::from_mnemonic(&phrase).unwrap();

        let kp1 = seed1.derive_messaging_keypair().unwrap();
        let kp2 = seed2.derive_messaging_keypair().unwrap();

        // Same seed → same identity
        assert_eq!(kp1.hades_id, kp2.hades_id);
        assert_eq!(
            kp1.ed25519_public.as_bytes(),
            kp2.ed25519_public.as_bytes()
        );
        assert_eq!(kp1.x25519_public.as_bytes(), kp2.x25519_public.as_bytes());
    }

    #[test]
    fn test_messaging_and_wallet_keys_different() {
        let seed = MasterSeed::generate().unwrap();

        let messaging = seed.derive_messaging_keypair().unwrap();
        let wallet_eth = seed.derive_wallet_key(60, 0).unwrap();

        // Messaging key and wallet key must be different
        assert_ne!(
            messaging.ed25519_public.as_bytes().to_vec(),
            wallet_eth.secret_bytes
        );
    }

    #[test]
    fn test_sign_verify() {
        let seed = MasterSeed::generate().unwrap();
        let kp = seed.derive_messaging_keypair().unwrap();

        let msg = b"Hello Hades";
        let sig = kp.sign(msg);
        assert!(kp.verify(msg, &sig));
        assert!(!kp.verify(b"tampered", &sig));
    }

    #[test]
    fn test_safety_number_symmetric() {
        let seed1 = MasterSeed::generate().unwrap();
        let seed2 = MasterSeed::generate().unwrap();

        let kp1 = seed1.derive_messaging_keypair().unwrap();
        let kp2 = seed2.derive_messaging_keypair().unwrap();

        let sn1 = kp1.safety_number_with(kp2.ed25519_public.as_bytes());
        let sn2 = kp2.safety_number_with(kp1.ed25519_public.as_bytes());

        // Safety number must be the same from both sides
        assert_eq!(sn1, sn2);
    }

    #[test]
    fn test_hades_id_format() {
        let seed = MasterSeed::generate().unwrap();
        let kp = seed.derive_messaging_keypair().unwrap();

        let full = kp.hades_id_hex();
        let short = kp.hades_id_short();

        assert_eq!(full.len(), 64); // 32 bytes = 64 hex chars
        assert!(short.contains("..."));
    }

    #[test]
    fn test_24_word_mnemonic() {
        let seed = MasterSeed::generate().unwrap();
        let words: Vec<&str> = seed.mnemonic().split_whitespace().collect();
        assert_eq!(words.len(), 24);
    }
}
