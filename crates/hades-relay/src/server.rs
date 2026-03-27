use std::sync::Arc;
use crate::config::RelayConfig;
use crate::prekey_store::PrekeyStore;
use crate::rate_limit::RateLimiter;
use crate::router::Router;

/// Shared server state.
pub struct ServerState {
    pub config: RelayConfig,
    pub router: Router,
    pub prekey_store: PrekeyStore,
    pub rate_limiter: RateLimiter,
}

impl ServerState {
    pub fn new(config: RelayConfig) -> Arc<Self> {
        let rate_limiter = RateLimiter::new(config.rate_limit_rps, config.rate_limit_burst);
        Arc::new(Self {
            config,
            router: Router::new(),
            prekey_store: PrekeyStore::new(),
            rate_limiter,
        })
    }
}
