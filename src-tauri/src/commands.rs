use crate::db;
use crate::error::{AppError, AppResult};
use crate::pipeline;
use crate::state::AppState;
use crate::websocket::{RelayConnection, RelayMessage};
use hades_crypto::double_ratchet::RatchetState;
use hades_identity::seed::MasterSeed;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;

type SharedState = State<'_, Arc<RwLock<AppState>>>;

// ─── Identity ───────────────────────────────────────────────────────

#[tauri::command]
pub async fn generate_identity(state: SharedState) -> AppResult<String> {
    let master_seed = MasterSeed::generate()
        .map_err(|e| AppError::Identity(e.to_string()))?;
    let messaging_kp = master_seed.derive_messaging_keypair()
        .map_err(|e| AppError::Identity(e.to_string()))?;
    let pubkey_hex = hex::encode(messaging_kp.ed25519_public.as_bytes());

    let mut s = state.write().await;

    // Persist if DB is open
    if let Some(ref db) = s.db {
        db::keys::store_identity(
            db.conn(),
            messaging_kp.ed25519_public.as_bytes(),
            &messaging_kp.ed25519_signing.to_bytes(),
        )?;
    }

    s.messaging_keypair = Some(messaging_kp);
    Ok(pubkey_hex)
}

#[tauri::command]
pub async fn get_identity_pubkey(state: SharedState) -> AppResult<Option<String>> {
    let s = state.read().await;
    Ok(s.messaging_keypair.as_ref().map(|kp| hex::encode(kp.ed25519_public.as_bytes())))
}

#[tauri::command]
pub async fn get_safety_number(
    state: SharedState,
    contact_pubkey_hex: String,
) -> AppResult<String> {
    let s = state.read().await;
    let keypair = s
        .messaging_keypair
        .as_ref()
        .ok_or(AppError::NotInitialized("No identity".into()))?;

    let contact_bytes = hex::decode(&contact_pubkey_hex)
        .map_err(|_| AppError::Identity("Invalid hex".into()))?;

    let mut contact_key = [0u8; 32];
    if contact_bytes.len() >= 32 {
        contact_key.copy_from_slice(&contact_bytes[..32]);
    }

    let safety_number = keypair.safety_number_with(&contact_key);
    Ok(safety_number)
}

// ─── Key Generation ─────────────────────────────────────────────────

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct X25519KeypairResult {
    pub public_key: String,
    pub secret_key: String,
}

#[tauri::command]
pub async fn generate_x25519_keypair() -> AppResult<X25519KeypairResult> {
    let secret = x25519_dalek::StaticSecret::random_from_rng(rand::thread_rng());
    let public = x25519_dalek::PublicKey::from(&secret);

    Ok(X25519KeypairResult {
        public_key: hex::encode(public.as_bytes()),
        secret_key: hex::encode(secret.to_bytes()),
    })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SignedPrekeyResult {
    pub public_key: String,
    pub signature: String,
}

#[tauri::command]
pub async fn generate_signed_prekey(
    state: SharedState,
) -> AppResult<SignedPrekeyResult> {
    let s = state.read().await;
    let keypair = s
        .messaging_keypair
        .as_ref()
        .ok_or(AppError::NotInitialized("No identity".into()))?;

    let secret = x25519_dalek::StaticSecret::random_from_rng(rand::thread_rng());
    let public = x25519_dalek::PublicKey::from(&secret);
    let signature_bytes = keypair.sign(public.as_bytes());
    let signature = ed25519_dalek::Signature::from_slice(&signature_bytes)
        .map_err(|_| AppError::Crypto("Signature creation failed".into()))?;

    // Persist if DB is available
    if let Some(ref db) = s.db {
        db::keys::upsert_signed_prekey(
            db.conn(),
            public.as_bytes(),
            &secret.to_bytes(),
            &signature.to_bytes(),
        )?;
    }

    Ok(SignedPrekeyResult {
        public_key: hex::encode(public.as_bytes()),
        signature: hex::encode(signature.to_bytes()),
    })
}

#[tauri::command]
pub async fn generate_prekey_bundle(
    state: SharedState,
    count: Option<u32>,
) -> AppResult<String> {
    let count = count.unwrap_or(100);
    let s = state.read().await;

    let mut prekeys = Vec::new();
    for _ in 0..count {
        let secret =
            x25519_dalek::StaticSecret::random_from_rng(rand::thread_rng());
        let public = x25519_dalek::PublicKey::from(&secret);
        prekeys.push((public.as_bytes().to_vec(), secret.to_bytes().to_vec()));
    }

    if let Some(ref db) = s.db {
        db::keys::insert_prekeys(db.conn(), &prekeys)?;
    }

    Ok(format!("Generated {} prekeys", count))
}

// ─── Sessions ───────────────────────────────────────────────────────

#[tauri::command]
pub async fn initiate_session(
    state: SharedState,
    contact_id: String,
    their_signed_prekey_hex: String,
) -> AppResult<String> {
    let mut s = state.write().await;

    let keypair = s
        .messaging_keypair
        .as_ref()
        .ok_or(AppError::NotInitialized("No identity".into()))?;

    // Parse their signed prekey
    let their_spk = hex::decode(&their_signed_prekey_hex)
        .map_err(|_| AppError::Identity("Invalid SPK hex".into()))?;

    // X3DH key agreement
    let our_ek = x25519_dalek::StaticSecret::random_from_rng(rand::thread_rng());

    let their_spk_pk = x25519_dalek::PublicKey::from(
        <[u8; 32]>::try_from(their_spk.as_slice())
            .map_err(|_| AppError::Identity("SPK wrong length".into()))?,
    );

    // Simplified X3DH: DH(EK_A, SPK_B) → shared secret
    let dh_output = our_ek.diffie_hellman(&their_spk_pk);

    // Derive shared secret with HKDF
    let hk = hkdf::Hkdf::<sha2::Sha256>::new(Some(&[0u8; 32]), dh_output.as_bytes());
    let mut shared_secret = [0u8; 32];
    hk.expand(b"HadesX3DH", &mut shared_secret)
        .map_err(|_| AppError::Crypto("HKDF failed".into()))?;

    // Initialize Double Ratchet as Alice (initiator)
    let their_spk_bytes: [u8; 32] = their_spk
        .try_into()
        .map_err(|_| AppError::Crypto("SPK wrong size".into()))?;

    let session = RatchetState::init_alice(shared_secret, their_spk_bytes);

    s.sessions.insert(contact_id.clone(), session);

    zeroize::Zeroize::zeroize(&mut shared_secret);

    Ok(format!("Session established with {}", contact_id))
}

#[tauri::command]
pub async fn has_session(
    state: SharedState,
    contact_id: String,
) -> AppResult<bool> {
    let s = state.read().await;
    if s.sessions.contains_key(&contact_id) {
        return Ok(true);
    }
    if let Some(ref db) = s.db {
        return db::sessions::has_session(db.conn(), &contact_id);
    }
    Ok(false)
}

// ─── Messages ───────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageOut {
    pub id: String,
    pub conversation_id: String,
    pub sender_id: String,
    pub content: String,
    pub timestamp: String,
    pub status: String,
    pub burn_after: Option<i64>,
    pub reply_to: Option<String>,
}

#[tauri::command]
pub async fn send_message(
    state: SharedState,
    contact_id: String,
    content: String,
    burn_after: Option<i64>,
    reply_to: Option<String>,
) -> AppResult<MessageOut> {
    let mut s = state.write().await;

    let our_x25519_pub = {
        let keypair = s
            .messaging_keypair
            .as_ref()
            .ok_or(AppError::NotInitialized("No identity".into()))?;
        *keypair.x25519_public.as_bytes()
    };

    let session = s
        .sessions
        .get_mut(&contact_id)
        .ok_or(AppError::Session("No session with this contact".into()))?;

    // Build message
    let msg_id = uuid_v4();
    let timestamp = unix_timestamp_str();
    let plaintext = pipeline::PlaintextMessage {
        id: msg_id.clone(),
        content: content.clone(),
        timestamp: timestamp.clone(),
        reply_to: reply_to.clone(),
        burn_after,
    };

    // Get recipient's identity key from contacts
    let recipient_key = if let Some(ref db) = s.db {
        let contact = db::contacts::get_contact(db.conn(), &contact_id)?
            .ok_or(AppError::Session("Contact not found".into()))?;
        let mut key = [0u8; 32];
        let len = contact.identity_key.len().min(32);
        key[..len].copy_from_slice(&contact.identity_key[..len]);
        key
    } else {
        return Err(AppError::NotInitialized("No database".into()));
    };

    // Encrypt
    let envelope = pipeline::encrypt_message(
        session,
        &plaintext,
        &our_x25519_pub,
        &recipient_key,
    )?;

    // Send over WebSocket
    if let Some(ref relay) = s.relay {
        relay
            .send(RelayMessage::Send {
                recipient_id: contact_id.clone(),
                envelope: envelope.clone(),
                message_id: msg_id.clone(),  // S2 FIX: include message_id
            })
            .await?;
    }

    // Store locally in encrypted DB
    let stored = db::messages::StoredMessage {
        id: msg_id.clone(),
        conversation_id: contact_id.clone(),
        sender_id: "self".to_string(),
        content_encrypted: content.as_bytes().to_vec(),
        content_nonce: vec![],
        timestamp: timestamp.clone(),
        status: "sent".to_string(),
        burn_after,
        reply_to: reply_to.clone(),
    };

    if let Some(ref db) = s.db {
        db::messages::insert_message(db.conn(), &stored)?;
    }

    Ok(MessageOut {
        id: msg_id,
        conversation_id: contact_id,
        sender_id: "self".to_string(),
        content,
        timestamp,
        status: "sent".to_string(),
        burn_after,
        reply_to,
    })
}

#[tauri::command]
pub async fn get_messages(
    state: SharedState,
    conversation_id: String,
    limit: Option<i64>,
    offset: Option<i64>,
) -> AppResult<Vec<MessageOut>> {
    let s = state.read().await;
    let db = s.db.as_ref().ok_or(AppError::DatabaseLocked)?;

    let stored = db::messages::get_messages_for_conversation(
        db.conn(),
        &conversation_id,
        limit.unwrap_or(50),
        offset.unwrap_or(0),
    )?;

    let messages: Vec<MessageOut> = stored
        .into_iter()
        .map(|m| MessageOut {
            id: m.id,
            conversation_id: m.conversation_id,
            sender_id: m.sender_id,
            content: String::from_utf8_lossy(&m.content_encrypted).to_string(),
            timestamp: m.timestamp,
            status: m.status,
            burn_after: m.burn_after,
            reply_to: m.reply_to,
        })
        .collect();

    Ok(messages)
}

#[tauri::command]
pub async fn mark_message_read(
    state: SharedState,
    message_id: String,
) -> AppResult<()> {
    let s = state.read().await;
    let db = s.db.as_ref().ok_or(AppError::DatabaseLocked)?;
    db::messages::update_message_status(db.conn(), &message_id, "read")?;
    Ok(())
}

// ─── Contacts ───────────────────────────────────────────────────────

#[tauri::command]
pub async fn add_contact(
    state: SharedState,
    id: String,
    display_name: String,
    identity_key_hex: String,
) -> AppResult<()> {
    let s = state.read().await;
    let db = s.db.as_ref().ok_or(AppError::DatabaseLocked)?;

    let identity_key = hex::decode(&identity_key_hex)
        .map_err(|_| AppError::Identity("Invalid hex".into()))?;

    let contact = db::contacts::Contact {
        id,
        display_name,
        identity_key,
        safety_number: None,
        verified: false,
        created_at: String::new(),
    };

    db::contacts::insert_contact(db.conn(), &contact)?;
    Ok(())
}

#[tauri::command]
pub async fn get_contacts(state: SharedState) -> AppResult<Vec<db::contacts::Contact>> {
    let s = state.read().await;
    let db = s.db.as_ref().ok_or(AppError::DatabaseLocked)?;
    db::contacts::get_all_contacts(db.conn())
}

#[tauri::command]
pub async fn delete_contact(state: SharedState, id: String) -> AppResult<()> {
    let s = state.read().await;
    let db = s.db.as_ref().ok_or(AppError::DatabaseLocked)?;
    db::contacts::delete_contact(db.conn(), &id)?;
    Ok(())
}

// ─── Database ───────────────────────────────────────────────────────

#[tauri::command]
pub async fn initialize_database(
    app_handle: tauri::AppHandle,
    state: SharedState,
    passphrase: String,
) -> AppResult<()> {
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|_| AppError::Internal("Cannot resolve app data dir".into()))?;

    std::fs::create_dir_all(&app_dir)
        .map_err(|e| AppError::Internal(format!("Cannot create dir: {}", e)))?;

    let db_path = app_dir.join("hades.db");
    let database = crate::db::Database::open(db_path, &passphrase)?;

    let mut s = state.write().await;
    s.db = Some(database);
    s.vault_unlocked = true;

    log::info!("Database initialized");
    Ok(())
}

#[tauri::command]
pub async fn lock_database(state: SharedState) -> AppResult<()> {
    let mut s = state.write().await;
    s.db = None;
    s.sessions.clear();
    s.vault_unlocked = false;
    log::info!("Database locked");
    Ok(())
}

#[tauri::command]
pub async fn unlock_database(
    app_handle: tauri::AppHandle,
    state: SharedState,
    passphrase: String,
) -> AppResult<()> {
    initialize_database(app_handle, state, passphrase).await
}

// ─── Network ────────────────────────────────────────────────────────

#[tauri::command]
pub async fn connect_relay(
    state: SharedState,
    app_handle: tauri::AppHandle,
    relay_url: String,
) -> AppResult<String> {
    let identity_pubkey = {
        let s = state.read().await;
        let keypair = s
            .messaging_keypair
            .as_ref()
            .ok_or(AppError::NotInitialized("No identity".into()))?;
        keypair.ed25519_public.as_bytes().to_vec()
    };

    let connection =
        RelayConnection::connect(&relay_url, &identity_pubkey).await?;

    {
        let mut s = state.write().await;
        s.relay = Some(connection);
    }

    // Start background message receiver
    let state_clone = state.inner().clone();
    tokio::spawn(async move {
        pipeline::incoming_message_loop(state_clone, app_handle).await;
    });

    Ok("Connected to relay".into())
}

#[tauri::command]
pub async fn disconnect_relay(state: SharedState) -> AppResult<()> {
    let mut s = state.write().await;
    s.relay = None;
    Ok(())
}

#[tauri::command]
pub async fn get_connection_status(state: SharedState) -> AppResult<String> {
    let s = state.read().await;
    match &s.relay {
        Some(relay) if relay.is_connected() => Ok("connected".into()),
        Some(_) => Ok("disconnected".into()),
        None => Ok("not_initialized".into()),
    }
}

// ─── Anti-Forensics ─────────────────────────────────────────────────

#[tauri::command]
pub async fn emergency_wipe(
    app_handle: tauri::AppHandle,
    state: SharedState,
) -> AppResult<()> {
    let mut s = state.write().await;

    // 1. Clear all sessions (zeroize keys in memory)
    s.sessions.clear();
    s.messaging_keypair = None;
    s.relay = None;

    // 2. Destroy database
    if let Some(db) = s.db.take() {
        db.destroy()?;
    }

    // 3. Wipe app data directory
    if let Ok(app_dir) = app_handle.path().app_data_dir() {
        if app_dir.exists() {
            wipe_directory_recursive(&app_dir);
        }
    }

    s.vault_unlocked = false;
    log::info!("Emergency wipe complete");
    Ok(())
}

fn wipe_directory_recursive(dir: &std::path::Path) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                wipe_directory_recursive(&path);
                std::fs::remove_dir(&path).ok();
            } else {
                if let Ok(meta) = std::fs::metadata(&path) {
                    let zeros = vec![0u8; meta.len() as usize];
                    std::fs::write(&path, &zeros).ok();
                }
                std::fs::remove_file(&path).ok();
            }
        }
    }
    std::fs::remove_dir(dir).ok();
}

// ─── Recovery ───────────────────────────────────────────────────────

#[tauri::command]
pub async fn generate_recovery_phrase(state: SharedState) -> AppResult<Vec<String>> {
    let s = state.read().await;
    let _keypair = s
        .messaging_keypair
        .as_ref()
        .ok_or(AppError::NotInitialized("No identity".into()))?;

    // Use the recovery module from hades-identity
    let mnemonic = hades_identity::recovery::Mnemonic::generate();
    Ok(mnemonic.words)
}

#[tauri::command]
pub async fn restore_from_recovery(
    app_handle: tauri::AppHandle,
    state: SharedState,
    words: Vec<String>,
    passphrase: String,
) -> AppResult<String> {
    // ── C3 FIX: Derive identity FROM the mnemonic, not a new random one ──
    let phrase = words.join(" ");
    let master_seed = MasterSeed::from_mnemonic(&phrase)
        .map_err(|e| AppError::Identity(format!("Invalid recovery phrase: {}", e)))?;

    let messaging_kp = master_seed
        .derive_messaging_keypair()
        .map_err(|e| AppError::Identity(e.to_string()))?;
    let pubkey_hex = messaging_kp.hades_id_hex();

    // Derive wallet from same seed
    let wallet = hades_wallet::hd::HdWallet::from_mnemonic(&phrase)
        .map_err(|e| AppError::Internal(format!("Wallet derivation failed: {}", e)))?;

    // ── C2 FIX: Use the actual user passphrase, not "temporary_passphrase" ──
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|_| AppError::Internal("Cannot resolve app dir".into()))?;
    std::fs::create_dir_all(&app_dir)
        .map_err(|e| AppError::Internal(format!("Cannot create dir: {}", e)))?;

    let db_path = app_dir.join("hades.db");

    // Remove existing database if present (fresh restore)
    if db_path.exists() {
        std::fs::remove_file(&db_path).ok();
    }

    let database = crate::db::Database::open(db_path, &passphrase)?;

    // Store restored identity
    db::keys::store_identity(
        database.conn(),
        messaging_kp.ed25519_public.as_bytes(),
        &messaging_kp.ed25519_signing.to_bytes(),
    )?;

    // Store mnemonic (encrypted by SQLCipher with the user's passphrase)
    database.conn().execute(
        "INSERT OR REPLACE INTO kv_store (key, value) VALUES ('master_mnemonic', ?1)",
        rusqlite::params![phrase.as_bytes()],
    )?;

    // Store wallet accounts
    let wallet_accounts = wallet.derive_all_accounts();
    for acc in &wallet_accounts {
        let row = db::wallet::WalletAccountRow {
            id: 0,
            chain: format!("{:?}", acc.chain),
            address: acc.address.clone(),
            derivation_path: acc.derivation_path.clone(),
            public_key_hex: acc.public_key_hex.clone(),
        };
        db::wallet::insert_account(database.conn(), &row)?;
    }

    let mut s = state.write().await;
    s.db = Some(database);
    s.messaging_keypair = Some(messaging_kp);
    s.wallet = Some(wallet);
    s.vault_unlocked = true;

    Ok(pubkey_hex)
}

// ─── Devices ────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfo {
    pub device_id: String,
    pub device_name: String,
    pub last_seen: Option<String>,
    pub is_current: bool,
}

#[tauri::command]
pub async fn get_devices(state: SharedState) -> AppResult<Vec<DeviceInfo>> {
    let s = state.read().await;
    let db = s.db.as_ref().ok_or(AppError::DatabaseLocked)?;

    let mut stmt = db.conn().prepare(
        "SELECT device_id, device_name, last_seen, is_revoked FROM devices WHERE is_revoked = 0",
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(DeviceInfo {
            device_id: row.get(0)?,
            device_name: row.get(1)?,
            last_seen: row.get(2)?,
            is_current: false,
        })
    })?;

    let mut devices = Vec::new();
    for row in rows {
        devices.push(row?);
    }
    Ok(devices)
}

#[tauri::command]
pub async fn revoke_device(
    state: SharedState,
    device_id: String,
) -> AppResult<()> {
    let s = state.read().await;
    let db = s.db.as_ref().ok_or(AppError::DatabaseLocked)?;

    db.conn().execute(
        "UPDATE devices SET is_revoked = 1 WHERE device_id = ?1",
        rusqlite::params![device_id],
    )?;

    Ok(())
}

// ─── KV Store ───────────────────────────────────────────────────────

#[tauri::command]
pub async fn kv_get(state: SharedState, key: String) -> AppResult<Option<String>> {
    let s = state.read().await;
    let db = s.db.as_ref().ok_or(AppError::DatabaseLocked)?;

    let result: Option<Vec<u8>> = db.conn()
        .query_row(
            "SELECT value FROM kv_store WHERE key = ?1",
            rusqlite::params![key],
            |row| row.get(0),
        )
        .ok();

    Ok(result.map(|b| String::from_utf8_lossy(&b).to_string()))
}

#[tauri::command]
pub async fn kv_set(state: SharedState, key: String, value: String) -> AppResult<()> {
    let s = state.read().await;
    let db = s.db.as_ref().ok_or(AppError::DatabaseLocked)?;

    db.conn().execute(
        "INSERT OR REPLACE INTO kv_store (key, value) VALUES (?1, ?2)",
        rusqlite::params![key, value.as_bytes()],
    )?;

    Ok(())
}

// ─── Search ─────────────────────────────────────────────────────────

#[tauri::command]
pub async fn search_messages_command(
    state: SharedState,
    query: String,
    limit: Option<i64>,
) -> AppResult<Vec<crate::search::SearchResult>> {
    let s = state.read().await;
    let db = s.db.as_ref().ok_or(AppError::DatabaseLocked)?;
    crate::search::search_messages(db.conn(), &query, limit.unwrap_or(20))
}

// ─── Call History ───────────────────────────────────────────────────

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CallRecord {
    pub id: String,
    pub contact_id: String,
    pub contact_name: String,
    pub call_type: String,
    pub direction: String,
    pub duration: i64,
    pub timestamp: String,
}

#[tauri::command]
pub async fn get_call_history(state: SharedState) -> AppResult<Vec<CallRecord>> {
    let s = state.read().await;
    let db = s.db.as_ref().ok_or(AppError::DatabaseLocked)?;

    let mut stmt = db.conn().prepare(
        "SELECT id, contact_id, contact_name, call_type, direction, duration, timestamp \
         FROM call_history ORDER BY timestamp DESC LIMIT 50",
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(CallRecord {
            id: row.get(0)?,
            contact_id: row.get(1)?,
            contact_name: row.get(2)?,
            call_type: row.get(3)?,
            direction: row.get(4)?,
            duration: row.get(5)?,
            timestamp: row.get(6)?,
        })
    })?;

    Ok(rows.filter_map(|r| r.ok()).collect())
}

// ─── Conversation Management ────────────────────────────────────────

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BurnResult {
    pub deleted_count: u64,
}

#[tauri::command]
pub async fn burn_conversation(
    state: SharedState,
    conversation_id: String,
) -> AppResult<BurnResult> {
    let s = state.read().await;
    let database = s.db.as_ref().ok_or(AppError::NotInitialized("Database locked".into()))?;
    let count = db::messages::burn_conversation(database.conn(), &conversation_id)?;
    Ok(BurnResult { deleted_count: count })
}

#[tauri::command]
pub async fn save_draft(
    state: SharedState,
    conversation_id: String,
    text: String,
) -> AppResult<()> {
    let s = state.read().await;
    let database = s.db.as_ref().ok_or(AppError::NotInitialized("Database locked".into()))?;
    database.conn().execute(
        "INSERT OR REPLACE INTO message_drafts (conversation_id, draft_text, updated_at) VALUES (?1, ?2, datetime('now'))",
        rusqlite::params![conversation_id, text],
    )?;
    Ok(())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DraftResult {
    pub text: Option<String>,
}

#[tauri::command]
pub async fn get_draft(
    state: SharedState,
    conversation_id: String,
) -> AppResult<DraftResult> {
    let s = state.read().await;
    let database = s.db.as_ref().ok_or(AppError::NotInitialized("Database locked".into()))?;
    let text: Option<String> = database.conn()
        .query_row(
            "SELECT draft_text FROM message_drafts WHERE conversation_id = ?1",
            rusqlite::params![conversation_id],
            |row| row.get(0),
        )
        .ok();
    Ok(DraftResult { text })
}

#[tauri::command]
pub async fn toggle_star_message(
    state: SharedState,
    message_id: String,
) -> AppResult<bool> {
    let s = state.read().await;
    let database = s.db.as_ref().ok_or(AppError::NotInitialized("Database locked".into()))?;

    let exists: bool = database.conn()
        .query_row(
            "SELECT COUNT(*) FROM starred_messages WHERE message_id = ?1",
            rusqlite::params![&message_id],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0) > 0;

    if exists {
        database.conn().execute(
            "DELETE FROM starred_messages WHERE message_id = ?1",
            rusqlite::params![&message_id],
        )?;
        Ok(false) // unstarred
    } else {
        database.conn().execute(
            "INSERT INTO starred_messages (message_id) VALUES (?1)",
            rusqlite::params![&message_id],
        )?;
        Ok(true) // starred
    }
}

#[tauri::command]
pub async fn toggle_pin_conversation(
    state: SharedState,
    conversation_id: String,
) -> AppResult<bool> {
    let s = state.read().await;
    let database = s.db.as_ref().ok_or(AppError::NotInitialized("Database locked".into()))?;

    let exists: bool = database.conn()
        .query_row(
            "SELECT COUNT(*) FROM pinned_conversations WHERE conversation_id = ?1",
            rusqlite::params![&conversation_id],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0) > 0;

    if exists {
        database.conn().execute(
            "DELETE FROM pinned_conversations WHERE conversation_id = ?1",
            rusqlite::params![&conversation_id],
        )?;
        Ok(false) // unpinned
    } else {
        let max_order: i64 = database.conn()
            .query_row("SELECT COALESCE(MAX(pin_order), 0) FROM pinned_conversations", [], |row| row.get(0))
            .unwrap_or(0);
        database.conn().execute(
            "INSERT INTO pinned_conversations (conversation_id, pin_order) VALUES (?1, ?2)",
            rusqlite::params![&conversation_id, max_order + 1],
        )?;
        Ok(true) // pinned
    }
}

#[tauri::command]
pub async fn toggle_mute_conversation(
    state: SharedState,
    conversation_id: String,
    mute_until: Option<i64>,
) -> AppResult<bool> {
    let s = state.read().await;
    let database = s.db.as_ref().ok_or(AppError::NotInitialized("Database locked".into()))?;

    if let Some(until) = mute_until {
        database.conn().execute(
            "INSERT OR REPLACE INTO muted_conversations (conversation_id, muted_until) VALUES (?1, ?2)",
            rusqlite::params![&conversation_id, until],
        )?;
        Ok(true) // muted
    } else {
        database.conn().execute(
            "DELETE FROM muted_conversations WHERE conversation_id = ?1",
            rusqlite::params![&conversation_id],
        )?;
        Ok(false) // unmuted
    }
}

#[tauri::command]
pub async fn toggle_archive_conversation(
    state: SharedState,
    conversation_id: String,
) -> AppResult<bool> {
    let s = state.read().await;
    let database = s.db.as_ref().ok_or(AppError::NotInitialized("Database locked".into()))?;

    let exists: bool = database.conn()
        .query_row(
            "SELECT COUNT(*) FROM archived_conversations WHERE conversation_id = ?1",
            rusqlite::params![&conversation_id],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0) > 0;

    if exists {
        database.conn().execute(
            "DELETE FROM archived_conversations WHERE conversation_id = ?1",
            rusqlite::params![&conversation_id],
        )?;
        Ok(false) // unarchived
    } else {
        database.conn().execute(
            "INSERT INTO archived_conversations (conversation_id) VALUES (?1)",
            rusqlite::params![&conversation_id],
        )?;
        Ok(true) // archived
    }
}

// ─── Helpers ────────────────────────────────────────────────────────

fn uuid_v4() -> String {
    let mut bytes = [0u8; 16];
    rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut bytes);
    bytes[6] = (bytes[6] & 0x0F) | 0x40;
    bytes[8] = (bytes[8] & 0x3F) | 0x80;
    format!(
        "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
        u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
        u16::from_be_bytes([bytes[4], bytes[5]]),
        u16::from_be_bytes([bytes[6], bytes[7]]),
        u16::from_be_bytes([bytes[8], bytes[9]]),
        u64::from_be_bytes([0, 0, bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]]),
    )
}

fn unix_timestamp_str() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .to_string()
}
