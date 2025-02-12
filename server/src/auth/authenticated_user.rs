use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum_extra::headers::authorization::Basic;
use axum_extra::headers::Authorization;
use axum_extra::TypedHeader;
use uuid::Uuid;

use crate::managers::entities::account::Account;
use crate::managers::entities::device::Device;
use crate::managers::traits::account_manager::AccountManager;
use crate::managers::traits::device_manager::DeviceManager;
use crate::state::state_type::StateType;
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
        let (userinfo, password) = {
            let TypedHeader(basic) =
                TypedHeader::<Authorization<Basic>>::from_request_parts(parts, &state)
                    .await
                    .map_err(|_| ServerError::AuthBasicParseError)?;
            (basic.username().to_string(), basic.password().to_string())
        };

        let (account_id, device_id) = userinfo
            .split_once(".")
            .ok_or(ServerError::AuthBasicParseError)?;
        let account_id =
            Uuid::parse_str(account_id).map_err(|_| ServerError::AuthBasicParseError)?;
        let device_id = device_id
            .parse()
            .map_err(|_| ServerError::AuthBasicParseError)?;

        let account = { state.accounts.lock().await.get_account(&account_id).await? };
        let device = {
            state
                .devices
                .lock()
                .await
                .get_device(&account_id, &device_id)
                .await?
        };

        device.password().verify(password)?;
        Ok(Self { account, device })
    }
}

#[cfg(test)]
mod test {}
