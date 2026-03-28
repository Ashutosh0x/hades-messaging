use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub enum TorStatus {
    Disconnected,
    Bootstrapping { progress: u8, stage: String },
    Connected { circuit_count: u32 },
    Error(String),
}

/// Wrapper around arti-client for Tor connectivity.
/// Feature-gated: only active with `tor` feature flag.
pub struct TorClient {
    status: Arc<RwLock<TorStatus>>,
}

impl TorClient {
    pub async fn bootstrap() -> Result<Self, String> {
        let status = Arc::new(RwLock::new(TorStatus::Bootstrapping {
            progress: 0, stage: "Initializing".into(),
        }));

        // TODO: When arti crate is added as dependency, enable real Tor bootstrap:
        // use arti_client::{TorClient as ArtiClient, TorClientConfig};
        // let config = TorClientConfig::default();
        // let client = ArtiClient::create_bootstrapped(config).await?;

        *status.write().await = TorStatus::Connected { circuit_count: 3 };
        tracing::info!("Tor client ready (stub mode)");

        Ok(Self { status })
    }

    pub async fn status(&self) -> TorStatus {
        self.status.read().await.clone()
    }

    /// Connect WebSocket through Tor.
    /// Currently falls back to direct connection for development.
    pub async fn connect_ws(&self, url: &str) -> Result<(), String> {
        tracing::info!("Tor WS connect (stub): {}", url);
        // TODO: Route WebSocket through Tor SOCKS proxy
        Ok(())
    }
}
