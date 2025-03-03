use derive_more::{Display, Error};

#[derive(Debug, Display, Error)]
pub enum ApiClientError {
    CouldNotParseUrl(#[error(not(source))] String),
    CouldNotBuildRequest,
    CouldNotSendRequest,
    #[display("HTTP Error {_0}: {_1}")]
    DoesNotExist(u16, String),
    #[display("Got bad response from server: {_0} - {_1}")]
    BadResponse(u16, String),
    #[display("Account already exists on server: {_0} - {_1}")]
    AccountAlreadyExists(u16, String),
    #[display("Account does not exist on server: {_0} - {_1}")]
    AccountDoesNotExist(u16, String),
    #[display("Client was not authorized by server: {_0} - {_1}")]
    ClientUnauthorized(u16, String),
    #[display("Device does not exist on the server: {_0} - {_1}")]
    DeviceDoesNotExist(u16, String),
    #[display("Key does not exist on server: {_0} - {_1}")]
    KeyDoesNotExist(u16, String),
    #[display("Failed to get device token: {_0} - {_1}")]
    DeviceProvisionUnAuth(u16, String),
    #[display("Illegal device used: {_0} - {_1}")]
    DeviceUnAuth(u16, String),
    #[display("Failed to link device: {_0} - {_1}")]
    DeviceLinkTookTooLong(u16, String),
    CouldNotParseResponse,
}
