use x25519_dalek::{PublicKey as X25519Public, StaticSecret};
use zeroize::ZeroizeOnDrop;

/// Represents a single hop in an onion circuit.
#[derive(ZeroizeOnDrop)]
pub struct RelayNode {
    /// Shared secret established with this node
    shared_key: [u8; 32],
    /// Node's public key
    #[zeroize(skip)]
    pub public_key: [u8; 32],
    /// Human-readable node identifier (for logging)
    #[zeroize(skip)]
    pub node_id: String,
}

impl RelayNode {
    /// Create a relay node from a DH handshake.
    pub fn from_handshake(
        our_secret: &StaticSecret,
        their_public: &X25519Public,
        node_id: String,
    ) -> Self {
        let shared = our_secret.diffie_hellman(their_public);
        let key = blake3::hash(shared.as_bytes());

        Self {
            shared_key: *key.as_bytes(),
            public_key: *their_public.as_bytes(),
            node_id,
        }
    }

    /// Get the shared symmetric key for this hop.
    pub fn key(&self) -> &[u8; 32] {
        &self.shared_key
    }
}
