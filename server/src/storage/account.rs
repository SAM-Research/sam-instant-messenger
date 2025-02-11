use bon::Builder;
use sam_common::address::AccountId;

use super::IdentityKey;

#[derive(Debug, Builder, Clone, PartialEq, Eq)]
pub struct Account {
    aci: AccountId,
    identity_key: IdentityKey,
}

impl Account {
    pub fn aci(&self) -> &AccountId {
        &self.aci
    }
    pub fn identity_key(&self) -> &IdentityKey {
        &self.identity_key
    }
}
