use derive_more::{Display, Error, From};

#[derive(Debug, Display, Error)]
pub enum ApiClientError {
    CouldNotParseUrl(#[error(not(source))] String),
    CouldNotBuildRequest,
    CouldNotSendRequest,
    #[display("Got bad response from server: {_0} - {_1}")]
    ErrorResponse(u16, String),
    CouldNotParseResponse,
    FailedToBuildApiClient,
}

#[derive(Debug, Display, Error, From)]
pub enum TLSError {
    LoadError(std::io::Error),
    RustlsError(rustls::Error),
    PrivateKeyWasNone,
}
