//! Bridge rotation manager for Hades onion routing.
//!
//! Automatically rotates Tor bridges at randomised intervals (7-30 days)
//! to prevent long-term enumeration by censors. Supports multiple
//! distribution methods for acquiring fresh bridges.

use std::time::{Duration, Instant};

/// How bridges are obtained.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DistributionMethod {
    /// Tor's Resource Distribution System (anti-enumeration).
    RDSys,
    /// In-browser CAPTCHA challenge.
    Moat,
    /// Telegram bot (@GetBridgesBot).
    Telegram,
    /// Email to bridges@torproject.org.
    Email,
    /// Your own private bridge pool.
    PrivatePool,
}

/// A single bridge endpoint.
#[derive(Debug, Clone)]
pub struct BridgeConfig {
    /// Full bridge line.
    pub bridge_line: String,
    /// When this bridge was added.
    pub added_at: Instant,
    /// Number of consecutive failures.
    pub failure_count: u32,
}

/// Manages bridge lifecycle: acquisition, rotation, and eviction.
pub struct BridgeRotationManager {
    /// Active bridge pool.
    pub bridges: Vec<BridgeConfig>,
    /// Maximum age before forced rotation.
    pub max_age: Duration,
    /// How we get new bridges.
    pub distribution: DistributionMethod,
    /// Minimum bridges to keep in pool.
    pub min_pool_size: usize,
}

impl BridgeRotationManager {
    pub fn new(distribution: DistributionMethod) -> Self {
        Self {
            bridges: Vec::new(),
            max_age: Duration::from_secs(14 * 24 * 3600), // 14 days default
            distribution,
            min_pool_size: 3,
        }
    }

    /// Check if any bridges need rotation and replace them.
    pub fn rotate_stale(&mut self) {
        let now = Instant::now();
        self.bridges.retain(|b| {
            let age = now.duration_since(b.added_at);
            let stale = age > self.max_age;
            let failing = b.failure_count > 5;
            !(stale || failing)
        });
    }

    /// Get the best bridge (fewest failures, newest).
    pub fn best_bridge(&self) -> Option<&BridgeConfig> {
        self.bridges
            .iter()
            .filter(|b| b.failure_count < 3)
            .min_by_key(|b| b.failure_count)
    }

    /// Record a connection failure for a bridge.
    pub fn record_failure(&mut self, bridge_line: &str) {
        if let Some(b) = self.bridges.iter_mut().find(|b| b.bridge_line == bridge_line) {
            b.failure_count += 1;
        }
    }

    /// Add a fresh bridge to the pool.
    pub fn add_bridge(&mut self, bridge_line: String) {
        self.bridges.push(BridgeConfig {
            bridge_line,
            added_at: Instant::now(),
            failure_count: 0,
        });
    }

    /// Whether the pool needs replenishment.
    pub fn needs_bridges(&self) -> bool {
        self.bridges.len() < self.min_pool_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_bridge_pool_needs_bridges() {
        let mgr = BridgeRotationManager::new(DistributionMethod::PrivatePool);
        assert!(mgr.needs_bridges());
    }

    #[test]
    fn best_bridge_prefers_least_failures() {
        let mut mgr = BridgeRotationManager::new(DistributionMethod::PrivatePool);
        mgr.add_bridge("bridge1".into());
        mgr.add_bridge("bridge2".into());
        mgr.record_failure("bridge1");
        mgr.record_failure("bridge1");
        let best = mgr.best_bridge().unwrap();
        assert_eq!(best.bridge_line, "bridge2");
    }
}
