use async_trait::async_trait;
use reqwest::{Client as ReqwestClient, Method, Url};
use sam_common::{
    api::{
        keys::PreKeyBundles, LinkDeviceRequest, LinkDeviceResponse, LinkDeviceToken,
        PublishPreKeys, RegistrationRequest, RegistrationResponse,
    },
    AccountId, DeviceId,
};

use super::{api_trait::SamApiClient, SamApiClientError};

#[derive(Debug)]
pub struct HttpClient {
    http_client: ReqwestClient,
    base_url: String,
}

impl HttpClient {
    pub fn new(base_url: String) -> Self {
        Self {
            http_client: ReqwestClient::new(),
            base_url,
        }
    }
}

#[async_trait(?Send)]
impl SamApiClient for HttpClient {
    async fn register_account(
        &self,
        username: &str,
        password: &str,
        registration_request: RegistrationRequest,
    ) -> Result<RegistrationResponse, SamApiClientError> {
        let url_str = format!("http://{}/api/v1/account", self.base_url);
        let url = Url::parse(&url_str).map_err(|_| SamApiClientError::CouldNotParseUrl(url_str))?;
        // println!("{}", to_string(&registration_request).unwrap());
        let request = self
            .http_client
            .request(Method::POST, url)
            .json(&registration_request)
            .basic_auth(username, Some(password))
            .build()
            .map_err(|_| SamApiClientError::CouldNotBuildRequest)?;
        let response = self
            .http_client
            .execute(request)
            .await
            .map_err(|_| SamApiClientError::CouldNotSendRequest)?;
        let status = response.status();
        if !status.is_success() {
            return Err(SamApiClientError::BadResponse(
                status.as_u16(),
                response.text().await.unwrap_or(
                    status
                        .canonical_reason()
                        .unwrap_or("Unknown reason")
                        .to_owned(),
                ),
            ));
        }

        Ok(response
            .json()
            .await
            .map_err(|_| SamApiClientError::CouldNotParseResponse)?)
    }

    async fn delete_account(
        self,
        _username: &str,
        _password: &str,
    ) -> Result<(), SamApiClientError> {
        todo!()
    }
    async fn get_pre_keys(
        &self,
        _username: &str,
        _password: &str,
        _account_id: AccountId,
    ) -> Result<PreKeyBundles, SamApiClientError> {
        todo!()
    }

    async fn publish_pre_keys(
        &self,
        username: &str,
        password: &str,
        bundle: PublishPreKeys,
    ) -> Result<(), SamApiClientError> {
        let url_str = format!("http://{}/api/v1/account", self.base_url);
        let url = Url::parse(&url_str).map_err(|_| SamApiClientError::CouldNotParseUrl(url_str))?;

        let request = self
            .http_client
            .request(Method::POST, url)
            .json(&bundle)
            .basic_auth(username, Some(password))
            .build()
            .map_err(|_| SamApiClientError::CouldNotBuildRequest)?;

        let response = self
            .http_client
            .execute(request)
            .await
            .map_err(|_| SamApiClientError::CouldNotSendRequest)?;

        let status = response.status();
        if !status.is_success() {
            return Err(SamApiClientError::BadResponse(
                status.as_u16(),
                response.text().await.unwrap_or(
                    status
                        .canonical_reason()
                        .unwrap_or("Unknown reason")
                        .to_owned(),
                ),
            ));
        }

        Ok(response
            .json()
            .await
            .map_err(|_| SamApiClientError::CouldNotParseResponse)?)
    }

    async fn provision_device(
        &self,
        _username: &str,
        _password: &str,
    ) -> Result<LinkDeviceToken, SamApiClientError> {
        todo!()
    }

    async fn link_device(
        &self,
        _username: &str,
        _password: &str,
        _request: LinkDeviceRequest,
    ) -> Result<LinkDeviceResponse, SamApiClientError> {
        todo!()
    }

    async fn delete_device(
        &self,
        _username: &str,
        _password: &str,
        _device_id: DeviceId,
    ) -> Result<(), SamApiClientError> {
        todo!()
    }
}
