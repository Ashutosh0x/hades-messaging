//! Tauri commands for contact management via identity bundles.
//!
//! - `get_contact_link`: Generate our shareable hades:// link
//! - `get_contact_qr`: Generate QR code data for our identity
//! - `add_contact_from_bundle`: Add a contact from a link or QR scan
//! - `get_contact_wallet_address`: Get a contact's wallet address for a chain

use crate::contacts::ContactIdentityBundle;
use crate::db;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;

type SharedState = State<'_, Arc<RwLock<AppState>>>;

/// Generate our shareable contact link
#[tauri::command]
pub async fn get_contact_link(state: SharedState) -> AppResult<String> {
    let s = state.read().await;
    let kp = s
        .messaging_keypair
        .as_ref()
        .ok_or(AppError::NotInitialized("No identity".into()))?;

    // Build wallet address map from DB
    let wallet_addrs = if let Some(ref db) = s.db {
        let accounts = db::wallet::get_all_accounts(db.conn())?;
        let mut map = std::collections::HashMap::new();
        for acc in accounts {
            map.insert(acc.chain, acc.address);
        }
        Some(map)
    } else {
        None
    };

    let bundle = ContactIdentityBundle::from_keypair(kp, s.display_name.as_deref(), wallet_addrs);

    Ok(bundle.to_link())
}

/// Generate QR code data for our identity
#[tauri::command]
pub async fn get_contact_qr(state: SharedState) -> AppResult<String> {
    let s = state.read().await;
    let kp = s
        .messaging_keypair
        .as_ref()
        .ok_or(AppError::NotInitialized("No identity".into()))?;

    // Don't include wallet addresses in QR (too large for QR codes)
    let bundle = ContactIdentityBundle::from_keypair(kp, s.display_name.as_deref(), None);

    Ok(bundle.to_qr_data())
}

#[derive(Debug, Serialize)]
pub struct ContactResult {
    pub hades_id: String,
    pub display_name: String,
    pub safety_number: String,
    pub has_wallet_addresses: bool,
}

/// Add a contact from a hades:// link or QR scan data
#[tauri::command]
pub async fn add_contact_from_bundle(
    state: SharedState,
    bundle_data: String,
) -> AppResult<ContactResult> {
    // Parse and verify the bundle
    let bundle = if bundle_data.starts_with("hades://") {
        ContactIdentityBundle::from_link(&bundle_data)?
    } else {
        ContactIdentityBundle::from_qr_data(&bundle_data)?
    };

    let s = state.read().await;
    let db = s.db.as_ref().ok_or(AppError::DatabaseLocked)?;
    let our_kp = s
        .messaging_keypair
        .as_ref()
        .ok_or(AppError::NotInitialized("No identity".into()))?;

    // Don't add ourselves
    if bundle.hades_id == our_kp.hades_id_hex() {
        return Err(AppError::Identity(
            "Cannot add yourself as a contact".into(),
        ));
    }

    // Compute safety number (Signal-style)
    let their_pubkey = hex::decode(&bundle.ed25519_public_key)
        .map_err(|_| AppError::Identity("Invalid public key".into()))?;
    let mut their_pk_arr = [0u8; 32];
    if their_pubkey.len() >= 32 {
        their_pk_arr.copy_from_slice(&their_pubkey[..32]);
    }
    let safety_number = our_kp.safety_number_with(&their_pk_arr);

    // Store contact in DB
    let identity_key = hex::decode(&bundle.x25519_public_key)
        .map_err(|_| AppError::Identity("Invalid X25519 key".into()))?;

    let contact = db::contacts::Contact {
        id: bundle.hades_id.clone(),
        display_name: bundle
            .display_name
            .clone()
            .unwrap_or_else(|| format!("hades:{}", &bundle.hades_id[..8])),
        identity_key,
        safety_number: Some(safety_number.clone()),
        verified: false,
        created_at: String::new(),
    };

    db::contacts::insert_contact(db.conn(), &contact)?;

    // Store their wallet addresses if provided
    if let Some(ref addrs) = bundle.wallet_addresses {
        let addrs_json = serde_json::to_vec(addrs).unwrap_or_default();
        db.conn().execute(
            "INSERT OR REPLACE INTO kv_store (key, value) VALUES (?1, ?2)",
            rusqlite::params![format!("wallet_addrs:{}", bundle.hades_id), addrs_json],
        )?;
    }

    // Store their Ed25519 public key for signature verification
    db.conn().execute(
        "INSERT OR REPLACE INTO kv_store (key, value) VALUES (?1, ?2)",
        rusqlite::params![
            format!("ed25519_pub:{}", bundle.hades_id),
            hex::decode(&bundle.ed25519_public_key).unwrap_or_default(),
        ],
    )?;

    log::info!(
        "Contact added: {} ({})",
        contact.display_name,
        &bundle.hades_id[..8]
    );

    Ok(ContactResult {
        hades_id: bundle.hades_id,
        display_name: contact.display_name,
        safety_number,
        has_wallet_addresses: bundle.wallet_addresses.is_some(),
    })
}

/// Get a contact's wallet address for a specific chain
#[tauri::command]
pub async fn get_contact_wallet_address(
    state: SharedState,
    contact_id: String,
    chain: String,
) -> AppResult<Option<String>> {
    let s = state.read().await;
    let db = s.db.as_ref().ok_or(AppError::DatabaseLocked)?;

    let addrs_bytes: Option<Vec<u8>> = db
        .conn()
        .query_row(
            "SELECT value FROM kv_store WHERE key = ?1",
            rusqlite::params![format!("wallet_addrs:{}", contact_id)],
            |row| row.get(0),
        )
        .ok();

    if let Some(bytes) = addrs_bytes {
        let addrs: std::collections::HashMap<String, String> =
            serde_json::from_slice(&bytes).unwrap_or_default();
        Ok(addrs.get(&chain).cloned())
    } else {
        Ok(None)
    }
}
