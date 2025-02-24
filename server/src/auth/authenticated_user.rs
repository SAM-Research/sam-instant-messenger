use std::str::FromStr;

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum_extra::headers::authorization::Basic;
use axum_extra::headers::Authorization;
use axum_extra::TypedHeader;
use sam_common::address::AccountId;
use utoipa::ToSchema;

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
            AccountId::from_str(account_id).map_err(|_| ServerError::AuthBasicParseError)?;
        let device_id = device_id
            .parse()
            .map_err(|_| ServerError::AuthBasicParseError)?;

        let account = { state.accounts.get_account(account_id).await? };
        let device = { state.devices.get_device(account_id, device_id).await? };

        device.password().verify(password)?;
        Ok(Self { account, device })
    }
}

#[cfg(test)]
mod test {
    use crate::{
        auth::{authenticated_user::AuthenticatedUser, password::Password},
        managers::{
            entities::{account::Account, device::Device},
            in_memory::test_utils::LINK_SECRET,
            traits::{account_manager::AccountManager, device_manager::DeviceManager},
        },
        state::ServerState,
    };
    use axum::{
        body::Body,
        extract::{FromRequest, FromRequestParts, Request},
        http::{header::AUTHORIZATION, request::Parts},
    };
    use base64::{engine::general_purpose::STANDARD, Engine as _};
    use libsignal_protocol::IdentityKeyPair;
    use rand::rngs::OsRng;
    use sam_common::address::AccountId;

    #[tokio::test]
    async fn test_from_request_parts() {
        let mut state = ServerState::in_memory_default(LINK_SECRET.to_string());

        let account_id = AccountId::generate();
        let account_pwd = "thebestetpassword3".to_string();
        let device_id = 1u32.into();

        let auth_header = format!(
            "Basic {}",
            STANDARD.encode(format!("{account_id}.{device_id}:{account_pwd}"))
        );

        let mut rng = OsRng;
        let pair = IdentityKeyPair::generate(&mut rng);

        let account = Account::builder()
            .id(account_id)
            .identity(*pair.identity_key())
            .username("abc3".to_string())
            .build();

        state
            .accounts
            .add_account(&account)
            .await
            .expect("Can add account");

        let device = Device::builder()
            .id(device_id)
            .name("a".to_string())
            .password(Password::generate(account_pwd).expect("abc3 can create password"))
            .creation(0)
            .registration_id(1.into())
            .build();

        let account_id = account.id();
        state
            .devices
            .add_device(account_id, device)
            .await
            .expect("Alice can add device");

        let request = Request::builder()
            .header(AUTHORIZATION, auth_header)
            .body(Body::empty())
            .unwrap();

        let mut parts = Parts::from_request(request, &state).await.unwrap();

        let result = AuthenticatedUser::from_request_parts(&mut parts, &state).await;

        assert!(
            result.is_ok_and(|au| au.account().id() == account_id && au.device().id() == device_id)
        )
    }
}
