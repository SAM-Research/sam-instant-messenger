use super::{api_trait::SamApiClient, SamApiClientError};
use async_trait::async_trait;
use reqwest::{Client as ReqwestClient, Method, Request, Response, Url};
use sam_common::{
    api::{
        keys::PreKeyBundles, LinkDeviceRequest, LinkDeviceResponse, LinkDeviceToken,
        PublishPreKeys, RegistrationRequest, RegistrationResponse,
    },
    AccountId, DeviceId,
};

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

    async fn make_request(&self, request: Request) -> Result<Response, SamApiClientError> {
        let reponse = self
            .http_client
            .execute(request)
            .await
            .map_err(|_| SamApiClientError::CouldNotSendRequest)?;
        let status = reponse.status();

        if !status.is_success() {
            return Err(SamApiClientError::BadResponse(
                status.as_u16(),
                reponse.text().await.unwrap_or(
                    status
                        .canonical_reason()
                        .unwrap_or("Unknown reason")
                        .to_owned(),
                ),
            ));
        };

        Ok(reponse)
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
        let url_str = format!("{}/api/v1/account", self.base_url);
        let url = Url::parse(&url_str).map_err(|_| SamApiClientError::CouldNotParseUrl(url_str))?;
        let request = self
            .http_client
            .request(Method::POST, url)
            .json(&registration_request)
            .basic_auth(username, Some(password))
            .build()
            .map_err(|_| SamApiClientError::CouldNotBuildRequest)?;
        let response = self.make_request(request).await?;

        Ok(response
            .json()
            .await
            .map_err(|_| SamApiClientError::CouldNotParseResponse)?)
    }

    async fn delete_account(
        self,
        account_id: AccountId,
        device_id: DeviceId,
        password: &str,
    ) -> Result<(), SamApiClientError> {
        let url_str = format!("{}/api/v1/account", self.base_url);
        let url = Url::parse(&url_str).map_err(|_| SamApiClientError::CouldNotParseUrl(url_str))?;
        let request = self
            .http_client
            .request(Method::DELETE, url)
            .basic_auth(format!("{}.{}", account_id, device_id), Some(password))
            .build()
            .map_err(|_| SamApiClientError::CouldNotBuildRequest)?;
        let _ = self.make_request(request).await?;
        Ok(())
    }

    async fn get_pre_keys(
        &self,
        account_id: AccountId,
        device_id: DeviceId,
        password: &str,
        receiver_account_id: AccountId,
    ) -> Result<PreKeyBundles, SamApiClientError> {
        let url_str = format!("{}/api/v1/keys/{}", self.base_url, receiver_account_id);
        let url = Url::parse(&url_str).map_err(|_| SamApiClientError::CouldNotParseUrl(url_str))?;

        let request = self
            .http_client
            .request(Method::GET, url)
            .basic_auth(format!("{}.{}", account_id, device_id), Some(password))
            .build()
            .map_err(|_| SamApiClientError::CouldNotBuildRequest)?;

        let response = self.make_request(request).await?;

        let prekey_bundles = response
            .json::<PreKeyBundles>()
            .await
            .map_err(|_| SamApiClientError::CouldNotParseResponse)?;

        Ok(prekey_bundles)
    }

    async fn publish_pre_keys(
        &self,
        account_id: AccountId,
        device_id: DeviceId,
        password: &str,
        bundle: PublishPreKeys,
    ) -> Result<(), SamApiClientError> {
        let url_str = format!("{}/api/v1/keys", self.base_url);
        let url = Url::parse(&url_str).map_err(|_| SamApiClientError::CouldNotParseUrl(url_str))?;

        let request = self
            .http_client
            .request(Method::PUT, url)
            .json(&bundle)
            .basic_auth(format!("{}.{}", account_id, device_id), Some(password))
            .build()
            .map_err(|_| SamApiClientError::CouldNotBuildRequest)?;

        let _ = self.make_request(request).await?;

        Ok(())
    }

    async fn provision_device(
        &self,
        account_id: AccountId,
        device_id: DeviceId,
        password: &str,
    ) -> Result<LinkDeviceToken, SamApiClientError> {
        let url_str = format!("{}/api/v1/devices/provision", self.base_url);
        let url = Url::parse(&url_str).map_err(|_| SamApiClientError::CouldNotParseUrl(url_str))?;

        let request = self
            .http_client
            .request(Method::GET, url)
            .basic_auth(format!("{}.{}", account_id, device_id), Some(password))
            .build()
            .map_err(|_| SamApiClientError::CouldNotBuildRequest)?;

        let response = self.make_request(request).await?;

        let token = response
            .json::<LinkDeviceToken>()
            .await
            .map_err(|_| SamApiClientError::CouldNotParseResponse)?;

        Ok(token)
    }

    async fn link_device(
        &self,
        account_id: AccountId,
        device_id: DeviceId,
        password: &str,
        request: LinkDeviceRequest,
    ) -> Result<LinkDeviceResponse, SamApiClientError> {
        let url_str = format!("{}/api/v1/devices/link", self.base_url);
        let url = Url::parse(&url_str).map_err(|_| SamApiClientError::CouldNotParseUrl(url_str))?;

        let request = self
            .http_client
            .request(Method::POST, url)
            .json(&request)
            .basic_auth(format!("{}.{}", account_id, device_id), Some(password))
            .build()
            .map_err(|_| SamApiClientError::CouldNotBuildRequest)?;

        let response = self.make_request(request).await?;

        let link_device_response = response
            .json::<LinkDeviceResponse>()
            .await
            .map_err(|_| SamApiClientError::CouldNotParseResponse)?;

        Ok(link_device_response)
    }

    async fn delete_device(
        &self,
        account_id: AccountId,
        device_id: DeviceId,
        password: &str,
        removed_device: DeviceId,
    ) -> Result<(), SamApiClientError> {
        let url_str = format!("{}/api/v1/device/{}", self.base_url, removed_device);
        let url = Url::parse(&url_str).map_err(|_| SamApiClientError::CouldNotParseUrl(url_str))?;

        let request = self
            .http_client
            .request(Method::DELETE, url)
            .basic_auth(format!("{}.{}", account_id, device_id), Some(password))
            .build()
            .map_err(|_| SamApiClientError::CouldNotBuildRequest)?;

        let _ = self.make_request(request).await?;

        Ok(())
    }
}
