use super::{
    api_trait::{ApiClient, ApiClientConfig},
    ApiClientError,
};
use async_trait::async_trait;
use log::debug;
use reqwest::{Client as ReqwestClient, ClientBuilder, Method, Request, Response, Url};
use rustls::ClientConfig;
use sam_common::{
    api::{
        keys::PreKeyBundles, LinkDeviceRequest, LinkDeviceResponse, LinkDeviceToken,
        PublishPreKeys, RegistrationRequest, RegistrationResponse,
    },
    AccountId, DeviceId,
};

#[derive(Debug)]
pub struct HttpClientConfig {
    base_url: String,
    client_builder: ClientBuilder,
}

impl HttpClientConfig {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url: format!("http://{}", base_url),
            client_builder: ReqwestClient::builder(),
        }
    }

    pub fn new_with_tls(base_url: String, config: ClientConfig) -> Self {
        Self {
            base_url: format!("https://{}", base_url),
            client_builder: ReqwestClient::builder().use_preconfigured_tls(config),
        }
    }
}

#[async_trait(?Send)]
impl ApiClientConfig for HttpClientConfig {
    type ApiClient = HttpClient;

    async fn create(self) -> Result<Self::ApiClient, ApiClientError> {
        Ok(Self::ApiClient {
            http_client: self
                .client_builder
                .build()
                .inspect_err(|e| debug!("{e}"))
                .map_err(|_| ApiClientError::FailedToBuildApiClient)?,
            base_url: self.base_url,
        })
    }
}

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

    async fn make_request(&self, request: Request) -> Result<Response, ApiClientError> {
        let response = self
            .http_client
            .execute(request)
            .await
            .inspect_err(|e| debug!("{e}"))
            .map_err(|_| ApiClientError::CouldNotSendRequest)?;
        let status = response.status();

        if !status.is_success() {
            return Err(ApiClientError::ErrorResponse(
                status.as_u16(),
                response.text().await.unwrap_or(
                    status
                        .canonical_reason()
                        .unwrap_or("Unknown reason")
                        .to_owned(),
                ),
            ));
        };
        Ok(response)
    }
}

#[async_trait(?Send)]
impl ApiClient for HttpClient {
    async fn register_account(
        &self,
        username: &str,
        password: &str,
        registration_request: RegistrationRequest,
    ) -> Result<RegistrationResponse, ApiClientError> {
        let url_str = format!("{}/api/v1/account", self.base_url);
        let url = Url::parse(&url_str)
            .inspect_err(|e| debug!("{e}"))
            .map_err(|_| ApiClientError::CouldNotParseUrl(url_str))?;
        let request = self
            .http_client
            .request(Method::POST, url)
            .json(&registration_request)
            .basic_auth(username, Some(password))
            .build()
            .inspect_err(|e| debug!("{e}"))
            .map_err(|_| ApiClientError::CouldNotBuildRequest)?;

        let response = self.make_request(request).await?;

        Ok(response
            .json()
            .await
            .inspect_err(|e| debug!("{e}"))
            .map_err(|_| ApiClientError::CouldNotParseResponse)?)
    }

    async fn delete_account(
        &self,
        account_id: AccountId,
        device_id: DeviceId,
        password: &str,
    ) -> Result<(), ApiClientError> {
        let url_str = format!("{}/api/v1/account", self.base_url);
        let url = Url::parse(&url_str)
            .inspect_err(|e| debug!("{e}"))
            .map_err(|_| ApiClientError::CouldNotParseUrl(url_str))?;
        let request = self
            .http_client
            .request(Method::DELETE, url)
            .basic_auth(format!("{}.{}", account_id, device_id), Some(password))
            .build()
            .inspect_err(|e| debug!("{e}"))
            .map_err(|_| ApiClientError::CouldNotBuildRequest)?;

        let _ = self.make_request(request).await?;

        Ok(())
    }

    async fn get_pre_key_bundles(
        &self,
        account_id: AccountId,
        device_id: DeviceId,
        password: &str,
        receiver_account_id: AccountId,
        devices: Option<Vec<DeviceId>>,
    ) -> Result<PreKeyBundles, ApiClientError> {
        let url_str = format!("{}/api/v1/keys/{}", self.base_url, receiver_account_id);
        let url = Url::parse(&url_str)
            .inspect_err(|e| debug!("{e}"))
            .map_err(|_| ApiClientError::CouldNotParseUrl(url_str))?;

        let request_builder = match devices {
            None => self
                .http_client
                .request(Method::GET, url)
                .basic_auth(format!("{}.{}", account_id, device_id), Some(password)),
            Some(devices) => self
                .http_client
                .request(Method::GET, url)
                .json(&devices)
                .basic_auth(format!("{}.{}", account_id, device_id), Some(password)),
        };

        let request = request_builder
            .build()
            .inspect_err(|e| debug!("{e}"))
            .map_err(|_| ApiClientError::CouldNotBuildRequest)?;

        let response = self.make_request(request).await?;

        let prekey_bundles = response
            .json::<PreKeyBundles>()
            .await
            .inspect_err(|e| debug!("{e}"))
            .map_err(|_| ApiClientError::CouldNotParseResponse)?;

        Ok(prekey_bundles)
    }

    async fn publish_pre_keys(
        &self,
        account_id: AccountId,
        device_id: DeviceId,
        password: &str,
        bundle: PublishPreKeys,
    ) -> Result<(), ApiClientError> {
        let url_str = format!("{}/api/v1/keys", self.base_url);
        let url = Url::parse(&url_str)
            .inspect_err(|e| debug!("{e}"))
            .map_err(|_| ApiClientError::CouldNotParseUrl(url_str))?;

        let request = self
            .http_client
            .request(Method::PUT, url)
            .json(&bundle)
            .basic_auth(format!("{}.{}", account_id, device_id), Some(password))
            .build()
            .inspect_err(|e| debug!("{e}"))
            .map_err(|_| ApiClientError::CouldNotBuildRequest)?;

        let _ = self.make_request(request).await?;

        Ok(())
    }

    async fn provision_device(
        &self,
        account_id: AccountId,
        device_id: DeviceId,
        password: &str,
    ) -> Result<LinkDeviceToken, ApiClientError> {
        let url_str = format!("{}/api/v1/devices/provision", self.base_url);
        let url = Url::parse(&url_str)
            .inspect_err(|e| debug!("{e}"))
            .map_err(|_| ApiClientError::CouldNotParseUrl(url_str))?;

        let request = self
            .http_client
            .request(Method::GET, url)
            .basic_auth(format!("{}.{}", account_id, device_id), Some(password))
            .build()
            .inspect_err(|e| debug!("{e}"))
            .map_err(|_| ApiClientError::CouldNotBuildRequest)?;

        let response = self.make_request(request).await?;

        let token = response
            .json::<LinkDeviceToken>()
            .await
            .inspect_err(|e| debug!("{e}"))
            .map_err(|_| ApiClientError::CouldNotParseResponse)?;

        Ok(token)
    }

    async fn link_device(
        &self,
        password: &str,
        request: LinkDeviceRequest,
    ) -> Result<LinkDeviceResponse, ApiClientError> {
        let url_str = format!("{}/api/v1/devices/link", self.base_url);
        let url = Url::parse(&url_str)
            .inspect_err(|e| debug!("{e}"))
            .map_err(|_| ApiClientError::CouldNotParseUrl(url_str))?;

        let request = self
            .http_client
            .request(Method::POST, url)
            .json(&request)
            .basic_auth("".to_owned(), Some(password))
            .build()
            .inspect_err(|e| debug!("{e}"))
            .map_err(|_| ApiClientError::CouldNotBuildRequest)?;

        let response = self.make_request(request).await?;

        let link_device_response = response
            .json::<LinkDeviceResponse>()
            .await
            .inspect_err(|e| debug!("{e}"))
            .map_err(|_| ApiClientError::CouldNotParseResponse)?;

        Ok(link_device_response)
    }

    async fn delete_device(
        &self,
        account_id: AccountId,
        device_id: DeviceId,
        password: &str,
        removed_device: DeviceId,
    ) -> Result<(), ApiClientError> {
        let url_str = format!("{}/api/v1/device/{}", self.base_url, removed_device);
        let url = Url::parse(&url_str)
            .inspect_err(|e| debug!("{e}"))
            .map_err(|_| ApiClientError::CouldNotParseUrl(url_str))?;

        let request = self
            .http_client
            .request(Method::DELETE, url)
            .basic_auth(format!("{}.{}", account_id, device_id), Some(password))
            .build()
            .inspect_err(|e| debug!("{e}"))
            .map_err(|_| ApiClientError::CouldNotBuildRequest)?;

        let _ = self.make_request(request).await?;

        Ok(())
    }

    async fn get_user_account_id(
        &self,
        account_id: AccountId,
        device_id: DeviceId,
        password: &str,
        username: &str,
    ) -> Result<AccountId, ApiClientError> {
        let url_str = format!("{}/api/v1/account/{}", self.base_url, username);
        let url = Url::parse(&url_str)
            .inspect_err(|e| debug!("{e}"))
            .map_err(|_| ApiClientError::CouldNotParseUrl(url_str))?;

        let request = self
            .http_client
            .request(Method::GET, url)
            .basic_auth(format!("{}.{}", account_id, device_id), Some(password))
            .build()
            .inspect_err(|e| debug!("{e}"))
            .map_err(|_| ApiClientError::CouldNotBuildRequest)?;

        let response = self.make_request(request).await?;

        let account_id = response
            .json::<AccountId>()
            .await
            .inspect_err(|e| debug!("{e}"))
            .map_err(|_| ApiClientError::CouldNotParseResponse)?;

        Ok(account_id)
    }
}
