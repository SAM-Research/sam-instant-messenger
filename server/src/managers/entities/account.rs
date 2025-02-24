use libsignal_protocol::IdentityKey;
use sam_common::address::AccountId;
use utoipa::ToSchema;

#[derive(Clone, bon::Builder)]
pub struct Account {
    id: AccountId,
    username: String,
    identity: IdentityKey,
}

impl Account {
    pub fn id(&self) -> AccountId {
        self.id
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn identity(&self) -> &IdentityKey {
        &self.identity
    }
}
