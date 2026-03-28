use crate::error::WalletError;
use crate::hd::Chain;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;

/// Multi-endpoint RPC client with failover and retries
pub struct RpcClient {
    client: reqwest::Client,
    endpoints: HashMap<Chain, Vec<String>>,
}

impl RpcClient {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(5)
            .build()
            .expect("Failed to build HTTP client");

        let mut endpoints: HashMap<Chain, Vec<String>> = HashMap::new();
        endpoints.insert(Chain::Ethereum, vec![
            "https://eth.llamarpc.com".into(), "https://rpc.ankr.com/eth".into(),
            "https://ethereum.publicnode.com".into(),
        ]);
        endpoints.insert(Chain::Polygon, vec![
            "https://polygon-rpc.com".into(), "https://rpc.ankr.com/polygon".into(),
        ]);
        endpoints.insert(Chain::Arbitrum, vec![
            "https://arb1.arbitrum.io/rpc".into(), "https://rpc.ankr.com/arbitrum".into(),
        ]);
        endpoints.insert(Chain::Optimism, vec![
            "https://mainnet.optimism.io".into(), "https://rpc.ankr.com/optimism".into(),
        ]);
        endpoints.insert(Chain::Base, vec![
            "https://mainnet.base.org".into(), "https://base.llamarpc.com".into(),
        ]);
        endpoints.insert(Chain::Avalanche, vec![
            "https://api.avax.network/ext/bc/C/rpc".into(), "https://rpc.ankr.com/avalanche".into(),
        ]);
        endpoints.insert(Chain::BnbSmartChain, vec![
            "https://bsc-dataseed.binance.org".into(), "https://rpc.ankr.com/bsc".into(),
        ]);
        endpoints.insert(Chain::Solana, vec!["https://api.mainnet-beta.solana.com".into()]);
        endpoints.insert(Chain::Bitcoin, vec![
            "https://blockstream.info/api".into(), "https://mempool.space/api".into(),
        ]);
        Self { client, endpoints }
    }

    pub fn set_endpoint(&mut self, chain: Chain, url: String) {
        self.endpoints.insert(chain, vec![url]);
    }

    async fn jsonrpc(&self, chain: Chain, method: &str, params: Value) -> Result<Value, WalletError> {
        let eps = self.endpoints.get(&chain)
            .ok_or(WalletError::UnsupportedChain(format!("{:?}", chain)))?;
        let body = serde_json::json!({"jsonrpc":"2.0","method":method,"params":params,"id":1});
        let mut last_err = WalletError::RpcError("No endpoints".into());
        for ep in eps {
            for attempt in 0..3u64 {
                match self.client.post(ep).json(&body).send().await {
                    Ok(resp) => match resp.json::<Value>().await {
                        Ok(json) => {
                            if let Some(e) = json.get("error") {
                                last_err = WalletError::RpcError(e["message"].as_str().unwrap_or("RPC error").to_string());
                                continue;
                            }
                            return Ok(json["result"].clone());
                        }
                        Err(e) => last_err = WalletError::RpcError(e.to_string()),
                    },
                    Err(e) => {
                        last_err = WalletError::RpcError(e.to_string());
                        if attempt < 2 { tokio::time::sleep(Duration::from_millis(500 * (attempt + 1))).await; }
                    }
                }
            }
        }
        Err(last_err)
    }

    async fn rest_get(&self, chain: Chain, path: &str) -> Result<Value, WalletError> {
        let eps = self.endpoints.get(&chain)
            .ok_or(WalletError::UnsupportedChain(format!("{:?}", chain)))?;
        let mut last_err = WalletError::RpcError("No endpoints".into());
        for ep in eps {
            let url = format!("{}{}", ep, path);
            for attempt in 0..3u64 {
                match self.client.get(&url).send().await {
                    Ok(resp) if resp.status().is_success() => match resp.json::<Value>().await {
                        Ok(json) => return Ok(json),
                        Err(e) => last_err = WalletError::RpcError(e.to_string()),
                    },
                    Ok(resp) => last_err = WalletError::RpcError(format!("HTTP {}", resp.status())),
                    Err(e) => {
                        last_err = WalletError::RpcError(e.to_string());
                        if attempt < 2 { tokio::time::sleep(Duration::from_millis(500 * (attempt + 1))).await; }
                    }
                }
            }
        }
        Err(last_err)
    }

    // ─── EVM ────────────────────────────────────────────────
    pub async fn eth_get_balance(&self, chain: Chain, address: &str) -> Result<u128, WalletError> {
        let r = self.jsonrpc(chain, "eth_getBalance", serde_json::json!([address, "latest"])).await?;
        let hex = r.as_str().unwrap_or("0x0").trim_start_matches("0x");
        Ok(u128::from_str_radix(hex, 16).unwrap_or(0))
    }

    pub async fn eth_get_nonce(&self, chain: Chain, address: &str) -> Result<u64, WalletError> {
        let r = self.jsonrpc(chain, "eth_getTransactionCount", serde_json::json!([address, "pending"])).await?;
        let hex = r.as_str().unwrap_or("0x0").trim_start_matches("0x");
        Ok(u64::from_str_radix(hex, 16).unwrap_or(0))
    }

    pub async fn eth_gas_price(&self, chain: Chain) -> Result<u64, WalletError> {
        let r = self.jsonrpc(chain, "eth_gasPrice", serde_json::json!([])).await?;
        let hex = r.as_str().unwrap_or("0x0").trim_start_matches("0x");
        Ok(u64::from_str_radix(hex, 16).unwrap_or(20_000_000_000))
    }

    pub async fn eth_max_priority_fee(&self, chain: Chain) -> Result<u64, WalletError> {
        let r = self.jsonrpc(chain, "eth_maxPriorityFeePerGas", serde_json::json!([])).await?;
        let hex = r.as_str().unwrap_or("0x77359400").trim_start_matches("0x");
        Ok(u64::from_str_radix(hex, 16).unwrap_or(2_000_000_000))
    }

    pub async fn eth_estimate_gas(&self, chain: Chain, from: &str, to: &str, value: &str, data: &str) -> Result<u64, WalletError> {
        let r = self.jsonrpc(chain, "eth_estimateGas", serde_json::json!([{"from":from,"to":to,"value":value,"data":data}])).await?;
        let hex = r.as_str().unwrap_or("0x5208").trim_start_matches("0x");
        Ok(u64::from_str_radix(hex, 16).unwrap_or(21_000))
    }

    pub async fn eth_send_raw_tx(&self, chain: Chain, raw_tx: &[u8]) -> Result<String, WalletError> {
        let r = self.jsonrpc(chain, "eth_sendRawTransaction", serde_json::json!([format!("0x{}", hex::encode(raw_tx))])).await?;
        r.as_str().map(|s| s.to_string()).ok_or(WalletError::BroadcastFailed("No tx hash".into()))
    }

    pub async fn eth_get_tx_receipt(&self, chain: Chain, tx_hash: &str) -> Result<Option<Value>, WalletError> {
        let r = self.jsonrpc(chain, "eth_getTransactionReceipt", serde_json::json!([tx_hash])).await?;
        if r.is_null() { Ok(None) } else { Ok(Some(r)) }
    }

    pub async fn eth_call(&self, chain: Chain, to: &str, data: &str) -> Result<String, WalletError> {
        let r = self.jsonrpc(chain, "eth_call", serde_json::json!([{"to":to,"data":data}, "latest"])).await?;
        Ok(r.as_str().unwrap_or("0x").to_string())
    }

    // ─── Bitcoin ────────────────────────────────────────────
    pub async fn btc_get_utxos(&self, address: &str) -> Result<Vec<crate::chains::bitcoin::Utxo>, WalletError> {
        let r = self.rest_get(Chain::Bitcoin, &format!("/address/{}/utxo", address)).await?;
        serde_json::from_value(r).map_err(|e| WalletError::RpcError(e.to_string()))
    }

    pub async fn btc_get_fee_estimates(&self) -> Result<BtcFeeEstimates, WalletError> {
        let r = self.rest_get(Chain::Bitcoin, "/fee-estimates").await?;
        Ok(BtcFeeEstimates {
            fast: r["1"].as_f64().unwrap_or(50.0) as u64,
            standard: r["3"].as_f64().unwrap_or(30.0) as u64,
            slow: r["6"].as_f64().unwrap_or(15.0) as u64,
        })
    }

    pub async fn btc_broadcast(&self, raw_tx: &[u8]) -> Result<String, WalletError> {
        let eps = self.endpoints.get(&Chain::Bitcoin).ok_or(WalletError::UnsupportedChain("Bitcoin".into()))?;
        let hex_tx = hex::encode(raw_tx);
        for ep in eps {
            let url = format!("{}/tx", ep);
            if let Ok(resp) = self.client.post(&url).header("Content-Type", "text/plain").body(hex_tx.clone()).send().await {
                if resp.status().is_success() {
                    if let Ok(txid) = resp.text().await {
                        return Ok(txid.trim().to_string());
                    }
                }
            }
        }
        Err(WalletError::BroadcastFailed("All endpoints failed".into()))
    }

    pub async fn btc_get_balance(&self, address: &str) -> Result<u64, WalletError> {
        let utxos = self.btc_get_utxos(address).await?;
        Ok(utxos.iter().map(|u| u.value).sum())
    }

    // ─── Solana ─────────────────────────────────────────────
    pub async fn sol_get_balance(&self, address: &str) -> Result<u64, WalletError> {
        let r = self.jsonrpc(Chain::Solana, "getBalance", serde_json::json!([address])).await?;
        Ok(r["value"].as_u64().unwrap_or(0))
    }

    pub async fn sol_get_recent_blockhash(&self) -> Result<String, WalletError> {
        let r = self.jsonrpc(Chain::Solana, "getLatestBlockhash", serde_json::json!([{"commitment":"finalized"}])).await?;
        r["value"]["blockhash"].as_str().map(|s| s.to_string()).ok_or(WalletError::RpcError("No blockhash".into()))
    }

    pub async fn sol_send_tx(&self, raw_tx: &[u8]) -> Result<String, WalletError> {
        use base64::Engine;
        let b64 = base64::engine::general_purpose::STANDARD.encode(raw_tx);
        let r = self.jsonrpc(Chain::Solana, "sendTransaction", serde_json::json!([b64, {"encoding":"base64","preflightCommitment":"confirmed"}])).await?;
        r.as_str().map(|s| s.to_string()).ok_or(WalletError::BroadcastFailed("No signature".into()))
    }

    pub async fn sol_get_tx_status(&self, signature: &str) -> Result<Option<String>, WalletError> {
        let r = self.jsonrpc(Chain::Solana, "getSignatureStatuses", serde_json::json!([[signature]])).await?;
        let status = &r["value"][0];
        if status.is_null() { return Ok(None); }
        if status["err"].is_null() { Ok(Some("confirmed".into())) } else { Ok(Some("failed".into())) }
    }
}

impl Default for RpcClient {
    fn default() -> Self { Self::new() }
}

#[derive(Debug, Clone, Serialize)]
pub struct BtcFeeEstimates {
    pub fast: u64,
    pub standard: u64,
    pub slow: u64,
}
