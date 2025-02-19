use async_trait::async_trait;
use derive_more::{Display, Error};
use sam_common::api::{keys::PublishPreKeys, RegistrationRequest, RegistrationResponse};

#[async_trait(?Send)]
pub trait SignalApiClient {
    type Error: std::error::Error;

    /// Send a [RegistrationRequest] to the server.
    /// Verifying the session is not implemented.
    async fn register_client(
        &self,
        password: String,
        registration_request: RegistrationRequest,
    ) -> Result<RegistrationResponse, Self::Error>;

    /// Uploads a Pre Key Bundle to the Server.
    async fn publish_pre_key_bundle(&self, bundle: PublishPreKeys) -> Result<(), Self::Error>;
}

#[derive(Debug)]
pub struct HttpClient;

#[derive(Debug, Display, Error)]
pub enum HttpClientError {}

#[async_trait(?Send)]
impl SignalApiClient for HttpClient {
    type Error = HttpClientError;

    async fn register_client(
        &self,
        _password: String,
        _registration_request: RegistrationRequest,
    ) -> Result<RegistrationResponse, Self::Error> {
        todo!()
    }

    async fn publish_pre_key_bundle(&self, _bundle: PublishPreKeys) -> Result<(), Self::Error> {
        todo!()
    }
}
