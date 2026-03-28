use crate::error::WalletError;
use crate::hd::Chain;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceData {
    pub usd: f64,
    pub usd_24h_change: f64,
}

pub struct PriceService {
    client: reqwest::Client,
    cache: Arc<RwLock<HashMap<String, (PriceData, Instant)>>>,
    cache_ttl: Duration,
}

impl PriceService {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder().timeout(Duration::from_secs(10)).build().unwrap(),
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl: Duration::from_secs(60),
        }
    }

    fn coingecko_id(chain: Chain) -> &'static str {
        match chain {
            Chain::Bitcoin => "bitcoin",
            Chain::Ethereum | Chain::Arbitrum | Chain::Optimism | Chain::Base => "ethereum",
            Chain::Solana => "solana",
            Chain::Polygon => "matic-network",
            Chain::Avalanche => "avalanche-2",
            Chain::BnbSmartChain => "binancecoin",
            Chain::Litecoin => "litecoin",
            Chain::Dogecoin => "dogecoin",
            Chain::Tron => "tron",
        }
    }

    pub async fn get_price(&self, chain: Chain) -> Result<PriceData, WalletError> {
        let id = Self::coingecko_id(chain);
        {
            let cache = self.cache.read().await;
            if let Some((data, fetched_at)) = cache.get(id) {
                if fetched_at.elapsed() < self.cache_ttl { return Ok(data.clone()); }
            }
        }

        let url = format!(
            "https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies=usd&include_24hr_change=true", id
        );
        let resp: serde_json::Value = self.client.get(&url).send().await?.json().await?;
        let usd = resp[id]["usd"].as_f64().unwrap_or(0.0);
        let change = resp[id]["usd_24h_change"].as_f64().unwrap_or(0.0);
        let data = PriceData { usd, usd_24h_change: change };

        let mut cache = self.cache.write().await;
        cache.insert(id.to_string(), (data.clone(), Instant::now()));
        Ok(data)
    }

    pub async fn get_all_prices(&self) -> Result<HashMap<Chain, PriceData>, WalletError> {
        let ids: Vec<&str> = Chain::all().iter()
            .map(|c| Self::coingecko_id(*c))
            .collect::<std::collections::HashSet<_>>().into_iter().collect();

        let url = format!(
            "https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies=usd&include_24hr_change=true",
            ids.join(",")
        );
        let resp: serde_json::Value = self.client.get(&url).send().await?.json().await?;

        let mut prices = HashMap::new();
        for chain in Chain::all() {
            let id = Self::coingecko_id(chain);
            if let Some(data) = resp.get(id) {
                let price = PriceData {
                    usd: data["usd"].as_f64().unwrap_or(0.0),
                    usd_24h_change: data["usd_24h_change"].as_f64().unwrap_or(0.0),
                };
                prices.insert(chain, price.clone());
                let mut cache = self.cache.write().await;
                cache.insert(id.to_string(), (price, Instant::now()));
            }
        }
        Ok(prices)
    }

    pub fn to_usd(balance: &str, price_usd: f64) -> f64 {
        balance.parse::<f64>().unwrap_or(0.0) * price_usd
    }
}

impl Default for PriceService {
    fn default() -> Self { Self::new() }
}
