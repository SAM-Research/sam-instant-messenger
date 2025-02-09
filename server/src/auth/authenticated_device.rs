use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use libsignal_protocol::{ProtocolAddress, ServiceId};

use crate::{
    auth::authenticate::{auth_header, SaltedTokenHash},
    error::HTTPError,
    state::{account::Account, device::Device, ServerState},
};

use super::authenticate::service_id_aci;

pub struct AuthenticatedDevice {
    pub account: Account,
    pub device: Device,
}

impl AuthenticatedDevice {
    pub fn new(account: Account, device: Device) -> Self {
        Self { account, device }
    }

    pub fn protocol_address(&self) -> ProtocolAddress {
        ProtocolAddress::new(self.account.aci.service_id_string(), self.device.device_id)
    }
}

#[async_trait::async_trait]
impl FromRequestParts<ServerState> for AuthenticatedDevice {
    type Rejection = HTTPError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &ServerState,
    ) -> Result<Self, Self::Rejection> {
        let (username, device_id, password) = {
            let basic = auth_header(&parts.headers)?;
            (
                basic.username().clone(),
                basic.device_id().into(),
                basic.password().clone(),
            )
        };
        let service_id = ServiceId::parse_from_service_id_string(&username).ok_or(HTTPError {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            body: "Error parsing service id".into(),
        })?;

        let accounts = state.accounts.lock().await;

        let aci = service_id_aci(service_id)?;
        let account = accounts.get_account(&aci).await.map_err(|e| HTTPError {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            body: e.to_string(),
        })?;

        let addr = ProtocolAddress::new(service_id.service_id_string(), device_id);
        let device = accounts.get_device(addr).await.map_err(|e| HTTPError {
            status_code: StatusCode::NOT_FOUND,
            body: e.to_string(),
        })?;

        let salted_token =
            SaltedTokenHash::new(device.auth_token.to_owned(), device.salt.to_owned());

        if salted_token.verify(&password)? {
            Ok(AuthenticatedDevice::new(account, device))
        } else {
            Err(HTTPError {
                status_code: StatusCode::UNAUTHORIZED,
                body: "".into(),
            })
        }
    }
}
