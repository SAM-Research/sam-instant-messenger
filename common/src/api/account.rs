use super::{device::DeviceActivationInfo, keys::id_key};
use crate::address::AccountId;
use libsignal_protocol::IdentityKey;
use serde::{Deserialize, Serialize};
use utoipa::openapi::{RefOr, Schema};
use utoipa::{PartialSchema, ToSchema};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistrationRequest {
    #[serde(with = "id_key")]
    pub identity_key: IdentityKey,
    pub device_activation: DeviceActivationInfo,
}

impl ToSchema for RegistrationRequest {
    fn name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("Pet")
    }
}
impl utoipa::PartialSchema for RegistrationRequest {
    fn schema() -> RefOr<Schema> {
        utoipa::openapi::ObjectBuilder::new()
            .property(
                "identity_key",
                utoipa::openapi::ObjectBuilder::new()
                    .schema_type(utoipa::openapi::schema::Type::String),
            )
            .required("identity_key")
            .property(
                "device_activation",
                utoipa::openapi::ObjectBuilder::new()
                    .schema_type(utoipa::openapi::schema::Type::String),
            )
            .required("device_activation")
            .example(Some(serde_json::json!({
              "identity_key":"bob the cat","device_activation":"beans"
            })))
            .into()
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RegistrationResponse {
    pub account_id: AccountId,
}

impl ToSchema for RegistrationResponse {
    fn name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("Pet")
    }
}
impl utoipa::PartialSchema for RegistrationResponse {
    fn schema() -> RefOr<Schema> {
        utoipa::openapi::ObjectBuilder::new()
            .property(
                "account_id",
                utoipa::openapi::ObjectBuilder::new()
                    .schema_type(utoipa::openapi::schema::Type::String),
            )
            .required("account_id")
            .example(Some(serde_json::json!({
              "account_id":"1234"
            })))
            .into()
    }
}
