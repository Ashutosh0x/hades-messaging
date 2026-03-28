use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBalance {
    pub chain: crate::hd::Chain,
    pub symbol: String,
    pub name: String,
    pub balance: String,
    pub balance_raw: String,
    pub decimals: u8,
    pub contract_address: Option<String>,
    pub usd_value: Option<f64>,
    pub icon_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionRequest {
    pub chain: crate::hd::Chain,
    pub from: String,
    pub to: String,
    pub amount: String,
    pub token_contract: Option<String>,
    pub gas_limit: Option<u64>,
    pub gas_price: Option<String>,
    pub memo: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionResult {
    pub tx_hash: String,
    pub chain: crate::hd::Chain,
    pub from: String,
    pub to: String,
    pub amount: String,
    pub symbol: String,
    pub status: TxStatus,
    pub explorer_url: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TxStatus {
    Pending,
    Confirmed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasEstimate {
    pub slow: GasTier,
    pub standard: GasTier,
    pub fast: GasTier,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasTier {
    pub gas_price_gwei: String,
    pub estimated_seconds: u32,
    pub estimated_usd: f64,
}
