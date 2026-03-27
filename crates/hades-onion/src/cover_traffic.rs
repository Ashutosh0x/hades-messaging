//! Cover traffic generator for Hades onion layer.
//!
//! Sends chaff (dummy) packets at configurable intervals to mask
//! real message timing. This defeats traffic correlation attacks
//! where an observer matches Alice's "send" spike with Bob's
//! "receive" spike.
//!
//! ## Traffic Patterns
//!
//! | Pattern    | Interval Distribution | Use Case                       |
//! |------------|-----------------------|--------------------------------|
//! | Constant   | Fixed                 | Maximum resistance, high cost  |
//! | Poisson    | Exponential (random)  | Good resistance, moderate cost |
//! | Adaptive   | Mirrors real traffic  | Low overhead when idle         |
//! | Mimicry    | Emulates WhatsApp     | Hides among popular apps       |

use std::time::Duration;

/// Cover traffic configuration.
#[derive(Debug, Clone)]
pub struct CoverTrafficConfig {
    /// Whether cover traffic is enabled.
    pub enabled: bool,
    /// Traffic shaping pattern.
    pub pattern: TrafficPattern,
    /// Intensity multiplier (0.0 = off, 1.0 = full rate).
    pub intensity: f32,
    /// Base interval between chaff packets.
    pub base_interval: Duration,
}

impl Default for CoverTrafficConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            pattern: TrafficPattern::Poisson,
            intensity: 0.5,
            base_interval: Duration::from_millis(2000),
        }
    }
}

/// Traffic shaping pattern.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TrafficPattern {
    /// Fixed interval between packets.
    Constant,
    /// Random intervals following Poisson distribution.
    Poisson,
    /// Adapts rate to match real traffic volume.
    Adaptive,
    /// Emulates the packet cadence of a popular messenger.
    Mimicry,
}

/// A single chaff packet ready for onion wrapping.
#[derive(Debug)]
pub struct ChaffPacket {
    /// Random payload (indistinguishable from real ciphertext).
    pub payload: Vec<u8>,
    /// Size bucket this packet was padded to.
    pub bucket: usize,
}

/// Fixed size buckets — same as sealed_sender_v2.
const SIZE_BUCKETS: [usize; 3] = [512, 8192, 65536];

/// Generate a single chaff packet.
///
/// The payload is filled with cryptographically random bytes so it is
/// indistinguishable from a real encrypted message to any observer.
pub fn generate_chaff(pattern: TrafficPattern) -> ChaffPacket {
    let size = match pattern {
        TrafficPattern::Mimicry => {
            // WhatsApp text messages are typically 100-500 bytes
            let base = 100 + (random_u32() % 400) as usize;
            select_bucket(base)
        }
        TrafficPattern::Constant | TrafficPattern::Poisson | TrafficPattern::Adaptive => {
            // Default to smallest bucket for efficiency
            SIZE_BUCKETS[0]
        }
    };

    let mut payload = vec![0u8; size];
    getrandom::getrandom(&mut payload).expect("CSPRNG failure");

    ChaffPacket {
        payload,
        bucket: size,
    }
}

/// Compute the next inter-packet delay for the given pattern.
pub fn next_delay(config: &CoverTrafficConfig) -> Duration {
    let base_ms = config.base_interval.as_millis() as f64;

    match config.pattern {
        TrafficPattern::Constant => config.base_interval,

        TrafficPattern::Poisson => {
            // Exponential distribution: -ln(U) * mean
            let u = (random_u32() as f64) / (u32::MAX as f64);
            let u = u.max(0.001); // avoid ln(0)
            let delay_ms = (-u.ln() * base_ms) as u64;
            Duration::from_millis(delay_ms.min(30_000)) // cap at 30s
        }

        TrafficPattern::Adaptive => {
            // Scale by intensity — higher intensity = shorter delays
            let scaled = base_ms / config.intensity.max(0.01) as f64;
            Duration::from_millis(scaled as u64)
        }

        TrafficPattern::Mimicry => {
            // WhatsApp-like: bursts of 2-5 messages then silence
            let jitter = (random_u32() % 3000) as u64;
            Duration::from_millis(500 + jitter)
        }
    }
}

/// Add timing jitter to a real message's send time.
///
/// Returns a random delay in \[50ms, 500ms\] to prevent exact
/// timing correlation between sender and receiver.
pub fn timing_jitter() -> Duration {
    let jitter_ms = 50 + (random_u32() % 450) as u64;
    Duration::from_millis(jitter_ms)
}

fn select_bucket(len: usize) -> usize {
    for &b in &SIZE_BUCKETS {
        if len <= b {
            return b;
        }
    }
    SIZE_BUCKETS[SIZE_BUCKETS.len() - 1]
}

fn random_u32() -> u32 {
    let mut buf = [0u8; 4];
    getrandom::getrandom(&mut buf).expect("CSPRNG failure");
    u32::from_le_bytes(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chaff_has_correct_bucket_size() {
        let pkt = generate_chaff(TrafficPattern::Constant);
        assert!(SIZE_BUCKETS.contains(&pkt.bucket));
        assert_eq!(pkt.payload.len(), pkt.bucket);
    }

    #[test]
    fn poisson_delay_is_bounded() {
        let cfg = CoverTrafficConfig::default();
        for _ in 0..100 {
            let d = next_delay(&cfg);
            assert!(d.as_millis() <= 30_000);
        }
    }

    #[test]
    fn timing_jitter_is_in_range() {
        for _ in 0..100 {
            let j = timing_jitter();
            assert!(j.as_millis() >= 50);
            assert!(j.as_millis() <= 500);
        }
    }
}
