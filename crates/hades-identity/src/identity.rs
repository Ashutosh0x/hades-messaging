use ed25519_dalek::{Signer, Signature, Verifier, VerifyingKey};
use zeroize::ZeroizeOnDrop;

use hades_crypto::pqxdh::IdentityKeyPair;
use crate::error::IdentityError;

/// An identity of a Hades user, tied to a long-lived Ed25519 key.
#[derive(ZeroizeOnDrop)]
pub struct Identity {
    pub keypair: IdentityKeyPair,
}

impl Identity {
    pub fn generate() -> Self {
        Self {
            keypair: IdentityKeyPair::generate(),
        }
    }

    pub fn public_key(&self) -> VerifyingKey {
        self.keypair.verifying_key()
    }
    
    pub fn sign(&self, message: &[u8]) -> Signature {
        self.keypair.signing_key.sign(message)
    }
}

/// A public identity for a Hades user.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicIdentity {
    pub key: VerifyingKey,
}

impl PublicIdentity {
    pub fn new(key: VerifyingKey) -> Self {
        Self { key }
    }

    pub fn verify(&self, message: &[u8], signature: &Signature) -> Result<(), IdentityError> {
        self.key.verify(message, signature).map_err(|_| IdentityError::InvalidSignature)
    }
}
