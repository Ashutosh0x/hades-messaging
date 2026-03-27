use crate::identity::PublicIdentity;

/// Generates a human-readable safety number / fingerprint for out-of-band verification.
pub struct SafetyNumber;

impl SafetyNumber {
    pub fn generate(alice: &PublicIdentity, bob: &PublicIdentity) -> String {
        let mut alice_bytes = alice.key.as_bytes().to_vec();
        let mut bob_bytes = bob.key.as_bytes().to_vec();
        
        // Sort to ensure fingerprint is identical regardless of who generates it
        if alice_bytes > bob_bytes {
            std::mem::swap(&mut alice_bytes, &mut bob_bytes);
        }
        
        // Hash the concatenated keys
        let mut combined = alice_bytes;
        combined.extend(bob_bytes);
        
        let hash = blake3::hash(&combined);
        
        // Convert to a numeric string or hex format for users to compare
        let hex = hex::encode(hash.as_bytes());
        
        // In a real implementation this would format into groups of numbers
        format!("{}-{}", &hex[0..16], &hex[16..32])
    }
}
