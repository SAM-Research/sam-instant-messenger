use derive_more::{Display, Error, From};
use rustls::server::VerifierBuilderError;

#[derive(Debug, Display, Error, From)]
pub enum ClientTlsError {
    LoadError(std::io::Error),
    RustlsError(rustls::Error),
    PrivateKeyWasNone,
}

#[derive(Debug, Display, Error, From)]
pub enum ServerTlsError {
    LoadError(std::io::Error),
    VerifierError(VerifierBuilderError),
    RustlsError(rustls::Error),
    PrivateKeyWasNone,
}

#[derive(Debug, Display, Error)]
pub enum WebSocketError {
    UrlError,
    ConnectionFailed,
    Disconnected,
    AlreadyConnected,
}
