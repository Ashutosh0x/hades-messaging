use crate::db::Database;
use crate::websocket::RelayConnection;
use hades_crypto::double_ratchet::RatchetState;
use hades_identity::seed::MessagingKeypair;
use hades_wallet::hd::HdWallet;
use std::collections::HashMap;
use std::path::PathBuf;

/// Top-level application state, held behind Arc<RwLock<>>
pub struct AppState {
    /// SQLCipher database handle (None until unlocked)
    pub db: Option<Database>,

    /// Messaging identity keypair: Ed25519 + X25519 + Hades ID
    /// Derived from BIP-39 seed via m/13'/0'/0'
    pub messaging_keypair: Option<MessagingKeypair>,

    /// HD wallet for multi-chain crypto operations
    /// Derived from the same BIP-39 seed via m/44'/{coin}'/0'/0/0
    pub wallet: Option<HdWallet>,

    /// Active Double Ratchet sessions keyed by contact Hades ID
    pub sessions: HashMap<String, RatchetState>,

    /// WebSocket connection to the relay
    pub relay: Option<RelayConnection>,

    /// Is the vault currently unlocked?
    pub vault_unlocked: bool,

    /// User's chosen display name (optional, stored in kv_store)
    pub display_name: Option<String>,

    /// App data directory (resolved from Tauri on setup)
    pub app_data_dir: Option<PathBuf>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            db: None,
            messaging_keypair: None,
            wallet: None,
            sessions: HashMap::new(),
            relay: None,
            vault_unlocked: false,
            display_name: None,
            app_data_dir: None,
        }
    }
}

impl Drop for AppState {
    fn drop(&mut self) {
        // Zeroize sensitive state on drop
        self.sessions.clear();
        self.messaging_keypair = None;
        self.wallet = None;
        self.vault_unlocked = false;
    }
}
