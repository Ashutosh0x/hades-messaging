use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

/// Circuit-based message router.
///
/// Maps circuit IDs to connected client channels. The relay routes
/// sealed envelopes without inspecting content.
pub struct Router {
    /// circuit_id → sender channel for the connected client
    routes: Arc<RwLock<HashMap<[u8; 32], mpsc::Sender<Vec<u8>>>>>,
}

impl Router {
    pub fn new() -> Self {
        Self {
            routes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a circuit for a connected client.
    pub async fn register(&self, circuit_id: [u8; 32], sender: mpsc::Sender<Vec<u8>>) {
        self.routes.write().await.insert(circuit_id, sender);
    }

    /// Unregister a circuit when a client disconnects.
    pub async fn unregister(&self, circuit_id: &[u8; 32]) {
        self.routes.write().await.remove(circuit_id);
    }

    /// Route a sealed envelope to its destination circuit.
    pub async fn route(&self, destination: &[u8; 32], envelope: Vec<u8>) -> Result<(), RouteError> {
        let routes = self.routes.read().await;
        let sender = routes.get(destination).ok_or(RouteError::NoRoute)?;
        sender.send(envelope).await.map_err(|_| RouteError::ChannelClosed)
    }

    /// Number of active circuits.
    pub async fn active_circuits(&self) -> usize {
        self.routes.read().await.len()
    }
}

#[derive(Debug)]
pub enum RouteError {
    NoRoute,
    ChannelClosed,
}
