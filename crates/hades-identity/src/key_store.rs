use std::collections::HashMap;
use crate::identity::PublicIdentity;
use crate::error::IdentityError;

pub trait KeyStore {
    fn save_identity(&mut self, id: &str, identity: PublicIdentity) -> Result<(), IdentityError>;
    fn get_identity(&self, id: &str) -> Result<Option<PublicIdentity>, IdentityError>;
}

pub struct InMemoryKeyStore {
    identities: HashMap<String, PublicIdentity>,
}

impl InMemoryKeyStore {
    pub fn new() -> Self {
        Self {
            identities: HashMap::new(),
        }
    }
    
    pub fn default() -> Self {
        Self::new()
    }
}

impl KeyStore for InMemoryKeyStore {
    fn save_identity(&mut self, id: &str, identity: PublicIdentity) -> Result<(), IdentityError> {
        self.identities.insert(id.to_string(), identity);
        Ok(())
    }

    fn get_identity(&self, id: &str) -> Result<Option<PublicIdentity>, IdentityError> {
        Ok(self.identities.get(id).cloned())
    }
}
