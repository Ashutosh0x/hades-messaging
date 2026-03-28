//! Client-side authentication handler for relay challenge-response.
//!
//! This module handles the client side of the Ed25519 challenge-response
//! protocol with the relay server. No passwords, no tokens — just
//! cryptographic proof of key possession.

use crate::error::{AppError, AppResult};
use crate::state::AppState;
use hades_identity::seed::MessagingKeypair;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Client-side authentication state
#[derive(Debug, Clone, Serialize)]
pub struct AuthState {
    pub authenticated: bool,
    pub hades_id: String,
    pub hades_id_short: String,
    pub ed25519_public_key: String,
    pub x25519_public_key: String,
}

/// Build the initial auth state from the messaging keypair
pub fn auth_state_from_keypair(keypair: &MessagingKeypair, authenticated: bool) -> AuthState {
    AuthState {
        authenticated,
        hades_id: keypair.hades_id_hex(),
        hades_id_short: keypair.hades_id_short(),
        ed25519_public_key: hex::encode(keypair.ed25519_public.as_bytes()),
        x25519_public_key: hex::encode(keypair.x25519_public.as_bytes()),
    }
}

/// Process incoming auth challenge from relay.
///
/// Signs the challenge nonce with our Ed25519 private key and returns
/// the auth response JSON to send back.
pub async fn process_auth_challenge(
    state: &Arc<RwLock<AppState>>,
    nonce_hex: &str,
) -> AppResult<serde_json::Value> {
    let s = state.read().await;

    let keypair = s
        .messaging_keypair
        .as_ref()
        .ok_or(AppError::NotInitialized("No identity keypair".into()))?;

    // Decode nonce
    let nonce = hex::decode(nonce_hex)
        .map_err(|_| AppError::Auth("Invalid challenge nonce hex".into()))?;

    // Sign the challenge with our Ed25519 private key
    let signature = keypair.sign(&nonce);

    // Build prekey bundle to upload
    let prekey_bundle = build_prekey_bundle(keypair)?;

    // Build auth response
    let response = serde_json::json!({
        "type": "auth_response",
        "ed25519_public_key": hex::encode(keypair.ed25519_public.as_bytes()),
        "signature": hex::encode(&signature),
        "x25519_public_key": hex::encode(keypair.x25519_public.as_bytes()),
        "prekey_bundle": prekey_bundle,
        "profile": {
            "display_name": s.display_name.as_deref(),
        }
    });

    Ok(response)
}

/// Build a prekey bundle to upload to the relay.
///
/// Generates a signed prekey and 100 one-time prekeys.
fn build_prekey_bundle(keypair: &MessagingKeypair) -> AppResult<serde_json::Value> {
    // Generate signed prekey (X25519)
    let spk_secret = x25519_dalek::StaticSecret::random_from_rng(rand::thread_rng());
    let spk_public = x25519_dalek::PublicKey::from(&spk_secret);
    let spk_signature = keypair.sign(spk_public.as_bytes());

    // Generate one-time prekeys
    let mut otpks = Vec::new();
    for _ in 0..100 {
        let otpk_secret = x25519_dalek::StaticSecret::random_from_rng(rand::thread_rng());
        let otpk_public = x25519_dalek::PublicKey::from(&otpk_secret);
        otpks.push(hex::encode(otpk_public.as_bytes()));
    }

    Ok(serde_json::json!({
        "signed_prekey": hex::encode(spk_public.as_bytes()),
        "signed_prekey_signature": hex::encode(&spk_signature),
        "one_time_prekeys": otpks,
    }))
}
