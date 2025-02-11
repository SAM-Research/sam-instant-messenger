use axum::extract::FromRequestParts;
use axum::http::request::Parts;

use crate::state::entities::account::Account;
use crate::state::entities::device::Device;
use crate::state::traits::state_type::StateType;
use crate::state::ServerState;
use crate::ServerError;

pub struct AuthenticatedUser {
    account: Account,
    device: Device,
}

impl AuthenticatedUser {
    pub fn account(&self) -> &Account {
        &self.account
    }

    pub fn device(&self) -> &Device {
        &self.device
    }
}

#[async_trait::async_trait]
impl<T: StateType> FromRequestParts<ServerState<T>> for AuthenticatedUser {
    type Rejection = ServerError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &ServerState<T>,
    ) -> Result<Self, Self::Rejection> {
        todo!()
    }
}
