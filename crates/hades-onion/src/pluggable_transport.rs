//! Pluggable transport abstraction for Hades onion routing.
//!
//! Wraps multiple anti-censorship transports so Hades traffic is
//! indistinguishable from allowed protocols. Selection is automatic
//! based on the user's censorship environment.
//!
//! ## Supported Transports (2026)
//!
//! | Transport   | Disguise                          | DPI Resistance |
//! |-------------|-----------------------------------|----------------|
//! | Obfs4       | Randomized byte stream            | High           |
//! | WebTunnel   | HTTPS WebSocket over real website | Very High      |
//! | Snowflake   | WebRTC to ephemeral peer proxies  | High           |
//! | Meek        | Domain fronting via CDN            | Moderate       |
//! | Obfs5       | Post-quantum obfuscated handshake | Experimental   |

use std::time::Duration;

/// Available pluggable transports.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TransportKind {
    /// Obfs4 — randomized byte stream, proven in the field.
    Obfs4,
    /// WebTunnel — mimics HTTPS WebSocket, coexists with real website.
    WebTunnel,
    /// Snowflake 2.0 — WebRTC to ephemeral volunteer proxies.
    Snowflake,
    /// Meek — domain fronting through CDN (Cloudflare, Azure).
    Meek,
    /// Obfs5 — experimental post-quantum obfuscated handshake.
    Obfs5,
    /// Direct — no transport wrapping (Tor-friendly networks).
    Direct,
}

/// Configuration for a single transport instance.
#[derive(Debug, Clone)]
pub struct TransportConfig {
    pub kind: TransportKind,
    /// Bridge line (e.g. `obfs4 1.2.3.4:443 FINGERPRINT ...`).
    pub bridge_line: String,
    /// Optional CDN domain for fronting (WebTunnel / Meek).
    pub front_domain: Option<String>,
    /// Handshake timeout.
    pub timeout: Duration,
}

/// Transport selection result from the probing engine.
#[derive(Debug)]
pub struct TransportProbeResult {
    pub kind: TransportKind,
    pub latency_ms: u64,
    pub reachable: bool,
}

/// Probe all available transports and rank by latency.
///
/// In production this sends a lightweight handshake to each bridge
/// and returns the fastest reachable transport.
pub async fn probe_transports(configs: &[TransportConfig]) -> Vec<TransportProbeResult> {
    let mut results = Vec::with_capacity(configs.len());

    for cfg in configs {
        // Placeholder — real implementation uses tokio::net::TcpStream
        results.push(TransportProbeResult {
            kind: cfg.kind,
            latency_ms: 0,
            reachable: false,
        });
    }

    // Sort by latency ascending (reachable first)
    results.sort_by(|a, b| {
        b.reachable
            .cmp(&a.reachable)
            .then(a.latency_ms.cmp(&b.latency_ms))
    });

    results
}

/// Select the best transport for the user's environment.
///
/// Priority order:
/// 1. WebTunnel (hardest to distinguish from normal HTTPS)
/// 2. Obfs4 (proven, fast)
/// 3. Snowflake (no bridge needed, but slower)
/// 4. Meek (high latency, last resort)
pub fn select_best(results: &[TransportProbeResult]) -> Option<TransportKind> {
    let priority = [
        TransportKind::WebTunnel,
        TransportKind::Obfs4,
        TransportKind::Snowflake,
        TransportKind::Meek,
        TransportKind::Direct,
    ];

    for preferred in &priority {
        if results.iter().any(|r| r.kind == *preferred && r.reachable) {
            return Some(*preferred);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn select_best_prefers_webtunnel() {
        let results = vec![
            TransportProbeResult { kind: TransportKind::Obfs4, latency_ms: 50, reachable: true },
            TransportProbeResult { kind: TransportKind::WebTunnel, latency_ms: 80, reachable: true },
        ];
        assert_eq!(select_best(&results), Some(TransportKind::WebTunnel));
    }

    #[test]
    fn select_best_falls_back() {
        let results = vec![
            TransportProbeResult { kind: TransportKind::Meek, latency_ms: 500, reachable: true },
            TransportProbeResult { kind: TransportKind::WebTunnel, latency_ms: 0, reachable: false },
        ];
        assert_eq!(select_best(&results), Some(TransportKind::Meek));
    }

    #[test]
    fn select_best_returns_none_when_all_blocked() {
        let results = vec![
            TransportProbeResult { kind: TransportKind::Obfs4, latency_ms: 0, reachable: false },
        ];
        assert_eq!(select_best(&results), None);
    }
}
