use crate::db;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use hades_wallet::hd::{Chain, HdWallet};
use hades_wallet::rpc::RpcClient;
use hades_wallet::transaction::{SendParams, TransactionService};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{Emitter, State};
use tokio::sync::RwLock;

type SharedState = State<'_, Arc<RwLock<AppState>>>;

// ─── Types ────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletInitResult {
    pub accounts: Vec<AccountInfo>,
    pub mnemonic: Option<String>,
    pub is_new: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountInfo {
    pub chain: String,
    pub address: String,
    pub ticker: String,
    pub chain_name: String,
    pub derivation_path: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BalanceResult {
    pub chain: String,
    pub symbol: String,
    pub balance: String,
    pub address: String,
    pub usd_value: Option<f64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendRequest {
    pub chain: String,
    pub to_address: String,
    pub amount: String,
    pub token_contract: Option<String>,
    pub conversation_id: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TxResult {
    pub tx_hash: String,
    pub explorer_url: String,
    pub from: String,
    pub to: String,
    pub amount: String,
    pub symbol: String,
    pub chain: String,
    pub status: String,
    pub message_id: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GasInfo {
    pub gas_limit: u64,
    pub gas_price_gwei: String,
    pub estimated_fee: String,
}

// S12/M6 FIX: Three-tier gas estimate matching frontend GasEstimate type
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GasEstimate {
    pub slow: GasTier,
    pub standard: GasTier,
    pub fast: GasTier,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GasTier {
    pub gas_price_gwei: String,
    pub estimated_seconds: u32,
    pub estimated_usd: f64,
}

// ─── Wallet Init ─────────────────────────────────────────────

#[tauri::command]
pub async fn wallet_init(state: SharedState) -> AppResult<WalletInitResult> {
    let mut s = state.write().await;

    if let Some(ref db) = s.db {
        let existing = db::wallet::get_all_accounts(db.conn())?;
        if !existing.is_empty() {
            // Restore wallet from stored mnemonic if needed
            if s.wallet.is_none() {
                let mnemonic_bytes: Vec<u8> = db.conn()
                    .query_row("SELECT value FROM kv_store WHERE key = 'wallet_mnemonic'", [], |row| row.get(0))
                    .map_err(|_| AppError::NotInitialized("No stored mnemonic".into()))?;
                let mnemonic = String::from_utf8(mnemonic_bytes)
                    .map_err(|_| AppError::Internal("Corrupted mnemonic".into()))?;
                let wallet = HdWallet::from_mnemonic(&mnemonic)
                    .map_err(|e| AppError::Internal(e.to_string()))?;
                s.wallet = Some(wallet);
            }
            let accounts = existing.iter().map(|a| {
                let chain = parse_chain(&a.chain);
                AccountInfo {
                    chain: a.chain.clone(), address: a.address.clone(),
                    ticker: chain.map(|c| c.ticker()).unwrap_or("?").to_string(),
                    chain_name: chain.map(|c| c.display_name()).unwrap_or("Unknown").to_string(),
                    derivation_path: a.derivation_path.clone(),
                }
            }).collect();
            return Ok(WalletInitResult { accounts, mnemonic: None, is_new: false });
        }
    }

    let wallet = HdWallet::generate().map_err(|e| AppError::Internal(format!("Wallet generation failed: {}", e)))?;
    let mnemonic = wallet.mnemonic().to_string();
    let all_accounts = wallet.derive_all_accounts();

    if let Some(ref db) = s.db {
        for acc in &all_accounts {
            let row = db::wallet::WalletAccountRow {
                id: 0, chain: format!("{:?}", acc.chain), address: acc.address.clone(),
                derivation_path: acc.derivation_path.clone(), public_key_hex: acc.public_key_hex.clone(),
            };
            db::wallet::insert_account(db.conn(), &row)?;
        }
        db.conn().execute("INSERT OR REPLACE INTO kv_store (key, value) VALUES ('wallet_mnemonic', ?1)",
            rusqlite::params![mnemonic.as_bytes()])?;
    }

    s.wallet = Some(wallet);

    let accounts = all_accounts.iter().map(|a| AccountInfo {
        chain: format!("{:?}", a.chain), address: a.address.clone(),
        ticker: a.chain.ticker().to_string(), chain_name: a.chain.display_name().to_string(),
        derivation_path: a.derivation_path.clone(),
    }).collect();

    Ok(WalletInitResult { accounts, mnemonic: Some(mnemonic), is_new: true })
}

#[tauri::command]
pub async fn wallet_import(state: SharedState, mnemonic: String) -> AppResult<WalletInitResult> {
    let wallet = HdWallet::from_mnemonic(&mnemonic).map_err(|e| AppError::Internal(format!("Invalid mnemonic: {}", e)))?;
    let all_accounts = wallet.derive_all_accounts();
    let mut s = state.write().await;

    if let Some(ref db) = s.db {
        db.conn().execute("DELETE FROM wallet_accounts", [])?;
        for acc in &all_accounts {
            let row = db::wallet::WalletAccountRow {
                id: 0, chain: format!("{:?}", acc.chain), address: acc.address.clone(),
                derivation_path: acc.derivation_path.clone(), public_key_hex: acc.public_key_hex.clone(),
            };
            db::wallet::insert_account(db.conn(), &row)?;
        }
        db.conn().execute("INSERT OR REPLACE INTO kv_store (key, value) VALUES ('wallet_mnemonic', ?1)",
            rusqlite::params![mnemonic.as_bytes()])?;
    }
    s.wallet = Some(wallet);

    let accounts = all_accounts.iter().map(|a| AccountInfo {
        chain: format!("{:?}", a.chain), address: a.address.clone(),
        ticker: a.chain.ticker().to_string(), chain_name: a.chain.display_name().to_string(),
        derivation_path: a.derivation_path.clone(),
    }).collect();
    Ok(WalletInitResult { accounts, mnemonic: None, is_new: false })
}

// ─── Balances ─────────────────────────────────────────────────

#[tauri::command]
pub async fn wallet_get_balance(state: SharedState, chain: String) -> AppResult<BalanceResult> {
    let s = state.read().await;
    let db = s.db.as_ref().ok_or(AppError::DatabaseLocked)?;
    let chain_enum = parse_chain(&chain).ok_or(AppError::Internal(format!("Unknown chain: {}", chain)))?;
    let accounts = db::wallet::get_all_accounts(db.conn())?;
    let account = accounts.iter().find(|a| a.chain == chain).ok_or(AppError::Internal("No account for chain".into()))?;
    let service = TransactionService::new();
    let balance = service.get_native_balance(chain_enum, &account.address).await.map_err(|e| AppError::Network(e.to_string()))?;
    Ok(BalanceResult { chain, symbol: chain_enum.ticker().to_string(), balance, address: account.address.clone(), usd_value: None })
}

#[tauri::command]
pub async fn wallet_get_all_balances(state: SharedState) -> AppResult<Vec<BalanceResult>> {
    let s = state.read().await;
    let db = s.db.as_ref().ok_or(AppError::DatabaseLocked)?;
    let accounts = db::wallet::get_all_accounts(db.conn())?;
    let service = TransactionService::new();
    let mut balances = Vec::new();
    for account in &accounts {
        let chain_enum = match parse_chain(&account.chain) { Some(c) => c, None => continue };
        let balance = match service.get_native_balance(chain_enum, &account.address).await {
            Ok(b) => b,
            Err(e) => { log::warn!("Balance fetch failed for {}: {}", account.chain, e); "0.00".to_string() }
        };
        balances.push(BalanceResult {
            chain: account.chain.clone(), symbol: chain_enum.ticker().to_string(),
            balance, address: account.address.clone(), usd_value: None,
        });
    }
    Ok(balances)
}

// ─── Send ─────────────────────────────────────────────────────

#[tauri::command]
pub async fn wallet_send(state: SharedState, app_handle: tauri::AppHandle, request: SendRequest) -> AppResult<TxResult> {
    let s = state.read().await;
    let wallet = s.wallet.as_ref().ok_or(AppError::NotInitialized("Wallet not loaded".into()))?;
    let db = s.db.as_ref().ok_or(AppError::DatabaseLocked)?;
    let chain_enum = parse_chain(&request.chain).ok_or(AppError::Internal(format!("Unknown chain: {}", request.chain)))?;

    let service = TransactionService::new();
    let params = SendParams {
        chain: chain_enum, to: request.to_address.clone(),
        amount: request.amount.clone(), token_contract: request.token_contract.clone(),
    };
    let result = service.send(wallet, &params).await.map_err(|e| AppError::Network(e.to_string()))?;

    let message_id = if request.conversation_id.is_some() { Some(uuid_v4()) } else { None };

    // Store transaction
    let tx_row = db::wallet::WalletTxRow {
        tx_hash: result.tx_hash.clone(), chain: request.chain.clone(),
        from_address: result.from.clone(), to_address: result.to.clone(),
        amount: result.amount.clone(), symbol: result.symbol.clone(),
        status: "pending".to_string(), explorer_url: Some(result.explorer_url.clone()),
        message_id: message_id.clone(), conversation_id: request.conversation_id.clone(),
        timestamp: unix_timestamp(),
    };
    db::wallet::insert_transaction(db.conn(), &tx_row)?;

    // In-chat transfer message
    if let (Some(ref conv_id), Some(ref msg_id)) = (&request.conversation_id, &message_id) {
        let content = serde_json::json!({
            "type": "crypto_transfer", "chain": request.chain, "symbol": result.symbol,
            "amount": result.amount, "to": result.to, "tx_hash": result.tx_hash,
            "explorer_url": result.explorer_url, "status": "pending"
        });
        let msg = db::messages::StoredMessage {
            id: msg_id.clone(), conversation_id: conv_id.clone(), sender_id: "self".into(),
            content_encrypted: content.to_string().as_bytes().to_vec(), content_nonce: vec![],
            timestamp: unix_timestamp().to_string(), status: "sent".into(),
            burn_after: None, reply_to: None,
        };
        db::messages::insert_message(db.conn(), &msg)?;
    }

    // Background confirmation tracker
    let tx_hash_clone = result.tx_hash.clone();
    let chain_clone = chain_enum;
    let state_clone = state.inner().clone();
    let app_clone = app_handle.clone();
    tokio::spawn(async move {
        let svc = TransactionService::new();
        match svc.wait_for_confirmation(chain_clone, &tx_hash_clone, 300).await {
            Ok(true) => {
                let s = state_clone.read().await;
                if let Some(ref db) = s.db {
                    let _ = db::wallet::update_tx_status(db.conn(), &tx_hash_clone, "confirmed");
                }
                let _ = app_clone.emit("wallet-tx-confirmed", serde_json::json!({"tx_hash": tx_hash_clone, "status": "confirmed"}));
                log::info!("TX confirmed: {}", tx_hash_clone);
            }
            Ok(false) => log::warn!("TX timeout: {}", tx_hash_clone),
            Err(e) => {
                let s = state_clone.read().await;
                if let Some(ref db) = s.db {
                    let _ = db::wallet::update_tx_status(db.conn(), &tx_hash_clone, "failed");
                }
                let _ = app_clone.emit("wallet-tx-failed", serde_json::json!({"tx_hash": tx_hash_clone, "error": e.to_string()}));
            }
        }
    });

    Ok(TxResult {
        tx_hash: result.tx_hash, explorer_url: result.explorer_url,
        from: result.from, to: result.to, amount: result.amount,
        symbol: result.symbol, chain: request.chain, status: "pending".to_string(),
        message_id,
    })
}

// ─── Gas Estimation ───────────────────────────────────────────

#[tauri::command]
pub async fn wallet_estimate_fee(_state: SharedState, chain: String, _to: String, _amount: String) -> AppResult<GasInfo> {
    let chain_enum = parse_chain(&chain).ok_or(AppError::Internal("Unknown chain".into()))?;
    let rpc = RpcClient::new();
    match chain_enum {
        c if c.is_evm() => {
            let gas_price = rpc.eth_gas_price(c).await.map_err(|e| AppError::Network(e.to_string()))?;
            let gas_limit = 21_000u64;
            let fee_eth = (gas_price as f64 * gas_limit as f64) / 1e18;
            Ok(GasInfo { gas_limit, gas_price_gwei: format!("{:.2}", gas_price as f64 / 1e9),
                estimated_fee: format!("{:.6} {}", fee_eth, c.ticker()) })
        }
        Chain::Bitcoin => {
            let fees = rpc.btc_get_fee_estimates().await.map_err(|e| AppError::Network(e.to_string()))?;
            let vbytes = 140u64;
            Ok(GasInfo { gas_limit: vbytes, gas_price_gwei: format!("{} sat/vB", fees.standard),
                estimated_fee: format!("{:.8} BTC", (fees.standard * vbytes) as f64 / 1e8) })
        }
        Chain::Solana => Ok(GasInfo { gas_limit: 1, gas_price_gwei: "5000 lamports".into(),
            estimated_fee: "0.000005 SOL".into() }),
        _ => Err(AppError::Internal("Unsupported chain".into())),
    }
}

// S12 FIX: Three-tier gas estimate matching frontend GasEstimate type
#[tauri::command]
pub async fn wallet_estimate_gas(_state: SharedState, chain: String, _to: String, _amount: String) -> AppResult<GasEstimate> {
    let chain_enum = parse_chain(&chain).ok_or(AppError::Internal("Unknown chain".into()))?;
    let rpc = RpcClient::new();
    match chain_enum {
        c if c.is_evm() => {
            let gas_price = rpc.eth_gas_price(c).await.map_err(|e| AppError::Network(e.to_string()))?;
            let gas_limit = 21_000u64;
            let base_gwei = gas_price as f64 / 1e9;
            let base_fee_eth = (gas_price as f64 * gas_limit as f64) / 1e18;
            // Approximate USD using a rough ETH price (will be replaced by price service)
            let eth_usd = 3500.0;
            Ok(GasEstimate {
                slow: GasTier {
                    gas_price_gwei: format!("{:.1}", base_gwei * 0.8),
                    estimated_seconds: 120,
                    estimated_usd: base_fee_eth * 0.8 * eth_usd,
                },
                standard: GasTier {
                    gas_price_gwei: format!("{:.1}", base_gwei),
                    estimated_seconds: 30,
                    estimated_usd: base_fee_eth * eth_usd,
                },
                fast: GasTier {
                    gas_price_gwei: format!("{:.1}", base_gwei * 1.5),
                    estimated_seconds: 12,
                    estimated_usd: base_fee_eth * 1.5 * eth_usd,
                },
            })
        }
        Chain::Bitcoin => {
            let fees = rpc.btc_get_fee_estimates().await.map_err(|e| AppError::Network(e.to_string()))?;
            let vbytes = 140u64;
            let btc_usd = 68000.0;
            Ok(GasEstimate {
                slow: GasTier {
                    gas_price_gwei: format!("{} sat/vB", fees.slow),
                    estimated_seconds: 3600,
                    estimated_usd: (fees.slow * vbytes) as f64 / 1e8 * btc_usd,
                },
                standard: GasTier {
                    gas_price_gwei: format!("{} sat/vB", fees.standard),
                    estimated_seconds: 600,
                    estimated_usd: (fees.standard * vbytes) as f64 / 1e8 * btc_usd,
                },
                fast: GasTier {
                    gas_price_gwei: format!("{} sat/vB", fees.fast),
                    estimated_seconds: 60,
                    estimated_usd: (fees.fast * vbytes) as f64 / 1e8 * btc_usd,
                },
            })
        }
        Chain::Solana => Ok(GasEstimate {
            slow: GasTier { gas_price_gwei: "5000 lamports".into(), estimated_seconds: 5, estimated_usd: 0.001 },
            standard: GasTier { gas_price_gwei: "5000 lamports".into(), estimated_seconds: 2, estimated_usd: 0.001 },
            fast: GasTier { gas_price_gwei: "10000 lamports".into(), estimated_seconds: 1, estimated_usd: 0.002 },
        }),
        _ => Err(AppError::Internal("Unsupported chain".into())),
    }
}

// ─── History & Addresses ──────────────────────────────────────

#[tauri::command]
pub async fn wallet_get_transactions(state: SharedState, chain: Option<String>, limit: Option<i64>) -> AppResult<Vec<db::wallet::WalletTxRow>> {
    let s = state.read().await;
    let db = s.db.as_ref().ok_or(AppError::DatabaseLocked)?;
    db::wallet::get_transactions(db.conn(), chain.as_deref(), limit.unwrap_or(50))
}

#[tauri::command]
pub async fn wallet_get_address(state: SharedState, chain: String) -> AppResult<String> {
    let s = state.read().await;
    let db = s.db.as_ref().ok_or(AppError::DatabaseLocked)?;
    let accounts = db::wallet::get_all_accounts(db.conn())?;
    let acc = accounts.iter().find(|a| a.chain == chain).ok_or(AppError::Internal("No account".into()))?;
    Ok(acc.address.clone())
}

#[tauri::command]
pub async fn wallet_get_all_addresses(state: SharedState) -> AppResult<Vec<AccountInfo>> {
    let s = state.read().await;
    let db = s.db.as_ref().ok_or(AppError::DatabaseLocked)?;
    let accounts = db::wallet::get_all_accounts(db.conn())?;
    Ok(accounts.iter().map(|a| {
        let chain = parse_chain(&a.chain);
        AccountInfo {
            chain: a.chain.clone(), address: a.address.clone(),
            ticker: chain.map(|c| c.ticker()).unwrap_or("?").to_string(),
            chain_name: chain.map(|c| c.display_name()).unwrap_or("Unknown").to_string(),
            derivation_path: a.derivation_path.clone(),
        }
    }).collect())
}

#[tauri::command]
pub async fn wallet_export_mnemonic(state: SharedState) -> AppResult<String> {
    let s = state.read().await;
    if let Some(ref wallet) = s.wallet {
        let mnemonic = wallet.mnemonic();
        if !mnemonic.is_empty() { return Ok(mnemonic.to_string()); }
    }
    if let Some(ref db) = s.db {
        let mut stmt = db.conn().prepare("SELECT value FROM kv_store WHERE key = 'wallet_mnemonic'")?;
        let mnemonic: Vec<u8> = stmt.query_row([], |row| row.get(0))?;
        return Ok(String::from_utf8_lossy(&mnemonic).to_string());
    }
    Err(AppError::NotInitialized("No wallet".into()))
}

// ─── Helpers ──────────────────────────────────────────────────

fn parse_chain(s: &str) -> Option<Chain> {
    match s {
        "Bitcoin" => Some(Chain::Bitcoin), "Ethereum" => Some(Chain::Ethereum),
        "Solana" => Some(Chain::Solana), "Polygon" => Some(Chain::Polygon),
        "Arbitrum" => Some(Chain::Arbitrum), "Optimism" => Some(Chain::Optimism),
        "Avalanche" => Some(Chain::Avalanche), "Base" => Some(Chain::Base),
        "BnbSmartChain" => Some(Chain::BnbSmartChain), "Litecoin" => Some(Chain::Litecoin),
        "Dogecoin" => Some(Chain::Dogecoin), "Tron" => Some(Chain::Tron),
        _ => None,
    }
}

fn uuid_v4() -> String {
    let mut bytes = [0u8; 16];
    rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut bytes);
    bytes[6] = (bytes[6] & 0x0F) | 0x40;
    bytes[8] = (bytes[8] & 0x3F) | 0x80;
    format!("{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
        u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
        u16::from_be_bytes([bytes[4], bytes[5]]),
        u16::from_be_bytes([bytes[6], bytes[7]]),
        u16::from_be_bytes([bytes[8], bytes[9]]),
        u64::from_be_bytes([0, 0, bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]]))
}

fn unix_timestamp() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as i64
}
