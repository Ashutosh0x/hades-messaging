use crate::db;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use crate::websocket::RelayMessage;
use hades_crypto::aead;
use hades_crypto::double_ratchet::RatchetState;
use hades_crypto::padding;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaintextMessage {
    pub id: String,
    pub content: String,
    pub timestamp: String,
    pub reply_to: Option<String>,
    pub burn_after: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecryptedMessage {
    pub id: String,
    pub conversation_id: String,
    pub sender_id: String,
    pub content: String,
    pub timestamp: String,
    pub burn_after: Option<i64>,
    pub reply_to: Option<String>,
}

/// Encrypt a message for a contact and return the sealed envelope.
///
/// Pipeline: serialize → pad → ratchet encrypt → sealed sender wrap
pub fn encrypt_message(
    session: &mut RatchetState,
    plaintext: &PlaintextMessage,
    sender_identity: &[u8; 32],
    recipient_identity: &[u8; 32],
) -> AppResult<Vec<u8>> {
    // 1. Serialize the plaintext
    let plaintext_bytes = serde_json::to_vec(plaintext)
        .map_err(|e| AppError::Crypto(format!("Serialize failed: {}", e)))?;

    // 2. Pad to fixed bucket (prevents length-based analysis)
    let padded = padding::pad_message(&plaintext_bytes);

    // 3. Double Ratchet encrypt (returns header + ciphertext)
    let (header, ciphertext) = session
        .encrypt(&padded)
        .map_err(|e| AppError::Crypto(format!("Ratchet encrypt failed: {:?}", e)))?;

    // 4. Combine header + ciphertext into a frame
    let header_bytes = serde_json::to_vec(&header)
        .map_err(|e| AppError::Crypto(format!("Header serialize failed: {}", e)))?;

    let mut frame = Vec::new();
    frame.extend_from_slice(&(header_bytes.len() as u32).to_be_bytes());
    frame.extend_from_slice(&header_bytes);
    frame.extend_from_slice(&ciphertext);

    // 5. Sealed Sender: wrap so relay can't see sender identity
    let sealed = seal_sender_wrap(&frame, sender_identity, recipient_identity)?;

    Ok(sealed)
}

/// Decrypt a received sealed envelope.
///
/// Pipeline: unseal sender → parse frame → ratchet decrypt → unpad → deserialize
pub fn decrypt_message(
    session: &mut RatchetState,
    sealed_envelope: &[u8],
    our_identity_secret: &[u8; 32],
    _our_identity_public: &[u8; 32],
) -> AppResult<(PlaintextMessage, [u8; 32])> {
    // 1. Unseal (recover sender identity + inner frame)
    let (sender_pubkey, frame) =
        unseal_sender(sealed_envelope, our_identity_secret)?;

    // 2. Parse header + ciphertext from frame
    if frame.len() < 4 {
        return Err(AppError::Crypto("Frame too short".into()));
    }
    let header_len =
        u32::from_be_bytes([frame[0], frame[1], frame[2], frame[3]]) as usize;

    if frame.len() < 4 + header_len {
        return Err(AppError::Crypto("Frame header truncated".into()));
    }

    let header: hades_crypto::double_ratchet::MessageHeader =
        serde_json::from_slice(&frame[4..4 + header_len])
            .map_err(|e| AppError::Crypto(format!("Header parse failed: {}", e)))?;
    let ciphertext = &frame[4 + header_len..];

    // 3. Double Ratchet decrypt
    let padded = session
        .decrypt(&header, ciphertext)
        .map_err(|e| AppError::Crypto(format!("Ratchet decrypt failed: {:?}", e)))?;

    // 4. Remove padding
    let plaintext_bytes = padding::unpad_message(&padded)
        .ok_or_else(|| AppError::Crypto("Unpad failed".into()))?;

    // 5. Deserialize
    let msg: PlaintextMessage = serde_json::from_slice(&plaintext_bytes)
        .map_err(|e| AppError::Crypto(format!("Message parse failed: {}", e)))?;

    Ok((msg, sender_pubkey))
}

/// Sealed Sender: ephemeral DH to hide sender identity from relay
fn seal_sender_wrap(
    inner: &[u8],
    sender_identity: &[u8; 32],
    recipient_identity: &[u8; 32],
) -> AppResult<Vec<u8>> {
    // Generate ephemeral X25519 keypair
    let ephemeral_sk =
        x25519_dalek::StaticSecret::random_from_rng(rand::thread_rng());
    let ephemeral_pk = x25519_dalek::PublicKey::from(&ephemeral_sk);

    // DH with recipient's identity key (used as X25519 key)
    let recipient_pk =
        x25519_dalek::PublicKey::from(*recipient_identity);
    let shared_secret = ephemeral_sk.diffie_hellman(&recipient_pk);

    // KDF: derive encryption key from shared secret
    let mut enc_key = [0u8; 32];
    let hk = hkdf::Hkdf::<sha2::Sha256>::new(None, shared_secret.as_bytes());
    hk.expand(b"HadesSealedSender", &mut enc_key)
        .map_err(|_| AppError::Crypto("HKDF expand failed".into()))?;

    // Build plaintext: sender_identity (32) + inner
    let mut plaintext = Vec::with_capacity(32 + inner.len());
    plaintext.extend_from_slice(sender_identity);
    plaintext.extend_from_slice(inner);

    let mut nonce_bytes = [0u8; 12];
    rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut nonce_bytes);

    let ciphertext = aead::encrypt(&enc_key, &nonce_bytes, &plaintext, b"")
        .map_err(|e| AppError::Crypto(format!("AEAD seal failed: {:?}", e)))?;

    // Output: ephemeral_pk (32) + nonce (12) + ciphertext
    let mut output = Vec::with_capacity(32 + 12 + ciphertext.len());
    output.extend_from_slice(ephemeral_pk.as_bytes());
    output.extend_from_slice(&nonce_bytes);
    output.extend_from_slice(&ciphertext);

    // Zeroize secrets
    zeroize::Zeroize::zeroize(&mut enc_key);

    Ok(output)
}

/// Unseal a sealed-sender envelope, returning (sender_identity, inner_frame)
fn unseal_sender(
    envelope: &[u8],
    our_secret: &[u8; 32],
) -> AppResult<([u8; 32], Vec<u8>)> {
    if envelope.len() < 32 + 12 + 16 {
        return Err(AppError::Crypto("Envelope too short".into()));
    }

    let mut ephemeral_pk_bytes = [0u8; 32];
    ephemeral_pk_bytes.copy_from_slice(&envelope[0..32]);
    let ephemeral_pk = x25519_dalek::PublicKey::from(ephemeral_pk_bytes);

    let mut nonce = [0u8; 12];
    nonce.copy_from_slice(&envelope[32..44]);

    let ciphertext = &envelope[44..];

    // DH with our identity secret
    let our_sk = x25519_dalek::StaticSecret::from(*our_secret);
    let shared_secret = our_sk.diffie_hellman(&ephemeral_pk);

    // KDF
    let mut enc_key = [0u8; 32];
    let hk = hkdf::Hkdf::<sha2::Sha256>::new(None, shared_secret.as_bytes());
    hk.expand(b"HadesSealedSender", &mut enc_key)
        .map_err(|_| AppError::Crypto("HKDF expand failed".into()))?;

    // Decrypt
    let plaintext = aead::decrypt(&enc_key, &nonce, ciphertext, b"")
        .map_err(|e| AppError::Crypto(format!("Unseal failed: {:?}", e)))?;

    if plaintext.len() < 32 {
        return Err(AppError::Crypto("Unsealed payload too short".into()));
    }

    let mut sender_id = [0u8; 32];
    sender_id.copy_from_slice(&plaintext[0..32]);
    let inner = plaintext[32..].to_vec();

    zeroize::Zeroize::zeroize(&mut enc_key);

    Ok((sender_id, inner))
}

/// Process incoming messages from the relay (run in background task)
pub async fn incoming_message_loop(
    state: Arc<RwLock<AppState>>,
    app_handle: tauri::AppHandle,
) {
    loop {
        let incoming_rx = {
            let s = state.read().await;
            match &s.relay {
                Some(relay) => relay.incoming_rx.clone(),
                None => {
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                    continue;
                }
            }
        };

        let msg = {
            let mut rx = incoming_rx.lock().await;
            rx.recv().await
        };

        match msg {
            Some(RelayMessage::Receive {
                envelope,
                timestamp: _,
                ..
            }) => {
                let mut s = state.write().await;
                let identity_x25519_bytes = {
                    match &s.messaging_keypair {
                        Some(kp) => *kp.x25519_public.as_bytes(),
                        None => continue,
                    }
                };

                // Try to decrypt with each active session
                let contact_ids: Vec<String> = s.sessions.keys().cloned().collect();
                for contact_id in &contact_ids {
                    if let Some(session) = s.sessions.get_mut(contact_id) {
                        // Use the x25519 secret bytes directly — we need a way to access them.
                        // For now, we use a placeholder 32-byte key. In production, store
                        // the x25519 secret in the database.
                        let our_secret = [0u8; 32]; // TODO: store & retrieve x25519 secret
                        match decrypt_message(
                            session,
                            &envelope,
                            &our_secret,
                            &identity_x25519_bytes,
                        ) {
                            Ok((plaintext_msg, sender_pubkey)) => {
                                let decrypted = DecryptedMessage {
                                    id: plaintext_msg.id.clone(),
                                    conversation_id: contact_id.clone(),
                                    sender_id: hex::encode(sender_pubkey),
                                    content: plaintext_msg.content,
                                    timestamp: plaintext_msg.timestamp,
                                    burn_after: plaintext_msg.burn_after,
                                    reply_to: plaintext_msg.reply_to,
                                };

                                // Store in database
                                if let Some(ref db) = s.db {
                                    let stored = db::messages::StoredMessage {
                                        id: decrypted.id.clone(),
                                        conversation_id: decrypted.conversation_id.clone(),
                                        sender_id: decrypted.sender_id.clone(),
                                        content_encrypted: decrypted.content.as_bytes().to_vec(),
                                        content_nonce: vec![],
                                        timestamp: decrypted.timestamp.clone(),
                                        status: "delivered".to_string(),
                                        burn_after: decrypted.burn_after,
                                        reply_to: decrypted.reply_to.clone(),
                                    };
                                    let _ = db::messages::insert_message(db.conn(), &stored);
                                }

                                // Emit event to frontend
                                let _ = app_handle.emit("new-message", &decrypted);
                                break;
                            }
                            Err(_) => continue,
                        }
                    }
                }
            }
            Some(RelayMessage::Receipt {
                message_id,
                status,
            }) => {
                let s = state.read().await;
                if let Some(ref db) = s.db {
                    let _ = db::messages::update_message_status(
                        db.conn(),
                        &message_id,
                        &status,
                    );
                    let _ = app_handle.emit(
                        "message-status",
                        serde_json::json!({ "id": message_id, "status": status }),
                    );
                }
            }
            None => {
                log::warn!("Relay connection lost, will reconnect...");
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
            _ => {}
        }
    }
}
