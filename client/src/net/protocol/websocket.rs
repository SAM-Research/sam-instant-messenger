use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

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

#[derive(Debug)]
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

    pub async fn connect<T, F, Fut>(
        &mut self,
        receive_handler: F,
    ) -> Result<Receiver<T>, WebSocketError>
    where
        F: Fn(SplitStream<WebSocket>, Sender<T>, Arc<AtomicBool>) -> Fut,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        if self.is_connected() {
            return Err(WebSocketError::AlreadyConnected);
        }
        let (sender, receiver) = self._connect().await?.split();
        let (enqueue, queue) = mpsc::channel(self.config.buffer);

        self.sink = Some(sender);

        // TODO: when async closures are allowed we need to make it so
        // the thread is responsible for the connected bool instead of the handler
        tokio::spawn(receive_handler(receiver, enqueue, self.connected.clone()));
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
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    use futures_util::stream::SplitStream;
    use futures_util::{SinkExt, StreamExt};
    use tokio::net::TcpListener;
    use tokio::sync::mpsc::Sender;
    use tokio_tungstenite::{accept_async, tungstenite::Message};

    use crate::net::protocol::websocket::WebSocketClient;
    use crate::net::protocol::websocket::WebSocketClientConfig;

    use super::WebSocket;

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

    async fn client_receiver(
        mut receiver: SplitStream<WebSocket>,
        enqueue: Sender<String>,
        connected: Arc<AtomicBool>,
    ) {
        connected.store(true, Ordering::SeqCst);
        if let Some(Ok(Message::Text(x))) = receiver.next().await {
            enqueue
                .send(x.to_string())
                .await
                .expect("Can enqueue string")
        }
        connected.store(false, Ordering::SeqCst);
    }

    #[tokio::test]
    async fn test_echo_send() {
        let addr = "127.0.0.1:9080".to_string();

        test_server(addr.clone()).await;

        let mut client: WebSocketClient = WebSocketClientConfig::builder()
            .url(format!("ws://{}", addr))
            .build()
            .into();

        let mut receiver = client.connect(client_receiver).await.expect("Can Connect");

        client
            .send(Message::Text("Hello".into()))
            .await
            .expect("Can send message");

        assert!(receiver.recv().await.is_some_and(|msg| msg == "Hello"));
        assert!(!client.is_connected());
    }
}
