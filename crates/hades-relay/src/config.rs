use serde::Deserialize;

/// Relay server configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct RelayConfig {
    /// Listen address (e.g. "0.0.0.0:8443")
    pub listen_addr: String,
    /// Maximum concurrent connections
    pub max_connections: usize,
    /// Message queue TTL in seconds
    pub message_ttl_secs: u64,
    /// Rate limit: requests per second per identity
    pub rate_limit_rps: u32,
    /// Rate limit: burst size
    pub rate_limit_burst: u32,
    /// Database path for prekey storage
    pub db_path: String,
    /// Enable Prometheus metrics endpoint
    pub metrics_enabled: bool,
}

impl Default for RelayConfig {
    fn default() -> Self {
        Self {
            listen_addr: "0.0.0.0:8443".to_string(),
            max_connections: 200_000,
            message_ttl_secs: 30 * 24 * 3600,
            rate_limit_rps: 10,
            rate_limit_burst: 20,
            db_path: "hades-relay.db".to_string(),
            metrics_enabled: true,
        }
    }
}
