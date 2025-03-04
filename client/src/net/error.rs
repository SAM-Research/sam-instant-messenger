use derive_more::Display;

#[derive(Debug, Display)]
pub enum ApiClientError {
    CouldNotParseUrl(String),
    CouldNotBuildRequest,
    CouldNotSendRequest,
    #[display("Got bad response from server: {_0} - {_1}")]
    ErrorResponse(u16, String),
    CouldNotParseResponse,
}
