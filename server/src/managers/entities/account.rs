use libsignal_protocol::IdentityKey;
use sam_common::address::AccountId;

#[derive(Clone, bon::Builder, Debug, PartialEq, Eq)]
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

#[cfg(test)]
impl Account {
    pub fn random() -> Self {
        use libsignal_protocol::IdentityKeyPair;
        use rand::rngs::OsRng;
        use sqlx::types::Uuid;

        let id = Uuid::new_v4();
        let id_key = IdentityKeyPair::generate(&mut OsRng);
        Self::builder()
            .username(id.to_string())
            .id(id.into())
            .identity(id_key.identity_key().to_owned())
            .build()
    }
}
