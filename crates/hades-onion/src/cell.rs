/// Fixed-size transport cell for onion routing.
///
/// All cells are the same size to prevent traffic analysis.
/// Cells that are shorter than CELL_SIZE are padded.
pub const CELL_SIZE: usize = 512;

#[derive(Clone)]
pub struct Cell {
    /// Circuit identifier for this cell
    pub circuit_id: [u8; 32],
    /// Cell command
    pub command: CellCommand,
    /// Payload (encrypted, fixed size)
    pub payload: [u8; CELL_SIZE],
}

#[derive(Clone, Debug, PartialEq)]
pub enum CellCommand {
    /// Create a new circuit hop
    Create,
    /// Circuit hop created successfully
    Created,
    /// Relay data through the circuit
    Relay,
    /// Destroy the circuit
    Destroy,
    /// Padding cell (cover traffic)
    Padding,
}

impl Cell {
    /// Create a new cell with the given payload, padded to CELL_SIZE.
    pub fn new(circuit_id: [u8; 32], command: CellCommand, data: &[u8]) -> Self {
        let mut payload = [0u8; CELL_SIZE];
        let copy_len = data.len().min(CELL_SIZE);
        payload[..copy_len].copy_from_slice(&data[..copy_len]);

        Self {
            circuit_id,
            command,
            payload,
        }
    }

    /// Create a padding cell for cover traffic.
    pub fn padding(circuit_id: [u8; 32]) -> Self {
        let mut payload = [0u8; CELL_SIZE];
        // Fill with random bytes for indistinguishability
        let mut rng_bytes = [0u8; CELL_SIZE];
        getrandom::getrandom(&mut rng_bytes).ok();
        payload.copy_from_slice(&rng_bytes);

        Self {
            circuit_id,
            command: CellCommand::Padding,
            payload,
        }
    }
}
