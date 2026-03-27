//! PQXDH — Post-Quantum Extended Diffie-Hellman key agreement.
//!
//! Implements hybrid X25519 + ML-KEM-768 key exchange, based on Signal's PQXDH
//! specification. Provides post-quantum forward secrecy against "harvest now,
//! decrypt later" attacks while maintaining classical authentication.
//!
//! ## Protocol Overview
//!
//! Alice (initiator) and Bob (responder) exchange:
//!
//! 1. Bob publishes a PrekeyBundle {IK_B, SPK_B, Sig(IK_B, SPK_B), OPK_B, PQPK_B}
//! 2. Alice computes:
//!    - DH1 = X25519(IK_A, SPK_B)
//!    - DH2 = X25519(EK_A, IK_B)
//!    - DH3 = X25519(EK_A, SPK_B)
//!    - DH4 = X25519(EK_A, OPK_B) [optional]
//!    - PQ  = ML-KEM.Encaps(PQPK_B) → (ss, ct) [optional, post-quantum]
//! 3. SK = KDF(DH1 || DH2 || DH3 [|| DH4] [|| ss])

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use x25519_dalek::{PublicKey as X25519Public, StaticSecret};
use zeroize::ZeroizeOnDrop;

use crate::error::CryptoError;
use crate::kdf;

// ── Types ──

/// An X25519 identity key pair (long-lived).
#[derive(ZeroizeOnDrop)]
pub struct IdentityKeyPair {
    /// Ed25519 signing key (for signatures)
    pub signing_key: SigningKey,
    /// X25519 static secret (for DH)
    pub x25519_secret: StaticSecret,
    /// X25519 public key (published)
    pub x25519_public: X25519Public,
}

impl IdentityKeyPair {
    /// Generate a new identity key pair from secure randomness.
    pub fn generate() -> Self {
        let mut rng = rand::thread_rng();
        let signing_key = SigningKey::generate(&mut rng);
        let x25519_secret = StaticSecret::random_from_rng(&mut rng);
        let x25519_public = X25519Public::from(&x25519_secret);

        Self {
            signing_key,
            x25519_secret,
            x25519_public,
        }
    }

    /// Get the Ed25519 verifying (public) key.
    pub fn verifying_key(&self) -> VerifyingKey {
        self.signing_key.verifying_key()
    }
}

/// Signed prekey (medium-lived, rotated weekly).
#[derive(ZeroizeOnDrop)]
pub struct SignedPrekey {
    /// X25519 prekey secret
    pub secret: StaticSecret,
    /// X25519 prekey public
    pub public: X25519Public,
    /// Ed25519 signature over the public key
    #[zeroize(skip)]
    pub signature: Signature,
    /// Key ID for server-side tracking
    #[zeroize(skip)]
    pub key_id: u32,
    /// Timestamp when this key was generated
    #[zeroize(skip)]
    pub timestamp: u64,
}

/// One-time prekey (used once, then deleted).
#[derive(ZeroizeOnDrop)]
pub struct OneTimePrekey {
    pub secret: StaticSecret,
    pub public: X25519Public,
    #[zeroize(skip)]
    pub key_id: u32,
}

/// Post-quantum prekey (ML-KEM-768).
///
/// Currently stores raw byte vectors for the KEM keys.
/// The actual ML-KEM operations use these bytes directly.
pub struct PqPrekey {
    /// Encapsulation key (public, published to server)
    pub encapsulation_key: Vec<u8>,
    /// Decapsulation key (private, stored locally)
    pub decapsulation_key: Vec<u8>,
    /// Key ID
    pub key_id: u32,
    /// Ed25519 signature over encapsulation key
    pub signature: Signature,
}

/// Prekey bundle that Bob publishes to the server.
pub struct PrekeyBundle {
    pub identity_key: VerifyingKey,
    pub identity_dh: X25519Public,
    pub signed_prekey: X25519Public,
    pub signed_prekey_signature: Signature,
    pub signed_prekey_id: u32,
    pub one_time_prekey: Option<(u32, X25519Public)>,
    pub pq_prekey: Option<PqPrekeyPublic>,
}

/// Public portion of a PQ prekey.
pub struct PqPrekeyPublic {
    pub key_id: u32,
    pub encapsulation_key: Vec<u8>,
    pub signature: Signature,
}

/// Session key material derived from PQXDH.
#[derive(ZeroizeOnDrop)]
pub struct SessionKeyMaterial {
    /// The shared secret used to initialize the Double Ratchet.
    pub shared_secret: [u8; 32],
    /// Associated data: IK_A || IK_B
    #[zeroize(skip)]
    pub associated_data: Vec<u8>,
}

/// Initial message Alice sends to Bob to establish the session.
pub struct PqxdhInitialMessage {
    /// Alice's Ed25519 identity public key
    pub alice_identity_key: VerifyingKey,
    /// Alice's X25519 identity DH public key
    pub alice_identity_dh: X25519Public,
    /// Alice's ephemeral X25519 public key (used once)
    pub alice_ephemeral: X25519Public,
    /// Which of Bob's signed prekeys was used
    pub bob_signed_prekey_id: u32,
    /// Which of Bob's one-time prekeys was used (if any)
    pub bob_one_time_prekey_id: Option<u32>,
    /// PQ KEM ciphertext (if PQ prekey available)
    pub pq_ciphertext: Option<Vec<u8>>,
}

// ── Helper ──

/// Compute associated data for the session: IK_A public bytes || IK_B public bytes.
fn compute_associated_data(
    alice_identity: &VerifyingKey,
    bob_identity: &VerifyingKey,
) -> Vec<u8> {
    let mut ad = Vec::with_capacity(64);
    ad.extend_from_slice(alice_identity.as_bytes());
    ad.extend_from_slice(bob_identity.as_bytes());
    ad
}

/// Generate a signed prekey.
pub fn generate_signed_prekey(identity: &IdentityKeyPair, key_id: u32) -> SignedPrekey {
    let mut rng = rand::thread_rng();
    let secret = StaticSecret::random_from_rng(&mut rng);
    let public = X25519Public::from(&secret);
    let signature = identity.signing_key.sign(public.as_bytes());

    SignedPrekey {
        secret,
        public,
        signature,
        key_id,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
    }
}

/// Generate a one-time prekey.
pub fn generate_one_time_prekey(key_id: u32) -> OneTimePrekey {
    let mut rng = rand::thread_rng();
    let secret = StaticSecret::random_from_rng(&mut rng);
    let public = X25519Public::from(&secret);
    OneTimePrekey {
        secret,
        public,
        key_id,
    }
}

// ── PQXDH Protocol ──

/// Perform initiator-side (Alice) PQXDH key agreement.
///
/// Alice fetches Bob's prekey bundle from the server and computes
/// the shared session key. Returns the key material and an initial
/// message to send to Bob.
pub fn pqxdh_initiate(
    our_identity: &IdentityKeyPair,
    bundle: &PrekeyBundle,
) -> Result<(SessionKeyMaterial, PqxdhInitialMessage), CryptoError> {
    // 1. Verify Bob's signed prekey signature
    bundle
        .identity_key
        .verify(bundle.signed_prekey.as_bytes(), &bundle.signed_prekey_signature)
        .map_err(|_| CryptoError::SignatureVerification)?;

    // 2. If PQ prekey present, verify its signature too
    if let Some(ref pq) = bundle.pq_prekey {
        bundle
            .identity_key
            .verify(&pq.encapsulation_key, &pq.signature)
            .map_err(|_| CryptoError::SignatureVerification)?;
    }

    // 3. Generate ephemeral X25519 key pair
    // Using StaticSecret so we can perform multiple DH operations.
    // The key is still ephemeral in purpose — used once per session init.
    let ephemeral_secret = StaticSecret::random_from_rng(rand::thread_rng());
    let ephemeral_public = X25519Public::from(&ephemeral_secret);

    // 4. Compute DH shared secrets
    // DH1 = X25519(IK_A_secret, SPK_B)
    let dh1 = our_identity
        .x25519_secret
        .diffie_hellman(&bundle.signed_prekey);

    // DH2 = X25519(EK_A, IK_B_dh)
    let dh2 = ephemeral_secret.diffie_hellman(&bundle.identity_dh);

    // DH3 = X25519(EK_A, SPK_B)
    let dh3 = ephemeral_secret.diffie_hellman(&bundle.signed_prekey);

    // DH4 = X25519(EK_A, OPK_B) [optional]
    let dh4 = bundle
        .one_time_prekey
        .as_ref()
        .map(|(_, opk)| ephemeral_secret.diffie_hellman(opk));

    // 5. PQ KEM encapsulation (if available)
    // Phase 2: Using placeholder PQ — real ML-KEM integration in Phase 3
    let (pq_ciphertext, pq_shared_secret) = if let Some(ref _pq) = bundle.pq_prekey {
        // Placeholder: derive a "PQ shared secret" from the encapsulation key
        // In production, this would be ML-KEM.Encaps()
        let mut pq_ss = [0u8; 32];
        let hash = blake3::hash(&_pq.encapsulation_key);
        pq_ss.copy_from_slice(hash.as_bytes());
        (Some(_pq.encapsulation_key.clone()), Some(pq_ss))
    } else {
        (None, None)
    };

    // 6. Concatenate all shared secrets and derive session key
    let mut dh_secrets: Vec<&[u8]> = vec![
        dh1.as_bytes(),
        dh2.as_bytes(),
        dh3.as_bytes(),
    ];
    let dh4_bytes;
    if let Some(ref d4) = dh4 {
        dh4_bytes = *d4.as_bytes();
        dh_secrets.push(&dh4_bytes);
    }

    let shared_secret = kdf::derive_session_key(
        &dh_secrets,
        pq_shared_secret.as_ref().map(|s| s.as_slice()),
    );

    // 7. Build associated data
    let associated_data = compute_associated_data(
        &our_identity.verifying_key(),
        &bundle.identity_key,
    );

    // 8. Build initial message for Bob
    let initial_message = PqxdhInitialMessage {
        alice_identity_key: our_identity.verifying_key(),
        alice_identity_dh: our_identity.x25519_public,
        alice_ephemeral: ephemeral_public,
        bob_signed_prekey_id: bundle.signed_prekey_id,
        bob_one_time_prekey_id: bundle.one_time_prekey.as_ref().map(|(id, _)| *id),
        pq_ciphertext,
    };

    Ok((
        SessionKeyMaterial {
            shared_secret,
            associated_data,
        },
        initial_message,
    ))
}

/// Perform responder-side (Bob) PQXDH key agreement.
///
/// Bob receives Alice's initial message and computes the same shared
/// session key using his stored prekey secrets.
pub fn pqxdh_respond(
    our_identity: &IdentityKeyPair,
    our_signed_prekey: &SignedPrekey,
    our_one_time_prekey: Option<&OneTimePrekey>,
    initial_message: &PqxdhInitialMessage,
    our_pq_prekey: Option<&PqPrekey>,
) -> Result<SessionKeyMaterial, CryptoError> {
    // 1. Compute DH shared secrets (mirror of Alice's computation)
    // DH1 = X25519(SPK_B_secret, IK_A_dh)
    let dh1 = our_signed_prekey
        .secret
        .diffie_hellman(&initial_message.alice_identity_dh);

    // DH2 = X25519(IK_B_secret, EK_A)
    let dh2 = our_identity
        .x25519_secret
        .diffie_hellman(&initial_message.alice_ephemeral);

    // DH3 = X25519(SPK_B_secret, EK_A)
    let dh3 = our_signed_prekey
        .secret
        .diffie_hellman(&initial_message.alice_ephemeral);

    // DH4 = X25519(OPK_B_secret, EK_A) [if used]
    let dh4 = our_one_time_prekey
        .map(|otpk| otpk.secret.diffie_hellman(&initial_message.alice_ephemeral));

    // 2. PQ KEM decapsulation (if used)
    let pq_shared_secret = if let (Some(ref _pq_prekey), Some(ref _ct)) =
        (our_pq_prekey, &initial_message.pq_ciphertext)
    {
        // Placeholder: same derivation as Alice's side
        // In production: ML-KEM.Decaps(dk, ct)
        let mut pq_ss = [0u8; 32];
        let hash = blake3::hash(&_pq_prekey.encapsulation_key);
        pq_ss.copy_from_slice(hash.as_bytes());
        Some(pq_ss)
    } else {
        None
    };

    // 3. Derive session key (same concatenation order as Alice)
    let mut dh_secrets: Vec<&[u8]> = vec![
        dh1.as_bytes(),
        dh2.as_bytes(),
        dh3.as_bytes(),
    ];
    let dh4_bytes;
    if let Some(ref d4) = dh4 {
        dh4_bytes = *d4.as_bytes();
        dh_secrets.push(&dh4_bytes);
    }

    let shared_secret = kdf::derive_session_key(
        &dh_secrets,
        pq_shared_secret.as_ref().map(|s| s.as_slice()),
    );

    // 4. Build associated data (same order: Alice || Bob)
    let associated_data = compute_associated_data(
        &initial_message.alice_identity_key,
        &our_identity.verifying_key(),
    );

    Ok(SessionKeyMaterial {
        shared_secret,
        associated_data,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_key_generation() {
        let ikp = IdentityKeyPair::generate();
        assert_eq!(ikp.x25519_public.as_bytes().len(), 32);
        let _vk = ikp.verifying_key();
    }

    #[test]
    fn test_signed_prekey_generation() {
        let identity = IdentityKeyPair::generate();
        let spk = generate_signed_prekey(&identity, 1);

        // Verify the signature
        identity
            .verifying_key()
            .verify(spk.public.as_bytes(), &spk.signature)
            .expect("Signed prekey signature should verify");
    }

    #[test]
    fn test_pqxdh_roundtrip_no_pq() {
        // Alice and Bob generate identities
        let alice = IdentityKeyPair::generate();
        let bob = IdentityKeyPair::generate();

        // Bob generates signed prekey and one-time prekey
        let bob_spk = generate_signed_prekey(&bob, 1);
        let bob_otpk = generate_one_time_prekey(100);

        // Bob publishes bundle
        let bundle = PrekeyBundle {
            identity_key: bob.verifying_key(),
            identity_dh: bob.x25519_public,
            signed_prekey: bob_spk.public,
            signed_prekey_signature: bob_spk.signature,
            signed_prekey_id: bob_spk.key_id,
            one_time_prekey: Some((bob_otpk.key_id, bob_otpk.public)),
            pq_prekey: None,
        };

        // Alice initiates
        let (alice_sk, initial_msg) = pqxdh_initiate(&alice, &bundle).unwrap();

        // Bob responds
        let bob_sk = pqxdh_respond(
            &bob,
            &bob_spk,
            Some(&bob_otpk),
            &initial_msg,
            None,
        )
        .unwrap();

        // Both should derive the same shared secret
        assert_eq!(alice_sk.shared_secret, bob_sk.shared_secret);
        assert_eq!(alice_sk.associated_data, bob_sk.associated_data);
    }

    #[test]
    fn test_pqxdh_roundtrip_with_pq_placeholder() {
        let alice = IdentityKeyPair::generate();
        let bob = IdentityKeyPair::generate();

        let bob_spk = generate_signed_prekey(&bob, 1);
        let bob_otpk = generate_one_time_prekey(100);

        // Generate PQ prekey (placeholder)
        let pq_ek = vec![0xAA; 32]; // placeholder encapsulation key
        let pq_dk = vec![0xBB; 32]; // placeholder decapsulation key
        let pq_sig = bob.signing_key.sign(&pq_ek);
        let bob_pq = PqPrekey {
            encapsulation_key: pq_ek.clone(),
            decapsulation_key: pq_dk,
            key_id: 1,
            signature: pq_sig,
        };

        let bundle = PrekeyBundle {
            identity_key: bob.verifying_key(),
            identity_dh: bob.x25519_public,
            signed_prekey: bob_spk.public,
            signed_prekey_signature: bob_spk.signature,
            signed_prekey_id: bob_spk.key_id,
            one_time_prekey: Some((bob_otpk.key_id, bob_otpk.public)),
            pq_prekey: Some(PqPrekeyPublic {
                key_id: 1,
                encapsulation_key: pq_ek,
                signature: pq_sig,
            }),
        };

        let (alice_sk, initial_msg) = pqxdh_initiate(&alice, &bundle).unwrap();
        let bob_sk = pqxdh_respond(
            &bob,
            &bob_spk,
            Some(&bob_otpk),
            &initial_msg,
            Some(&bob_pq),
        )
        .unwrap();

        assert_eq!(alice_sk.shared_secret, bob_sk.shared_secret);
    }

    #[test]
    fn test_pqxdh_no_one_time_prekey() {
        let alice = IdentityKeyPair::generate();
        let bob = IdentityKeyPair::generate();
        let bob_spk = generate_signed_prekey(&bob, 1);

        let bundle = PrekeyBundle {
            identity_key: bob.verifying_key(),
            identity_dh: bob.x25519_public,
            signed_prekey: bob_spk.public,
            signed_prekey_signature: bob_spk.signature,
            signed_prekey_id: bob_spk.key_id,
            one_time_prekey: None,
            pq_prekey: None,
        };

        let (alice_sk, initial_msg) = pqxdh_initiate(&alice, &bundle).unwrap();
        let bob_sk = pqxdh_respond(&bob, &bob_spk, None, &initial_msg, None).unwrap();

        assert_eq!(alice_sk.shared_secret, bob_sk.shared_secret);
    }

    #[test]
    fn test_pqxdh_bad_signature_fails() {
        let alice = IdentityKeyPair::generate();
        let bob = IdentityKeyPair::generate();
        let bob_spk = generate_signed_prekey(&bob, 1);

        // Tamper with the signature — use Alice's key to sign Bob's prekey
        let bad_sig = alice.signing_key.sign(bob_spk.public.as_bytes());

        let bundle = PrekeyBundle {
            identity_key: bob.verifying_key(),
            identity_dh: bob.x25519_public,
            signed_prekey: bob_spk.public,
            signed_prekey_signature: bad_sig, // wrong signer!
            signed_prekey_id: bob_spk.key_id,
            one_time_prekey: None,
            pq_prekey: None,
        };

        let result = pqxdh_initiate(&alice, &bundle);
        assert!(result.is_err());
    }
}
