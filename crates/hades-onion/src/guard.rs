use std::collections::HashSet;

/// Guard node selection and management.
///
/// Guard nodes are the first hop in every circuit. They are selected
/// from a stable set and rotated infrequently to balance between
/// anonymity and resistance to guard-level attacks.
pub struct GuardSet {
    /// Currently selected guard node IDs
    guards: HashSet<String>,
    /// Maximum number of guard nodes to maintain
    max_guards: usize,
}

impl GuardSet {
    pub fn new(max_guards: usize) -> Self {
        Self {
            guards: HashSet::new(),
            max_guards,
        }
    }

    /// Add a guard node to the set.
    pub fn add_guard(&mut self, node_id: String) -> bool {
        if self.guards.len() >= self.max_guards {
            return false;
        }
        self.guards.insert(node_id)
    }

    /// Remove a guard node.
    pub fn remove_guard(&mut self, node_id: &str) -> bool {
        self.guards.remove(node_id)
    }

    /// Check if a node is in the guard set.
    pub fn is_guard(&self, node_id: &str) -> bool {
        self.guards.contains(node_id)
    }

    /// Number of active guards.
    pub fn count(&self) -> usize {
        self.guards.len()
    }
}
