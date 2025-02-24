use async_trait::async_trait;
use sam_common::{
    api::{
        keys::PreKeyBundles, LinkDeviceRequest, LinkDeviceResponse, LinkDeviceToken,
        PublishPreKeys, RegistrationRequest, RegistrationResponse,
    },
    AccountId, DeviceId,
};

use super::SamApiClientError;

#[async_trait(?Send)]
pub trait SamApiClient {
    /// Registers a new account with the given registration request.
    ///
    /// # Arguments
    ///
    /// * `username` - The username of the account.
    /// * `password` - The password of the account.
    /// * `request` - A [`RegistrationRequest`] containing the necessary details for account creation.
    ///
    /// # Returns
    ///
    /// * `Ok(RegistrationResponse)` on successful registration.
    /// * `Err(SamApiClientError)` if the registration fails.
    async fn register_account(
        &self,
        username: &str,
        password: &str,
        request: RegistrationRequest,
    ) -> Result<RegistrationResponse, SamApiClientError>;

    /// Deletes the current account.
    ///
    /// # Arguments
    ///
    /// * `username` - The username of the account.
    /// * `password` - The password of the account.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the account was successfully deleted.
    /// * `Err(SamApiClientError)` if the deletion fails.
    // TODO: What if deletion fails?
    async fn delete_account(self, username: &str, password: &str) -> Result<(), SamApiClientError>;

    /// Retrieves pre-key bundles for the given account.
    ///
    /// A pre-key bundle is retrieved for each device that the recipient has.
    ///
    /// # Arguments
    ///
    /// * `username` - The username of the account.
    /// * `password` - The password of the account.
    /// * `account_id` - The [`AccountId`] for which pre-keys should be fetched.
    ///
    /// # Returns
    ///
    /// * `Ok(PreKeyBundles)` containing the retrieved pre-key bundles.
    /// * `Err(SamApiClientError)` if fetching pre-key bundles fails.
    async fn get_pre_keys(
        &self,
        username: &str,
        password: &str,
        account_id: AccountId,
    ) -> Result<PreKeyBundles, SamApiClientError>;

    /// Publishes pre-keys for a device.
    ///
    /// This function allows a client to upload a new set of pre-keys.
    ///
    /// # Arguments
    ///
    /// * `username` - The username of the account.
    /// * `password` - The password of the account.
    /// * `pre_keys` - A [`PublishPreKeys`] containing the new pre-keys to be published.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the pre-keys were successfully published.
    /// * `Err(SamApiClientError)` if the operation fails.
    async fn publish_pre_keys(
        &self,
        username: &str,
        password: &str,
        pre_keys: PublishPreKeys,
    ) -> Result<(), SamApiClientError>;

    /// Provisions a new device for the user.
    ///
    /// This function is used when adding a new device to an existing account.
    ///
    /// # Arguments
    ///
    /// * `username` - The username of the account.
    /// * `password` - The password of the account.
    ///
    /// # Returns
    ///
    /// * `Ok(LinkDeviceToken)` containing a token to link the new device.
    /// * `Err(SamApiClientError)` if the provisioning fails.
    async fn provision_device(
        &self,
        username: &str,
        password: &str,
    ) -> Result<LinkDeviceToken, SamApiClientError>;

    /// Links a new device to the user's account.
    ///
    /// This function completes the device linking process using a previously generated link token.
    ///
    /// # Arguments
    ///
    /// * `username` - The username of the account.
    /// * `password` - The password of the account.
    /// * `request` - A [`LinkDeviceRequest`] containing the necessary details for linking the device.
    ///
    /// # Returns
    ///
    /// * `Ok(LinkDeviceResponse)` if the device was successfully linked.
    /// * `Err(SamApiClientError)` if the operation fails.
    async fn link_device(
        &self,
        username: &str,
        password: &str,
        request: LinkDeviceRequest,
    ) -> Result<LinkDeviceResponse, SamApiClientError>;

    /// Deletes a specific device from the user's account.
    ///
    /// The device must not be the primary device which can only be deleted when the account is
    /// deleted.
    ///
    /// # Arguments
    ///
    /// * `username` - The username of the account.
    /// * `password` - The password of the account.
    /// * `device_id` - The [`DeviceId`] of the device to be deleted.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the device was successfully deleted.
    /// * `Err(SamApiClientError)` if the deletion fails.
    async fn delete_device(
        &self,
        username: &str,
        password: &str,
        device_id: DeviceId,
    ) -> Result<(), SamApiClientError>;
}
