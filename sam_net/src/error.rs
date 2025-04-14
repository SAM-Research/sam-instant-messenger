use derive_more::{Display, Error, From};

#[derive(Debug, Display, Error, From)]
pub enum TlsError {
    LoadError(std::io::Error),
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
