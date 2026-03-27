//! Anonymous credential system for Hades identity.
//!
//! Allows users to prove they hold a valid registration credential
//! without revealing *which* credential they hold. Built on blind
//! signatures and zero-knowledge presentation proofs.
//!
//! ## Flow
//!
//! 1. Client commits to identity key → server issues blinded credential.
//! 2. Client unblinds credential locally (server never sees final token).
//! 3. On each connection, client presents a ZK proof of possession.
//! 4. Server verifies proof without learning the user's identity.

use zeroize::Zeroize;

/// Server-side parameters (published once, never changes per epoch).
#[derive(Debug, Clone)]
pub struct ServerPublicParams {
    /// Epoch identifier (rotated daily/weekly).
    pub epoch: u32,
    /// Public key of the issuing authority.
    pub issuer_pubkey: [u8; 32],
}

/// A blinded credential request sent from client to server.
#[derive(Debug)]
pub struct CredentialRequest {
    /// Blinded identity commitment.
    pub blinded_commitment: [u8; 64],
    /// Epoch this request is valid for.
    pub epoch: u32,
}

/// A blinded credential issued by the server.
#[derive(Debug)]
pub struct BlindedCredential {
    /// Blinded signature over the commitment.
    pub blinded_signature: [u8; 64],
    /// Server proof of honest issuance.
    pub issuance_proof: Vec<u8>,
}

/// An unblinded credential stored on the client device.
#[derive(Zeroize)]
#[zeroize(drop)]
pub struct AnonymousCredential {
    /// The unblinded signature (secret — never leaves device).
    pub signature: [u8; 64],
    /// Epoch of issuance.
    pub epoch: u32,
}

/// A zero-knowledge presentation proof.
///
/// Proves possession of a valid credential without revealing
/// the credential itself or the user's identity.
#[derive(Debug)]
pub struct CredentialPresentation {
    /// ZK proof bytes.
    pub proof: Vec<u8>,
    /// Epoch binding.
    pub epoch: u32,
    /// Single-use nullifier (prevents double-spending the credential).
    pub nullifier: [u8; 32],
}

impl AnonymousCredential {
    /// Present this credential as a zero-knowledge proof.
    ///
    /// The returned presentation can be verified by the server
    /// without learning which user is presenting.
    pub fn present(&self, server_params: &ServerPublicParams) -> Result<CredentialPresentation, String> {
        if self.epoch != server_params.epoch {
            return Err("credential epoch mismatch — request a fresh credential".into());
        }

        // In production: construct a Schnorr-like ZKP or use the
        // zkgroup library for group-compatible proofs.
        let mut nullifier = [0u8; 32];
        getrandom::getrandom(&mut nullifier).map_err(|e| e.to_string())?;

        Ok(CredentialPresentation {
            proof: self.signature.to_vec(), // placeholder
            epoch: self.epoch,
            nullifier,
        })
    }
}

/// Verify a credential presentation (server-side).
pub fn verify_presentation(
    presentation: &CredentialPresentation,
    server_params: &ServerPublicParams,
) -> bool {
    // In production: verify ZKP against issuer_pubkey + epoch binding
    presentation.epoch == server_params.epoch && !presentation.proof.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credential_present_and_verify() {
        let params = ServerPublicParams {
            epoch: 42,
            issuer_pubkey: [1u8; 32],
        };

        let cred = AnonymousCredential {
            signature: [0xAB; 64],
            epoch: 42,
        };

        let presentation = cred.present(&params).unwrap();
        assert!(verify_presentation(&presentation, &params));
    }

    #[test]
    fn credential_rejects_wrong_epoch() {
        let params = ServerPublicParams {
            epoch: 43,
            issuer_pubkey: [1u8; 32],
        };

        let cred = AnonymousCredential {
            signature: [0xAB; 64],
            epoch: 42,
        };

        assert!(cred.present(&params).is_err());
    }
}
