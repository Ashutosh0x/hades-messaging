mod auth;
mod biometric;
mod burn_timer;
#[path = "commands.rs"]
mod legacy_commands;
mod contacts;
mod db;
mod error;
mod media;
mod notifications;
mod pipeline;
mod receipts;
mod search;
mod state;
mod typing;
mod wallet_commands;
mod websocket;

// New modular auth & contact commands
#[path = "commands/auth_commands.rs"]
mod auth_commands;
#[path = "commands/contact_commands.rs"]
mod contact_commands;

use state::AppState;
use std::sync::Arc;
use tokio::sync::RwLock;

pub fn run() {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info"),
    )
    .init();

    let app_state = Arc::new(RwLock::new(AppState::default()));

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(app_state.clone())
        .invoke_handler(tauri::generate_handler![
            // ─── New Auth Commands ──────────────────────────────
            auth_commands::create_identity,
            auth_commands::unlock_vault,
            auth_commands::restore_identity,
            auth_commands::has_identity,
            auth_commands::get_auth_state,
            // ─── New Contact Commands ───────────────────────────
            contact_commands::get_contact_link,
            contact_commands::get_contact_qr,
            contact_commands::add_contact_from_bundle,
            contact_commands::get_contact_wallet_address,
            // ─── Legacy Identity ────────────────────────────────
            legacy_commands::generate_identity,
            legacy_commands::get_identity_pubkey,
            legacy_commands::get_safety_number,
            // Keys
            legacy_commands::generate_x25519_keypair,
            legacy_commands::generate_signed_prekey,
            legacy_commands::generate_prekey_bundle,
            // Sessions
            legacy_commands::initiate_session,
            legacy_commands::has_session,
            // Messages
            legacy_commands::send_message,
            legacy_commands::get_messages,
            legacy_commands::mark_message_read,
            // Contacts
            legacy_commands::add_contact,
            legacy_commands::get_contacts,
            legacy_commands::delete_contact,
            // Database
            legacy_commands::initialize_database,
            legacy_commands::lock_database,
            legacy_commands::unlock_database,
            // Network
            legacy_commands::connect_relay,
            legacy_commands::disconnect_relay,
            legacy_commands::get_connection_status,
            // Anti-forensics
            legacy_commands::emergency_wipe,
            // Recovery
            legacy_commands::generate_recovery_phrase,
            legacy_commands::restore_from_recovery,
            // Device management
            legacy_commands::get_devices,
            legacy_commands::revoke_device,
            // Wallet
            wallet_commands::wallet_init,
            wallet_commands::wallet_import,
            wallet_commands::wallet_get_balance,
            wallet_commands::wallet_get_all_balances,
            wallet_commands::wallet_send,
            wallet_commands::wallet_get_transactions,
            wallet_commands::wallet_get_address,
            wallet_commands::wallet_get_all_addresses,
            wallet_commands::wallet_estimate_fee,
            wallet_commands::wallet_export_mnemonic,
            // Biometric
            biometric::biometric_available,
            biometric::biometric_authenticate,
            // Push
            notifications::register_push,
        ])
        .setup(move |app| {
            log::info!("Hades Messaging starting...");

            // Store app data dir in state
            {
                let state = app_state.clone();
                let handle = app.handle().clone();
                if let Ok(app_dir) = handle.path().app_data_dir() {
                    let rt = tokio::runtime::Handle::current();
                    rt.block_on(async {
                        let mut s = state.write().await;
                        s.app_data_dir = Some(app_dir);
                    });
                }
            }

            // Start burn timer background task
            let state_clone = app_state.clone();
            let handle = app.handle().clone();
            tokio::spawn(async move {
                burn_timer::burn_timer_loop(state_clone, handle).await;
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Hades");
}
