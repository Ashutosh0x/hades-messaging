use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SealedPayload {
    pub sender_id: String,
    pub ciphertext: String,
    pub timestamp: u64,
}

/// Represents the user's setting for OS-level notifications.
#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationConfig {
    /// Show sender name and decrypted message (Apple/Android default, unsafe)
    Full,
    /// Show sender name, hide message ("Elias: New message")
    SenderOnly,
    /// Absolute privacy ("Hades: New Secure Message")
    Sealed,
}

/// A background service stub that simulates intercepting a network payload,
/// decrypting it (if the vault is unlocked), and deciding what to show to the OS.
#[tauri::command]
pub fn handle_incoming_push(
    payload: SealedPayload,
    config: NotificationConfig,
    vault_locked: bool,
) -> Result<String, String> {
    // 1. Hardware State Check
    // If the vault is locked, the SQLCipher keys are wiped from RAM.
    // It is cryptographically impossible to decrypt the payload.
    if vault_locked {
        // We must fallback to the absolute sealed state, regardless of config.
        return Ok("Hades: New Secure Message".into());
    }

    // 2. We have keys. Determine OS disclosure level based on config.
    match config {
        NotificationConfig::Sealed => {
            // User explicitly requested zero metadata in OS logs
            Ok("Hades: New Secure Message".into())
        }
        NotificationConfig::SenderOnly => {
            // We can decrypt the sender metadata, but hide the content
            Ok(format!("{}: New message", payload.sender_id))
        }
        NotificationConfig::Full => {
            // Decrypt everything and hand it to the OS Apple/Google Push service.
            // In a true production app, we would:
            // 1. Open SQLCipher KV store using background memory-resident key
            // 2. Fetch the PQXDH session state for `sender_id`
            // 3. Advance the Double Ratchet chain
            // 4. `chacha20poly1305::decrypt(message_key, payload.ciphertext)`
            
            // Structured example of the decryption flow:
            /*
            let session = db.get_session(&payload.sender_id)?;
            let (next_session, message_key) = session.ratchet_forward()?;
            db.save_session(&payload.sender_id, next_session)?;
            
            use chacha20poly1305::{ChaCha20Poly1305, KeyInit, aead::Aead};
            let cipher = ChaCha20Poly1305::new(&message_key);
            let nonce = chacha20poly1305::Nonce::from_slice(&[0u8; 12]); // simplified
            let plaintext_bytes = cipher.decrypt(nonce, payload.ciphertext.as_bytes())
                .map_err(|_| "Decryption failed")?;
            let plaintext = String::from_utf8(plaintext_bytes).unwrap_or_default();
            
            return Ok(format!("{}: {}", payload.sender_id, plaintext));
            */
            
            // For now we simulate the decryption output for UI development.
            Ok(format!("{}: [Decrypted Content: {}]", payload.sender_id, payload.ciphertext))
        }
    }
}
