use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// In-memory prekey store for the relay server.
///
/// In production, this would be backed by SQLite or another persistent store.
/// The relay only stores opaque byte bundles — it cannot read key material.
pub struct PrekeyStore {
    /// identity_hash → list of one-time prekey bundles (opaque bytes)
    bundles: Arc<RwLock<HashMap<[u8; 32], PrekeyRecord>>>,
}

struct PrekeyRecord {
    /// Signed prekey bundle (always available)
    signed_prekey: Vec<u8>,
    /// One-time prekeys (consumed on fetch)
    one_time_prekeys: Vec<Vec<u8>>,
    /// PQ prekey (if uploaded)
    pq_prekey: Option<Vec<u8>>,
}

impl PrekeyStore {
    pub fn new() -> Self {
        Self {
            bundles: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Store a user's prekey bundle.
    pub async fn store_bundle(
        &self,
        identity: [u8; 32],
        signed_prekey: Vec<u8>,
        one_time_prekeys: Vec<Vec<u8>>,
        pq_prekey: Option<Vec<u8>>,
    ) {
        let mut store = self.bundles.write().await;
        let record = store.entry(identity).or_insert_with(|| PrekeyRecord {
            signed_prekey: Vec::new(),
            one_time_prekeys: Vec::new(),
            pq_prekey: None,
        });
        record.signed_prekey = signed_prekey;
        record.one_time_prekeys.extend(one_time_prekeys);
        record.pq_prekey = pq_prekey;
    }

    /// Fetch a prekey bundle for a target identity.
    /// Consumes one one-time prekey if available.
    pub async fn fetch_bundle(&self, identity: &[u8; 32]) -> Option<FetchedBundle> {
        let mut store = self.bundles.write().await;
        let record = store.get_mut(identity)?;

        let otpk = if !record.one_time_prekeys.is_empty() {
            Some(record.one_time_prekeys.remove(0))
        } else {
            None
        };

        Some(FetchedBundle {
            signed_prekey: record.signed_prekey.clone(),
            one_time_prekey: otpk,
            pq_prekey: record.pq_prekey.clone(),
            remaining_otpks: record.one_time_prekeys.len(),
        })
    }

    /// Get the count of remaining one-time prekeys for an identity.
    pub async fn remaining_prekeys(&self, identity: &[u8; 32]) -> usize {
        let store = self.bundles.read().await;
        store
            .get(identity)
            .map(|r| r.one_time_prekeys.len())
            .unwrap_or(0)
    }
}

/// Result of fetching a prekey bundle.
pub struct FetchedBundle {
    pub signed_prekey: Vec<u8>,
    pub one_time_prekey: Option<Vec<u8>>,
    pub pq_prekey: Option<Vec<u8>>,
    pub remaining_otpks: usize,
}
