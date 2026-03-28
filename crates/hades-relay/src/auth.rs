//! Relay Authentication — Challenge-Response Protocol
//!
//! The relay authenticates clients via Ed25519 challenge-response:
//! 1. Client sends public key
//! 2. Server sends random 32-byte nonce
//! 3. Client signs nonce with Ed25519 private key
//! 4. Server verifies signature → authenticated
//!
//! No passwords, no tokens, no accounts — just cryptographic proof of key possession.

use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

const CHALLENGE_EXPIRY_SECS: u64 = 30;
const CHALLENGE_SIZE: usize = 32;

/// Challenge issued to a connecting client
#[derive(Debug, Clone)]
struct PendingChallenge {
    nonce: [u8; CHALLENGE_SIZE],
    issued_at: Instant,
    client_addr: String,
}

/// Authenticated session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticatedIdentity {
    pub hades_id: String,
    pub ed25519_public_key: [u8; 32],
    pub x25519_public_key: Option<[u8; 32]>,
    pub authenticated_at: u64,
}

/// Server-side authenticator
pub struct RelayAuthenticator {
    pending_challenges: Arc<RwLock<HashMap<String, PendingChallenge>>>,
    authenticated: Arc<RwLock<HashMap<String, AuthenticatedIdentity>>>,
    auth_attempts: Arc<RwLock<HashMap<String, (u32, Instant)>>>,
}

/// Messages exchanged during the auth handshake
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AuthMessage {
    #[serde(rename = "auth_request")]
    AuthRequest {
        ed25519_public_key: String,
    },

    #[serde(rename = "auth_challenge")]
    AuthChallenge {
        nonce: String,
        server_time: u64,
    },

    #[serde(rename = "auth_response")]
    AuthResponse {
        ed25519_public_key: String,
        signature: String,
        x25519_public_key: Option<String>,
        prekey_bundle: Option<PrekeyBundleUpload>,
        profile: Option<PublicProfile>,
    },

    #[serde(rename = "auth_success")]
    AuthSuccess {
        hades_id: String,
        session_token: String,
        queued_messages: u32,
    },

    #[serde(rename = "auth_failed")]
    AuthFailed {
        reason: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrekeyBundleUpload {
    pub signed_prekey: String,
    pub signed_prekey_signature: String,
    pub one_time_prekeys: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicProfile {
    pub display_name: Option<String>,
    pub avatar_hash: Option<String>,
}

impl RelayAuthenticator {
    pub fn new() -> Self {
        Self {
            pending_challenges: Arc::new(RwLock::new(HashMap::new())),
            authenticated: Arc::new(RwLock::new(HashMap::new())),
            auth_attempts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Step 1: Generate a challenge for a client
    pub async fn create_challenge(
        &self,
        client_addr: &str,
    ) -> Result<AuthMessage, AuthError> {
        // Rate limit: max 10 auth attempts per minute per IP
        {
            let mut attempts = self.auth_attempts.write().await;
            let entry = attempts
                .entry(client_addr.to_string())
                .or_insert((0, Instant::now()));

            if entry.1.elapsed() > Duration::from_secs(60) {
                *entry = (0, Instant::now());
            }

            entry.0 += 1;
            if entry.0 > 10 {
                return Err(AuthError::RateLimited);
            }
        }

        // Generate random challenge nonce
        let mut nonce = [0u8; CHALLENGE_SIZE];
        getrandom::getrandom(&mut nonce)
            .map_err(|_| AuthError::Internal("RNG failed".into()))?;

        let challenge = PendingChallenge {
            nonce,
            issued_at: Instant::now(),
            client_addr: client_addr.to_string(),
        };

        // Store pending challenge keyed by client address
        {
            let mut pending = self.pending_challenges.write().await;
            pending.insert(client_addr.to_string(), challenge);
        }

        let server_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Ok(AuthMessage::AuthChallenge {
            nonce: hex::encode(nonce),
            server_time,
        })
    }

    /// Step 2: Verify the client's signed challenge
    pub async fn verify_response(
        &self,
        client_addr: &str,
        response: &AuthMessage,
    ) -> Result<AuthenticatedIdentity, AuthError> {
        let (pubkey_hex, signature_hex, x25519_hex, _prekeys, _profile) = match response {
            AuthMessage::AuthResponse {
                ed25519_public_key,
                signature,
                x25519_public_key,
                prekey_bundle,
                profile,
            } => (
                ed25519_public_key,
                signature,
                x25519_public_key,
                prekey_bundle,
                profile,
            ),
            _ => return Err(AuthError::InvalidMessage),
        };

        // Retrieve pending challenge
        let challenge = {
            let mut pending = self.pending_challenges.write().await;
            pending
                .remove(client_addr)
                .ok_or(AuthError::NoPendingChallenge)?
        };

        // Check challenge expiry
        if challenge.issued_at.elapsed() > Duration::from_secs(CHALLENGE_EXPIRY_SECS) {
            return Err(AuthError::ChallengeExpired);
        }

        // Parse public key
        let pubkey_bytes = hex::decode(pubkey_hex)
            .map_err(|_| AuthError::InvalidKey("Bad hex".into()))?;

        if pubkey_bytes.len() != 32 {
            return Err(AuthError::InvalidKey(
                "Ed25519 key must be 32 bytes".into(),
            ));
        }

        let mut pk_arr = [0u8; 32];
        pk_arr.copy_from_slice(&pubkey_bytes);

        let verifying_key = VerifyingKey::from_bytes(&pk_arr)
            .map_err(|e| AuthError::InvalidKey(e.to_string()))?;

        // Parse signature
        let sig_bytes = hex::decode(signature_hex)
            .map_err(|_| AuthError::InvalidSignature("Bad hex".into()))?;

        if sig_bytes.len() != 64 {
            return Err(AuthError::InvalidSignature(
                "Signature must be 64 bytes".into(),
            ));
        }

        let signature = Signature::from_slice(&sig_bytes)
            .map_err(|e| AuthError::InvalidSignature(e.to_string()))?;

        // THE CRITICAL CHECK: Verify signature of the challenge nonce
        verifying_key
            .verify(&challenge.nonce, &signature)
            .map_err(|_| AuthError::SignatureVerificationFailed)?;

        // Authentication succeeded — compute Hades ID
        let hades_id = blake3::hash(&pk_arr);
        let hades_id_hex = hex::encode(hades_id.as_bytes());

        // Parse X25519 key if provided
        let x25519_pk = if let Some(ref x_hex) = x25519_hex {
            let bytes = hex::decode(x_hex).ok();
            bytes.and_then(|b| {
                if b.len() == 32 {
                    let mut arr = [0u8; 32];
                    arr.copy_from_slice(&b);
                    Some(arr)
                } else {
                    None
                }
            })
        } else {
            None
        };

        let authenticated = AuthenticatedIdentity {
            hades_id: hades_id_hex.clone(),
            ed25519_public_key: pk_arr,
            x25519_public_key: x25519_pk,
            authenticated_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };

        // Store authenticated session
        {
            let mut auth = self.authenticated.write().await;
            auth.insert(hades_id_hex.clone(), authenticated.clone());
        }

        tracing::info!(
            "Authenticated: hades:{}...{} from {}",
            &hades_id_hex[..8],
            &hades_id_hex[hades_id_hex.len() - 8..],
            client_addr
        );

        Ok(authenticated)
    }

    /// Check if a Hades ID is currently authenticated
    pub async fn is_authenticated(&self, hades_id: &str) -> bool {
        let auth = self.authenticated.read().await;
        auth.contains_key(hades_id)
    }

    /// Remove authentication on disconnect
    pub async fn deauthenticate(&self, hades_id: &str) {
        let mut auth = self.authenticated.write().await;
        auth.remove(hades_id);
    }

    /// Periodic cleanup of expired challenges and old rate-limit entries
    pub async fn cleanup(&self) {
        let mut pending = self.pending_challenges.write().await;
        pending.retain(|_, c| {
            c.issued_at.elapsed() < Duration::from_secs(CHALLENGE_EXPIRY_SECS * 2)
        });

        let mut attempts = self.auth_attempts.write().await;
        attempts.retain(|_, (_, t)| t.elapsed() < Duration::from_secs(300));
    }
}

impl Default for RelayAuthenticator {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Rate limited: too many authentication attempts")]
    RateLimited,
    #[error("No pending challenge for this client")]
    NoPendingChallenge,
    #[error("Challenge expired")]
    ChallengeExpired,
    #[error("Invalid key: {0}")]
    InvalidKey(String),
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),
    #[error("Signature verification failed — client does not possess the private key")]
    SignatureVerificationFailed,
    #[error("Invalid auth message")]
    InvalidMessage,
    #[error("Internal error: {0}")]
    Internal(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};

    #[tokio::test]
    async fn test_full_auth_flow() {
        let auth = RelayAuthenticator::new();

        // Generate client keypair
        let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);
        let pubkey = signing_key.verifying_key();

        // Step 1: Request challenge
        let challenge = auth.create_challenge("127.0.0.1:12345").await.unwrap();
        let nonce_hex = match &challenge {
            AuthMessage::AuthChallenge { nonce, .. } => nonce.clone(),
            _ => panic!("Expected challenge"),
        };

        // Step 2: Client signs the challenge
        let nonce_bytes = hex::decode(&nonce_hex).unwrap();
        let signature = signing_key.sign(&nonce_bytes);

        let response = AuthMessage::AuthResponse {
            ed25519_public_key: hex::encode(pubkey.as_bytes()),
            signature: hex::encode(signature.to_bytes()),
            x25519_public_key: None,
            prekey_bundle: None,
            profile: None,
        };

        // Step 3: Server verifies
        let identity = auth
            .verify_response("127.0.0.1:12345", &response)
            .await
            .unwrap();

        assert_eq!(identity.ed25519_public_key, *pubkey.as_bytes());
        assert!(!identity.hades_id.is_empty());

        // Verify authenticated
        assert!(auth.is_authenticated(&identity.hades_id).await);
    }

    #[tokio::test]
    async fn test_wrong_key_fails() {
        let auth = RelayAuthenticator::new();

        let real_key = SigningKey::generate(&mut rand::rngs::OsRng);
        let fake_key = SigningKey::generate(&mut rand::rngs::OsRng);

        let challenge = auth.create_challenge("127.0.0.1:9999").await.unwrap();
        let nonce_hex = match &challenge {
            AuthMessage::AuthChallenge { nonce, .. } => nonce.clone(),
            _ => panic!(),
        };

        let nonce_bytes = hex::decode(&nonce_hex).unwrap();

        // Sign with WRONG key, claim to be real_key
        let signature = fake_key.sign(&nonce_bytes);

        let response = AuthMessage::AuthResponse {
            ed25519_public_key: hex::encode(real_key.verifying_key().as_bytes()),
            signature: hex::encode(signature.to_bytes()),
            x25519_public_key: None,
            prekey_bundle: None,
            profile: None,
        };

        let result = auth.verify_response("127.0.0.1:9999", &response).await;
        assert!(matches!(result, Err(AuthError::SignatureVerificationFailed)));
    }

    #[tokio::test]
    async fn test_expired_challenge() {
        let auth = RelayAuthenticator::new();

        let key = SigningKey::generate(&mut rand::rngs::OsRng);

        let challenge = auth.create_challenge("127.0.0.1:8888").await.unwrap();
        let nonce_hex = match &challenge {
            AuthMessage::AuthChallenge { nonce, .. } => nonce.clone(),
            _ => panic!(),
        };

        // Manually expire the challenge
        {
            let mut pending = auth.pending_challenges.write().await;
            if let Some(c) = pending.get_mut("127.0.0.1:8888") {
                c.issued_at = Instant::now() - Duration::from_secs(60);
            }
        }

        let nonce_bytes = hex::decode(&nonce_hex).unwrap();
        let signature = key.sign(&nonce_bytes);

        let response = AuthMessage::AuthResponse {
            ed25519_public_key: hex::encode(key.verifying_key().as_bytes()),
            signature: hex::encode(signature.to_bytes()),
            x25519_public_key: None,
            prekey_bundle: None,
            profile: None,
        };

        let result = auth.verify_response("127.0.0.1:8888", &response).await;
        assert!(matches!(result, Err(AuthError::ChallengeExpired)));
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        let auth = RelayAuthenticator::new();

        // 10 attempts should succeed
        for _ in 0..10 {
            assert!(auth.create_challenge("127.0.0.1:7777").await.is_ok());
        }

        // 11th should fail
        assert!(matches!(
            auth.create_challenge("127.0.0.1:7777").await,
            Err(AuthError::RateLimited)
        ));

        // Different IP should still work
        assert!(auth.create_challenge("127.0.0.2:7777").await.is_ok());
    }
}
