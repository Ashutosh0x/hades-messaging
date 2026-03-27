use crate::relay_node::RelayNode;

/// Manages building and maintaining onion circuits.
pub struct Circuit {
    /// Ordered list of relay nodes (guard → middle → exit)
    pub hops: Vec<RelayNode>,
    /// Circuit identifier
    pub circuit_id: [u8; 32],
    /// Messages routed through this circuit
    pub message_count: u32,
    /// Maximum messages before rotation
    pub max_messages: u32,
}

impl Circuit {
    /// Create a new circuit with the given hops.
    pub fn new(circuit_id: [u8; 32], hops: Vec<RelayNode>, max_messages: u32) -> Self {
        Self {
            hops,
            circuit_id,
            message_count: 0,
            max_messages,
        }
    }

    /// Whether this circuit should be rotated.
    pub fn should_rotate(&self) -> bool {
        self.message_count >= self.max_messages
    }

    /// Record a message sent through this circuit.
    pub fn record_message(&mut self) {
        self.message_count += 1;
    }

    /// Number of hops in this circuit.
    pub fn hop_count(&self) -> usize {
        self.hops.len()
    }
}
