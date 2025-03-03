use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use derive_more::Display;
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use tokio::{
    net::TcpStream,
    sync::mpsc::{self, Receiver, Sender},
};
use tokio_tungstenite::{
    connect_async_tls_with_config,
    tungstenite::{client::IntoClientRequest, http, protocol::WebSocketConfig, Message},
    Connector, MaybeTlsStream, WebSocketStream,
};

#[derive(Debug, Display)]
pub enum WebSocketError {
    UrlError,
    ConnectionFailed,
    Disconnected,
    AlreadyConnected,
}

pub type WebSocket = WebSocketStream<MaybeTlsStream<TcpStream>>;

#[derive(bon::Builder)]
pub struct WebSocketClientConfig {
    url: String,
    tungstenite_config: Option<WebSocketConfig>,
    #[builder(default = false)]
    disable_nagle: bool,
    tls: Option<Connector>,
    #[builder(default = vec![])]
    headers: Vec<(http::header::HeaderName, http::header::HeaderValue)>,
    #[builder(default = 10)]
    buffer: usize,
}

pub struct WebSocketClient {
    config: WebSocketClientConfig,

    sink: Option<SplitSink<WebSocket, Message>>,
    connected: Arc<AtomicBool>,
}

impl From<WebSocketClientConfig> for WebSocketClient {
    fn from(value: WebSocketClientConfig) -> Self {
        WebSocketClient::new(value)
    }
}

#[async_trait::async_trait]
pub trait WebSocketReceiver<T>: Send + 'static {
    async fn handler(&mut self, receiver: SplitStream<WebSocket>, enqueue: Sender<T>);
}

impl WebSocketClient {
    pub fn new(config: WebSocketClientConfig) -> Self {
        Self {
            config,
            sink: None,
            connected: Arc::new(AtomicBool::new(false)),
        }
    }

    async fn _connect(&self) -> Result<WebSocket, WebSocketError> {
        let mut req = self
            .config
            .url
            .clone()
            .into_client_request()
            .map_err(|_| WebSocketError::UrlError)?;
        let headers = req.headers_mut();
        for (name, value) in &self.config.headers {
            headers.insert(name, value.clone());
        }
        let (ws, _) = connect_async_tls_with_config(
            req,
            self.config.tungstenite_config,
            self.config.disable_nagle,
            self.config.tls.clone(),
        )
        .await
        .map_err(|_| WebSocketError::ConnectionFailed)?;
        Ok(ws)
    }

    pub async fn connect<T>(
        &mut self,
        mut ws_receiver: impl WebSocketReceiver<T>,
    ) -> Result<Receiver<T>, WebSocketError>
    where
        T: Send + 'static,
    {
        if self.is_connected() {
            return Err(WebSocketError::AlreadyConnected);
        }
        let (sender, receiver) = self._connect().await?.split();
        let (enqueue, queue) = mpsc::channel(self.config.buffer);

        self.sink = Some(sender);

        let connected = self.connected.clone();
        tokio::spawn(async move {
            connected.store(true, Ordering::SeqCst);
            ws_receiver.handler(receiver, enqueue).await;
            connected.store(false, Ordering::SeqCst);
        });
        Ok(queue)
    }

    pub fn is_connected(&self) -> bool {
        self.connected.load(Ordering::SeqCst)
    }

    pub async fn send(&mut self, message: Message) -> Result<(), WebSocketError> {
        let res = match &mut self.sink {
            Some(sender) => sender
                .send(message)
                .await
                .map_err(|_| WebSocketError::Disconnected),
            None => Err(WebSocketError::Disconnected)?,
        };

        match res {
            Ok(x) => Ok(x),
            Err(x) => {
                self.sink = None;
                Err(x)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use futures_util::stream::SplitStream;
    use futures_util::{SinkExt, StreamExt};
    use tokio::net::TcpListener;
    use tokio::sync::mpsc::Sender;
    use tokio_tungstenite::{accept_async, tungstenite::Message};

    use crate::net::protocol::websocket::WebSocketClient;
    use crate::net::protocol::websocket::WebSocketClientConfig;

    use super::{WebSocket, WebSocketReceiver};

    async fn test_server(addr: String) {
        let listener = TcpListener::bind(addr).await.unwrap();

        tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let mut ws_stream = accept_async(stream).await.unwrap();

            if let Some(Ok(msg)) = ws_stream.next().await {
                ws_stream.send(msg).await.unwrap();
            }
        });
    }

    struct WSReceiver;

    #[async_trait::async_trait]
    impl WebSocketReceiver<String> for WSReceiver {
        async fn handler(&mut self, mut receiver: SplitStream<WebSocket>, enqueue: Sender<String>) {
            if let Some(Ok(Message::Text(x))) = receiver.next().await {
                enqueue
                    .send(x.to_string())
                    .await
                    .expect("Can enqueue string")
            }
        }
    }

    #[tokio::test]
    async fn test_echo_send() {
        let addr = "127.0.0.1:9080".to_string();

        test_server(addr.clone()).await;

        let mut client: WebSocketClient = WebSocketClientConfig::builder()
            .url(format!("ws://{}", addr))
            .build()
            .into();

        let mut receiver = client.connect(WSReceiver {}).await.expect("Can Connect");

        client
            .send(Message::Text("Hello".into()))
            .await
            .expect("Can send message");

        assert!(receiver.recv().await.is_some_and(|msg| msg == "Hello"));
        assert!(!client.is_connected());
    }
}
