use libsignal_protocol::IdentityKey;
use uuid::Uuid;

#[derive(Clone, bon::Builder)]
pub struct Account {
    id: Uuid,
    username: String,
    identity: IdentityKey,
}

impl Account {
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn identity(&self) -> &IdentityKey {
        &self.identity
    }
}
