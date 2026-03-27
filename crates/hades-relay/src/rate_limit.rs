use std::num::NonZeroU32;
use dashmap::DashMap;
use governor::{Quota, RateLimiter as GovRateLimiter};
use governor::clock::DefaultClock;
use governor::state::{InMemoryState, NotKeyed};

type Limiter = GovRateLimiter<NotKeyed, InMemoryState, DefaultClock>;

/// Per-identity rate limiter using token bucket algorithm.
pub struct RateLimiter {
    limiters: DashMap<[u8; 32], Limiter>,
    rps: NonZeroU32,
    burst: NonZeroU32,
}

impl RateLimiter {
    pub fn new(rps: u32, burst: u32) -> Self {
        Self {
            limiters: DashMap::new(),
            rps: NonZeroU32::new(rps).unwrap_or(NonZeroU32::new(10).unwrap()),
            burst: NonZeroU32::new(burst).unwrap_or(NonZeroU32::new(20).unwrap()),
        }
    }

    /// Check if a request from the given identity is allowed.
    /// Returns `Ok(())` if allowed, `Err(retry_after_secs)` if rate limited.
    pub fn check(&self, identity: &[u8; 32]) -> Result<(), u32> {
        let limiter = self.limiters.entry(*identity).or_insert_with(|| {
            GovRateLimiter::direct(
                Quota::per_second(self.rps).allow_burst(self.burst),
            )
        });

        limiter.check().map_err(|_| {
            // Return a conservative retry-after hint
            1u32
        })
    }
}

