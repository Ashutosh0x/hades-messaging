use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// TTL cache for RPC responses to avoid spamming public endpoints
pub struct RpcCache {
    entries: Arc<RwLock<HashMap<String, CacheEntry>>>,
}

struct CacheEntry {
    value: serde_json::Value,
    expires_at: Instant,
}

impl RpcCache {
    pub fn new() -> Self {
        Self { entries: Arc::new(RwLock::new(HashMap::new())) }
    }

    pub async fn get(&self, key: &str) -> Option<serde_json::Value> {
        let entries = self.entries.read().await;
        entries.get(key).and_then(|entry| {
            if entry.expires_at > Instant::now() { Some(entry.value.clone()) }
            else { None }
        })
    }

    pub async fn set(&self, key: String, value: serde_json::Value, ttl: Duration) {
        let mut entries = self.entries.write().await;
        entries.insert(key, CacheEntry { value, expires_at: Instant::now() + ttl });
        if entries.len() > 1000 {
            let now = Instant::now();
            entries.retain(|_, v| v.expires_at > now);
        }
    }

    /// Balance cache: 15s TTL
    pub async fn get_or_fetch_balance<F, Fut>(
        &self, chain: &str, address: &str, fetch: F,
    ) -> Result<serde_json::Value, crate::error::WalletError>
    where F: FnOnce() -> Fut,
          Fut: std::future::Future<Output = Result<serde_json::Value, crate::error::WalletError>>,
    {
        let key = format!("balance:{}:{}", chain, address);
        if let Some(cached) = self.get(&key).await { return Ok(cached); }
        let value = fetch().await?;
        self.set(key, value.clone(), Duration::from_secs(15)).await;
        Ok(value)
    }

    /// Gas price cache: 12s TTL
    pub async fn get_or_fetch_gas<F, Fut>(
        &self, chain: &str, fetch: F,
    ) -> Result<serde_json::Value, crate::error::WalletError>
    where F: FnOnce() -> Fut,
          Fut: std::future::Future<Output = Result<serde_json::Value, crate::error::WalletError>>,
    {
        let key = format!("gas:{}", chain);
        if let Some(cached) = self.get(&key).await { return Ok(cached); }
        let value = fetch().await?;
        self.set(key, value.clone(), Duration::from_secs(12)).await;
        Ok(value)
    }

    /// Token decimals: cache 1 year (immutable)
    pub async fn get_or_fetch_decimals<F, Fut>(
        &self, chain: &str, contract: &str, fetch: F,
    ) -> Result<u8, crate::error::WalletError>
    where F: FnOnce() -> Fut,
          Fut: std::future::Future<Output = Result<u8, crate::error::WalletError>>,
    {
        let key = format!("decimals:{}:{}", chain, contract);
        if let Some(cached) = self.get(&key).await {
            return Ok(cached.as_u64().unwrap_or(18) as u8);
        }
        let value = fetch().await?;
        self.set(key, serde_json::json!(value), Duration::from_secs(86400 * 365)).await;
        Ok(value)
    }
}

impl Default for RpcCache {
    fn default() -> Self { Self::new() }
}
