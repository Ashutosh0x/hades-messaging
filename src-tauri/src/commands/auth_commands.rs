//! Tauri commands for the identity creation + authentication flow.
//!
//! - `create_identity`: First launch — generate seed, derive keys, store everything
//! - `unlock_vault`: Subsequent launches — decrypt DB, restore keypair
//! - `restore_identity`: Recovery from 24-word mnemonic

use crate::auth::{self, AuthState};
use crate::db;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use hades_identity::seed::MasterSeed;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;

type SharedState = State<'_, Arc<RwLock<AppState>>>;

// ─── First Launch: Create Identity ─────────────────────────

#[derive(Debug, Serialize)]
pub struct IdentityCreatedResult {
    pub hades_id: String,
    pub hades_id_short: String,
    pub mnemonic: String,
    pub ed25519_public_key: String,
    pub x25519_public_key: String,
    pub wallet_accounts: Vec<WalletAccountResult>,
}

#[derive(Debug, Serialize)]
pub struct WalletAccountResult {
    pub chain: String,
    pub address: String,
    pub ticker: String,
}

/// Called on first launch: generates everything from one BIP-39 seed.
///
/// One seed → messaging identity (Ed25519 + X25519) + all wallet keys (BIP-44).
#[tauri::command]
pub async fn create_identity(
    app_handle: tauri::AppHandle,
    state: SharedState,
    passphrase: String,
    display_name: Option<String>,
) -> AppResult<IdentityCreatedResult> {
    if passphrase.len() < 8 {
        return Err(AppError::Identity(
            "Passphrase must be at least 8 characters".into(),
        ));
    }

    // 1. Generate master seed (24-word BIP-39 mnemonic, 256-bit entropy)
    let master_seed =
        MasterSeed::generate().map_err(|e| AppError::Identity(e.to_string()))?;

    let mnemonic = master_seed.mnemonic().to_string();

    // 2. Derive messaging keypair: m/13'/0'/0' → Ed25519 + X25519 + Hades ID
    let messaging_kp = master_seed
        .derive_messaging_keypair()
        .map_err(|e| AppError::Identity(e.to_string()))?;

    // 3. Derive HD wallet from same seed
    let wallet = hades_wallet::hd::HdWallet::from_mnemonic(&mnemonic)
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let wallet_accounts = wallet.derive_all_accounts();

    // 4. Open encrypted database
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|_| AppError::Internal("Cannot resolve app data dir".into()))?;

    std::fs::create_dir_all(&app_dir)
        .map_err(|e| AppError::Internal(format!("Cannot create dir: {}", e)))?;

    let db_path = app_dir.join("hades.db");
    let database = crate::db::Database::open(db_path, &passphrase)?;

    // 5. Store identity in database
    db::keys::store_identity(
        database.conn(),
        messaging_kp.ed25519_public.as_bytes(),
        &messaging_kp.ed25519_signing.to_bytes(),
    )?;

    // 6. Store mnemonic (encrypted at rest by SQLCipher)
    database.conn().execute(
        "INSERT OR REPLACE INTO kv_store (key, value) VALUES ('master_mnemonic', ?1)",
        rusqlite::params![mnemonic.as_bytes()],
    )?;

    // 7. Store wallet accounts
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

    // 8. Store display name if provided
    if let Some(ref name) = display_name {
        database.conn().execute(
            "INSERT OR REPLACE INTO kv_store (key, value) VALUES ('display_name', ?1)",
            rusqlite::params![name.as_bytes()],
        )?;
    }

    // 9. Build result before moving keypair into state
    let hades_id = messaging_kp.hades_id_hex();
    let hades_id_short = messaging_kp.hades_id_short();
    let ed25519_pub = hex::encode(messaging_kp.ed25519_public.as_bytes());
    let x25519_pub = hex::encode(messaging_kp.x25519_public.as_bytes());

    let wallet_result: Vec<WalletAccountResult> = wallet_accounts
        .iter()
        .map(|a| WalletAccountResult {
            chain: format!("{:?}", a.chain),
            address: a.address.clone(),
            ticker: a.chain.ticker().to_string(),
        })
        .collect();

    // 10. Update app state
    {
        let mut s = state.write().await;
        s.db = Some(database);
        s.messaging_keypair = Some(messaging_kp);
        s.wallet = Some(wallet);
        s.vault_unlocked = true;
        s.display_name = display_name;
        s.app_data_dir = Some(app_dir);
    }

    log::info!("Identity created: hades:{}", hades_id_short);

    Ok(IdentityCreatedResult {
        hades_id,
        hades_id_short,
        mnemonic,
        ed25519_public_key: ed25519_pub,
        x25519_public_key: x25519_pub,
        wallet_accounts: wallet_result,
    })
}

/// Called on subsequent launches: unlock vault with passphrase,
/// restore identity keypair from stored mnemonic.
#[tauri::command]
pub async fn unlock_vault(
    app_handle: tauri::AppHandle,
    state: SharedState,
    passphrase: String,
) -> AppResult<AuthState> {
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|_| AppError::Internal("Cannot resolve app data dir".into()))?;

    let db_path = app_dir.join("hades.db");

    if !db_path.exists() {
        return Err(AppError::NotInitialized(
            "No identity exists. Create one first.".into(),
        ));
    }

    // Open database with passphrase (wrong passphrase = SQLCipher failure)
    let database = crate::db::Database::open(db_path, &passphrase)?;

    // Load mnemonic
    let mnemonic_bytes: Vec<u8> = database
        .conn()
        .query_row(
            "SELECT value FROM kv_store WHERE key = 'master_mnemonic'",
            [],
            |row| row.get(0),
        )
        .map_err(|_| AppError::InvalidPassphrase)?;

    let mnemonic = String::from_utf8(mnemonic_bytes)
        .map_err(|_| AppError::Internal("Corrupted mnemonic".into()))?;

    // Reconstruct master seed and derive messaging keypair
    let master_seed =
        MasterSeed::from_mnemonic(&mnemonic).map_err(|e| AppError::Identity(e.to_string()))?;

    let messaging_kp = master_seed
        .derive_messaging_keypair()
        .map_err(|e| AppError::Identity(e.to_string()))?;

    // Derive wallet
    let wallet = hades_wallet::hd::HdWallet::from_mnemonic(&mnemonic)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    // Load display name
    let display_name: Option<String> = database
        .conn()
        .query_row(
            "SELECT value FROM kv_store WHERE key = 'display_name'",
            [],
            |row| row.get::<_, Vec<u8>>(0),
        )
        .ok()
        .map(|b| String::from_utf8_lossy(&b).to_string());

    let auth_state = auth::auth_state_from_keypair(&messaging_kp, false);

    {
        let mut s = state.write().await;
        s.db = Some(database);
        s.messaging_keypair = Some(messaging_kp);
        s.wallet = Some(wallet);
        s.vault_unlocked = true;
        s.display_name = display_name;
        s.app_data_dir = Some(app_dir);
    }

    log::info!("Vault unlocked: hades:{}", auth_state.hades_id_short);

    Ok(auth_state)
}

/// Restore identity from 24-word recovery phrase.
///
/// Derives the exact same keypair + wallet as the original,
/// creating a fresh encrypted database.
#[tauri::command]
pub async fn restore_identity(
    app_handle: tauri::AppHandle,
    state: SharedState,
    mnemonic: String,
    passphrase: String,
    display_name: Option<String>,
) -> AppResult<AuthState> {
    if passphrase.len() < 8 {
        return Err(AppError::Identity(
            "Passphrase must be at least 8 characters".into(),
        ));
    }

    // Validate mnemonic
    let master_seed = MasterSeed::from_mnemonic(&mnemonic)
        .map_err(|e| AppError::Identity(format!("Invalid recovery phrase: {}", e)))?;

    // Derive everything from the restored seed
    let messaging_kp = master_seed
        .derive_messaging_keypair()
        .map_err(|e| AppError::Identity(e.to_string()))?;

    let wallet = hades_wallet::hd::HdWallet::from_mnemonic(&mnemonic)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let wallet_accounts = wallet.derive_all_accounts();

    // Create fresh database
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|_| AppError::Internal("Cannot resolve app data dir".into()))?;

    std::fs::create_dir_all(&app_dir)
        .map_err(|e| AppError::Internal(format!("Cannot create dir: {}", e)))?;

    let db_path = app_dir.join("hades.db");

    // Remove old database if exists (restore = fresh start)
    if db_path.exists() {
        std::fs::remove_file(&db_path).ok();
    }

    let database = crate::db::Database::open(db_path, &passphrase)?;

    // Store everything
    db::keys::store_identity(
        database.conn(),
        messaging_kp.ed25519_public.as_bytes(),
        &messaging_kp.ed25519_signing.to_bytes(),
    )?;

    database.conn().execute(
        "INSERT OR REPLACE INTO kv_store (key, value) VALUES ('master_mnemonic', ?1)",
        rusqlite::params![mnemonic.as_bytes()],
    )?;

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

    if let Some(ref name) = display_name {
        database.conn().execute(
            "INSERT OR REPLACE INTO kv_store (key, value) VALUES ('display_name', ?1)",
            rusqlite::params![name.as_bytes()],
        )?;
    }

    let auth_state = auth::auth_state_from_keypair(&messaging_kp, false);

    {
        let mut s = state.write().await;
        s.db = Some(database);
        s.messaging_keypair = Some(messaging_kp);
        s.wallet = Some(wallet);
        s.vault_unlocked = true;
        s.display_name = display_name;
        s.app_data_dir = Some(app_dir);
    }

    log::info!("Identity restored: hades:{}", auth_state.hades_id_short);

    Ok(auth_state)
}

/// Check if an identity database exists (for routing to create vs unlock)
#[tauri::command]
pub async fn has_identity(app_handle: tauri::AppHandle) -> AppResult<bool> {
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|_| AppError::Internal("Cannot resolve app data dir".into()))?;

    Ok(app_dir.join("hades.db").exists())
}

/// Get the current auth state (or null if not authenticated)
#[tauri::command]
pub async fn get_auth_state(state: SharedState) -> AppResult<Option<AuthState>> {
    let s = state.read().await;
    Ok(s.messaging_keypair
        .as_ref()
        .map(|kp| auth::auth_state_from_keypair(kp, s.vault_unlocked)))
}
