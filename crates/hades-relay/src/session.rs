/// Per-connection session state machine.
///
/// Each WebSocket connection goes through:
/// 1. `Handshake` — client authenticates with challenge-response
/// 2. `Authenticated` — client can send/receive envelopes
/// 3. `Closed` — connection terminated

#[derive(Debug, Clone, PartialEq)]
pub enum SessionState {
    Handshake,
    Authenticated {
        identity_hash: [u8; 32],
        hades_id: String,
    },
    Closed,
}

pub struct Session {
    pub state: SessionState,
    pub circuit_id: Option<[u8; 32]>,
    pub hades_id: Option<String>,
    pub messages_sent: u64,
    pub messages_received: u64,
}

impl Session {
    pub fn new() -> Self {
        Self {
            state: SessionState::Handshake,
            circuit_id: None,
            hades_id: None,
            messages_sent: 0,
            messages_received: 0,
        }
    }

    pub fn authenticate(
        &mut self,
        identity_hash: [u8; 32],
        circuit_id: [u8; 32],
        hades_id: String,
    ) {
        self.state = SessionState::Authenticated {
            identity_hash,
            hades_id: hades_id.clone(),
        };
        self.circuit_id = Some(circuit_id);
        self.hades_id = Some(hades_id);
    }

    pub fn close(&mut self) {
        self.state = SessionState::Closed;
    }

    pub fn is_authenticated(&self) -> bool {
        matches!(self.state, SessionState::Authenticated { .. })
    }
}

