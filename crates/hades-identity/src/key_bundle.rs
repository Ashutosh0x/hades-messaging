use ed25519_dalek::{Signature, VerifyingKey, Verifier};
use serde::{Deserialize, Serialize};
use crate::error::IdentityError;

/// Core bundle structure, typically signed by the identity key.
#[derive(Clone, Serialize, Deserialize)]
pub struct DeviceKeyBundle {
    pub identity_key: [u8; 32],
    pub signed_prekey: [u8; 32],
    pub signature: Vec<u8>,
    pub one_time_prekeys: Vec<[u8; 32]>,
    pub pq_encapsulation_key: Option<Vec<u8>>,
    pub pq_signature: Option<Vec<u8>>,
}

impl DeviceKeyBundle {
    pub fn verify(&self) -> Result<(), IdentityError> {
        let pub_key = VerifyingKey::from_bytes(&self.identity_key)
            .map_err(|_| IdentityError::InvalidBundle)?;
        
        let sig_bytes: [u8; 64] = self.signature.as_slice().try_into()
            .map_err(|_| IdentityError::InvalidBundle)?;
        let sig = Signature::from_bytes(&sig_bytes);
        pub_key.verify(&self.signed_prekey, &sig)
            .map_err(|_| IdentityError::InvalidSignature)?;
        
        if let (Some(pq_key), Some(pq_sig_bytes)) = (&self.pq_encapsulation_key, &self.pq_signature) {
            let pq_sig_arr: [u8; 64] = pq_sig_bytes.as_slice().try_into()
                .map_err(|_| IdentityError::InvalidBundle)?;
            let pq_signature = Signature::from_bytes(&pq_sig_arr);
            pub_key.verify(pq_key, &pq_signature)
                .map_err(|_| IdentityError::InvalidSignature)?;
        }
        
        Ok(())
    }
}
