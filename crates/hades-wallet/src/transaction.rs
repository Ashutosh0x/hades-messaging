use crate::chains::ethereum::{self, EvmTxParams, U256};
use crate::chains::{bitcoin, solana};
use crate::error::WalletError;
use crate::hd::{Chain, HdWallet};
use crate::rpc::RpcClient;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendParams {
    pub chain: Chain,
    pub to: String,
    pub amount: String, // Human-readable: "0.5"
    pub token_contract: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendResult {
    pub tx_hash: String,
    pub chain: Chain,
    pub from: String,
    pub to: String,
    pub amount: String,
    pub symbol: String,
    pub explorer_url: String,
    pub raw_tx_hex: String,
}

pub struct TransactionService {
    rpc: RpcClient,
}

impl TransactionService {
    pub fn new() -> Self { Self { rpc: RpcClient::new() } }
    pub fn with_rpc(rpc: RpcClient) -> Self { Self { rpc } }

    pub async fn send(&self, wallet: &HdWallet, params: &SendParams) -> Result<SendResult, WalletError> {
        match params.chain {
            c if c.is_evm() => {
                if let Some(ref contract) = params.token_contract {
                    self.send_erc20(wallet, c, &params.to, &params.amount, contract).await
                } else {
                    self.send_evm_native(wallet, c, &params.to, &params.amount).await
                }
            }
            Chain::Bitcoin => self.send_btc(wallet, &params.to, &params.amount).await,
            Chain::Solana => self.send_sol(wallet, &params.to, &params.amount).await,
            _ => Err(WalletError::UnsupportedChain(params.chain.display_name().into())),
        }
    }

    async fn send_evm_native(&self, wallet: &HdWallet, chain: Chain, to: &str, amount: &str) -> Result<SendResult, WalletError> {
        let chain_id = chain.chain_id().ok_or(WalletError::UnsupportedChain("Not EVM".into()))?;
        let secret = wallet.derive_secret(chain, 0)?;
        let account = wallet.derive_account(chain, 0)?;

        let to_hex = to.trim_start_matches("0x");
        let to_bytes = hex::decode(to_hex).map_err(|_| WalletError::InvalidAddress(to.into()))?;
        if to_bytes.len() != 20 { return Err(WalletError::InvalidAddress("Address must be 20 bytes".into())); }
        let mut to_addr = [0u8; 20];
        to_addr.copy_from_slice(&to_bytes);

        let nonce = self.rpc.eth_get_nonce(chain, &account.address).await?;
        let base_fee = self.rpc.eth_gas_price(chain).await?;
        let priority_fee = self.rpc.eth_max_priority_fee(chain).await.unwrap_or(2_000_000_000);
        let value = U256::from_human(amount, chain.decimals())?;

        let balance = self.rpc.eth_get_balance(chain, &account.address).await?;
        let value_u128 = bytes_to_u128(&value.to_minimal_bytes());
        let fee_est = (base_fee as u128 + priority_fee as u128) * 21_000;
        if balance < value_u128 + fee_est {
            return Err(WalletError::InsufficientBalance {
                have: format_balance(balance, chain.decimals()),
                need: format!("{} + gas", amount),
            });
        }

        let gas_limit = self.rpc.eth_estimate_gas(chain, &account.address, to,
            &format!("0x{}", hex::encode(value.to_minimal_bytes())), "0x").await.unwrap_or(21_000);
        let gas_limit = (gas_limit as f64 * 1.2) as u64;

        let tx_params = EvmTxParams {
            chain_id, nonce, to: to_addr, value, data: vec![],
            max_fee_per_gas: base_fee + priority_fee * 2,
            max_priority_fee_per_gas: priority_fee, gas_limit,
        };
        let (raw_tx, _tx_hash) = ethereum::sign_eip1559_tx(&secret.bytes, &tx_params)?;
        let returned_hash = self.rpc.eth_send_raw_tx(chain, &raw_tx).await?;
        log::info!("[{}] TX broadcast: {}", chain.display_name(), returned_hash);

        Ok(SendResult {
            tx_hash: returned_hash.clone(), chain, from: account.address,
            to: to.to_string(), amount: amount.to_string(),
            symbol: chain.ticker().to_string(),
            explorer_url: chain.explorer_tx_url(&returned_hash),
            raw_tx_hex: hex::encode(&raw_tx),
        })
    }

    async fn send_erc20(&self, wallet: &HdWallet, chain: Chain, to: &str, amount: &str, contract_address: &str) -> Result<SendResult, WalletError> {
        let chain_id = chain.chain_id().ok_or(WalletError::UnsupportedChain("Not EVM".into()))?;
        let secret = wallet.derive_secret(chain, 0)?;
        let account = wallet.derive_account(chain, 0)?;
        let decimals = self.get_erc20_decimals(chain, contract_address).await?;

        let to_hex = to.trim_start_matches("0x");
        let to_bytes = hex::decode(to_hex).map_err(|_| WalletError::InvalidAddress(to.into()))?;
        let mut to_addr = [0u8; 20];
        to_addr.copy_from_slice(&to_bytes);

        let contract_hex = contract_address.trim_start_matches("0x");
        let contract_bytes = hex::decode(contract_hex).map_err(|_| WalletError::InvalidAddress(contract_address.into()))?;
        let mut contract_addr = [0u8; 20];
        contract_addr.copy_from_slice(&contract_bytes);

        let value = U256::from_human(amount, decimals)?;
        let calldata = ethereum::erc20_transfer_data(&to_addr, &value);

        let nonce = self.rpc.eth_get_nonce(chain, &account.address).await?;
        let base_fee = self.rpc.eth_gas_price(chain).await?;
        let priority_fee = self.rpc.eth_max_priority_fee(chain).await.unwrap_or(2_000_000_000);
        let gas_limit = self.rpc.eth_estimate_gas(chain, &account.address, contract_address,
            "0x0", &format!("0x{}", hex::encode(&calldata))).await.unwrap_or(65_000);
        let gas_limit = (gas_limit as f64 * 1.3) as u64;

        let tx_params = EvmTxParams {
            chain_id, nonce, to: contract_addr, value: U256::zero(),
            data: calldata, max_fee_per_gas: base_fee + priority_fee * 2,
            max_priority_fee_per_gas: priority_fee, gas_limit,
        };
        let (raw_tx, _) = ethereum::sign_eip1559_tx(&secret.bytes, &tx_params)?;
        let returned_hash = self.rpc.eth_send_raw_tx(chain, &raw_tx).await?;
        log::info!("[{}] ERC-20 TX: {}", chain.display_name(), returned_hash);

        Ok(SendResult {
            tx_hash: returned_hash.clone(), chain, from: account.address,
            to: to.to_string(), amount: amount.to_string(), symbol: "TOKEN".to_string(),
            explorer_url: chain.explorer_tx_url(&returned_hash),
            raw_tx_hex: hex::encode(&raw_tx),
        })
    }

    async fn get_erc20_decimals(&self, chain: Chain, contract: &str) -> Result<u8, WalletError> {
        let result = self.rpc.eth_call(chain, contract, "0x313ce567").await?;
        let hex = result.trim_start_matches("0x");
        if hex.len() >= 64 { Ok(u8::from_str_radix(&hex[62..64], 16).unwrap_or(18)) } else { Ok(18) }
    }

    async fn send_btc(&self, wallet: &HdWallet, to: &str, amount: &str) -> Result<SendResult, WalletError> {
        let secret = wallet.derive_secret(Chain::Bitcoin, 0)?;
        let account = wallet.derive_account(Chain::Bitcoin, 0)?;
        let amount_sats = parse_btc_amount(amount)?;
        let utxos = self.rpc.btc_get_utxos(&account.address).await?;
        if utxos.is_empty() {
            return Err(WalletError::InsufficientBalance { have: "0 sats".into(), need: format!("{} sats", amount_sats) });
        }
        let fees = self.rpc.btc_get_fee_estimates().await?;
        let (raw_tx, _txid) = bitcoin::build_and_sign_btc_tx(&secret.bytes, &utxos, to, amount_sats, fees.standard, ::bitcoin::Network::Bitcoin)?;
        let returned_txid = self.rpc.btc_broadcast(&raw_tx).await?;
        log::info!("[Bitcoin] TX: {}", returned_txid);
        Ok(SendResult {
            tx_hash: returned_txid.clone(), chain: Chain::Bitcoin, from: account.address,
            to: to.to_string(), amount: amount.to_string(), symbol: "BTC".to_string(),
            explorer_url: Chain::Bitcoin.explorer_tx_url(&returned_txid),
            raw_tx_hex: hex::encode(&raw_tx),
        })
    }

    async fn send_sol(&self, wallet: &HdWallet, to: &str, amount: &str) -> Result<SendResult, WalletError> {
        let secret = wallet.derive_secret(Chain::Solana, 0)?;
        let account = wallet.derive_account(Chain::Solana, 0)?;
        let lamports = parse_sol_amount(amount)?;
        let balance = self.rpc.sol_get_balance(&account.address).await?;
        if balance < lamports + 5000 {
            return Err(WalletError::InsufficientBalance { have: format!("{} lamports", balance), need: format!("{} lamports", lamports + 5000) });
        }
        let blockhash = self.rpc.sol_get_recent_blockhash().await?;
        let (raw_tx, _signature) = solana::build_and_sign_sol_tx(&secret.bytes, to, lamports, &blockhash)?;
        let returned_sig = self.rpc.sol_send_tx(&raw_tx).await?;
        log::info!("[Solana] TX: {}", returned_sig);
        Ok(SendResult {
            tx_hash: returned_sig.clone(), chain: Chain::Solana, from: account.address,
            to: to.to_string(), amount: amount.to_string(), symbol: "SOL".to_string(),
            explorer_url: Chain::Solana.explorer_tx_url(&returned_sig),
            raw_tx_hex: hex::encode(&raw_tx),
        })
    }

    pub async fn get_native_balance(&self, chain: Chain, address: &str) -> Result<String, WalletError> {
        match chain {
            c if c.is_evm() => { let wei = self.rpc.eth_get_balance(c, address).await?; Ok(format_balance(wei, c.decimals())) }
            Chain::Bitcoin => { let sats = self.rpc.btc_get_balance(address).await?; Ok(format_balance(sats as u128, 8)) }
            Chain::Solana => { let lam = self.rpc.sol_get_balance(address).await?; Ok(format_balance(lam as u128, 9)) }
            _ => Err(WalletError::UnsupportedChain(chain.display_name().into())),
        }
    }

    pub async fn wait_for_confirmation(&self, chain: Chain, tx_hash: &str, timeout_secs: u64) -> Result<bool, WalletError> {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(timeout_secs);
        loop {
            if start.elapsed() > timeout { return Ok(false); }
            let confirmed = match chain {
                c if c.is_evm() => {
                    let receipt = self.rpc.eth_get_tx_receipt(c, tx_hash).await?;
                    receipt.map(|r| r["status"].as_str() == Some("0x1")).unwrap_or(false)
                }
                Chain::Solana => {
                    let status = self.rpc.sol_get_tx_status(tx_hash).await?;
                    status.as_deref() == Some("confirmed")
                }
                _ => return Ok(false),
            };
            if confirmed { return Ok(true); }
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        }
    }
}

impl Default for TransactionService {
    fn default() -> Self { Self::new() }
}

fn parse_btc_amount(amount: &str) -> Result<u64, WalletError> {
    let parts: Vec<&str> = amount.split('.').collect();
    let whole: u64 = parts[0].parse().map_err(|_| WalletError::Internal("Invalid amount".into()))?;
    let frac: u64 = if parts.len() > 1 {
        let s = format!("{:0<8}", parts[1]);
        s[..8].parse().unwrap_or(0)
    } else { 0 };
    Ok(whole * 100_000_000 + frac)
}

fn parse_sol_amount(amount: &str) -> Result<u64, WalletError> {
    let parts: Vec<&str> = amount.split('.').collect();
    let whole: u64 = parts[0].parse().map_err(|_| WalletError::Internal("Invalid amount".into()))?;
    let frac: u64 = if parts.len() > 1 {
        let s = format!("{:0<9}", parts[1]);
        s[..9].parse().unwrap_or(0)
    } else { 0 };
    Ok(whole * 1_000_000_000 + frac)
}

fn format_balance(raw: u128, decimals: u8) -> String {
    let divisor = 10u128.pow(decimals as u32);
    let whole = raw / divisor;
    let frac = raw % divisor;
    let frac_str = format!("{:0>width$}", frac, width = decimals as usize);
    let trimmed = frac_str.trim_end_matches('0');
    let frac_display = if trimmed.len() < 2 { &frac_str[..2] } else { trimmed };
    format!("{}.{}", whole, frac_display)
}

fn bytes_to_u128(bytes: &[u8]) -> u128 {
    let mut arr = [0u8; 16];
    let start = 16usize.saturating_sub(bytes.len());
    let len = bytes.len().min(16);
    arr[start..start + len].copy_from_slice(&bytes[..len]);
    u128::from_be_bytes(arr)
}
