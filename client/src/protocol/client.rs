use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use futures_util::{lock::Mutex, stream::SplitStream, StreamExt};
use log::error;
use prost::{bytes::Bytes, Message as PMessage};
use sam_common::{
    address::MessageId,
    sam_message::{ClientEnvelope, ClientMessage, MessageType, ServerEnvelope, ServerMessage},
};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio_tungstenite::tungstenite::{
    protocol::{frame::coding::CloseCode, CloseFrame},
    Message,
};

use super::{
    error::SamProtocolError,
    traits::SamProtocolClient,
    websocket::{WebSocket, WebSocketClient, WebSocketError},
};

pub struct ProtocolClient {
    client: Arc<Mutex<WebSocketClient>>,
    status_messages: Option<Receiver<ServerMessage>>,
}

impl ProtocolClient {
    pub fn new(client: WebSocketClient) -> Self {
        Self {
            client: Arc::new(Mutex::new(client)),
            status_messages: None,
        }
    }
}

async fn protocol_handler(
    mut receiver: SplitStream<WebSocket>,
    enqueue: Sender<ServerEnvelope>,
    client: Arc<Mutex<WebSocketClient>>,
    enqueue_status: Sender<ServerMessage>,
    connected: Arc<AtomicBool>,
) {
    connected.store(true, Ordering::SeqCst);

    while let Some(Ok(msg)) = receiver.next().await {
        let res = match msg {
            Message::Binary(b) => ServerMessage::decode(Bytes::from(b)),
            Message::Close(_) => break,
            _ => continue,
        };

        let msg = match res {
            Ok(msg) => msg,
            Err(e) => {
                error!("Failed to decode message from server '{e}', disconnecting...");
                break;
            }
        };

        let envelope = match msg.r#type() {
            MessageType::Message => msg.message,
            _ => {
                match enqueue_status.send(msg).await {
                    Ok(_) => continue,
                    Err(_) => break, // disconnected
                }
            }
        };

        let res = match envelope {
            Some(envelope) => {
                let id = envelope.id.clone();
                enqueue.send(envelope).await.map(|_| id)
            }
            None => {
                error!("Malformed server message, disconnecting...");
                break;
            }
        };

        let ack_res = match res {
            Ok(id) => client
                .lock()
                .await
                .send(Message::Binary(
                    ClientMessage::builder()
                        .id(id)
                        .r#type(MessageType::Ack as i32)
                        .build()
                        .encode_to_vec()
                        .into(),
                ))
                .await
                .map_err(|_| ()),
            Err(_) => Err(()),
        };
        if ack_res.is_err() {
            break; // disconnected
        }
    }

    connected.store(false, Ordering::SeqCst);
}

#[async_trait::async_trait]
impl SamProtocolClient for ProtocolClient {
    async fn connect(&mut self) -> Result<Receiver<ServerEnvelope>, SamProtocolError> {
        let (status_sender, status_receiver) = channel(10);

        let client = self.client.clone();

        let handler = move |receiver: SplitStream<WebSocket>,
                            enqueue: Sender<ServerEnvelope>,
                            connected: Arc<AtomicBool>| {
            let status_sender = status_sender.clone();
            let client = client.clone();
            async move { protocol_handler(receiver, enqueue, client, status_sender, connected).await }
        };

        self.status_messages = Some(status_receiver);
        self.client
            .lock()
            .await
            .connect(handler)
            .await
            .map_err(SamProtocolError::from)
    }
    async fn disconnect(&mut self) -> Result<(), SamProtocolError> {
        self.status_messages = None;
        self.client
            .lock()
            .await
            .send(Message::Close(Some(CloseFrame {
                code: CloseCode::Normal,
                reason: "bye!".into(),
            })))
            .await
            .map_err(SamProtocolError::from)
    }

    async fn is_connected(&self) -> bool {
        self.client.lock().await.is_connected()
    }

    async fn send_message(&mut self, message: ClientEnvelope) -> Result<(), SamProtocolError> {
        let id = MessageId::generate();

        let message = ClientMessage::builder()
            .id(id.into())
            .r#type(MessageType::Message as i32)
            .message(message)
            .build();
        self.client
            .lock()
            .await
            .send(Message::Binary(message.encode_to_vec().into()))
            .await?;

        let response = match &mut self.status_messages {
            Some(status) => status.recv().await.ok_or(WebSocketError::Disconnected),
            None => Err(WebSocketError::Disconnected),
        }?;

        let is_match = match MessageId::try_from(response.id.clone()) {
            Ok(res_id) => res_id == id && response.r#type() == MessageType::Ack,
            Err(_) => Err(SamProtocolError::MalformedServerResponse)?,
        };

        if is_match {
            Ok(())
        } else {
            Err(SamProtocolError::MalformedServerResponse)
        }
    }
}

#[cfg(test)]
mod test {
    use std::{collections::HashMap, time::Duration};

    use futures_util::{SinkExt, StreamExt};
    use prost::{bytes::Bytes, Message as PMessage};
    use sam_common::{
        address::{AccountId, MessageId},
        sam_message::{
            ClientEnvelope, ClientMessage, EnvelopeType, MessageType, ServerEnvelope, ServerMessage,
        },
    };
    use tokio::net::TcpListener;
    use tokio_tungstenite::{accept_async, tungstenite::Message};

    use crate::protocol::{
        client::ProtocolClient,
        traits::SamProtocolClient,
        websocket::{WebSocketClient, WebSocketClientConfig},
    };

    async fn test_server(addr: String) {
        let listener = TcpListener::bind(addr).await.unwrap();

        tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let mut ws_stream = accept_async(stream).await.unwrap();

            if let Some(Ok(msg)) = ws_stream.next().await {
                let id = match msg {
                    Message::Binary(b) => {
                        ClientMessage::decode(Bytes::from(b))
                            .expect("Can decode client message")
                            .id
                    }
                    _ => return,
                };
                let server_msg = ServerMessage::builder()
                    .id(id)
                    .r#type(MessageType::Ack as i32)
                    .build();
                let env_id = MessageId::generate();
                let env_msg = ServerMessage::builder()
                    .id(env_id.into())
                    .r#type(MessageType::Message as i32)
                    .message(
                        ServerEnvelope::builder()
                            .id(env_id.into())
                            .r#type(EnvelopeType::PlaintextContent as i32)
                            .content(vec![1, 2, 3])
                            .destination_account_id(AccountId::generate().into())
                            .destination_device_id(1u32)
                            .source_account_id(AccountId::generate().into())
                            .source_device_id(1u32)
                            .build(),
                    )
                    .build();
                ws_stream
                    .send(Message::Binary(server_msg.encode_to_vec().into()))
                    .await
                    .expect("Can send message");
                ws_stream
                    .send(Message::Binary(env_msg.encode_to_vec().into()))
                    .await
                    .expect("Can send message");
            }
        });
    }

    #[tokio::test]
    async fn test_send_and_ack_and_envelope() {
        let addr = "127.0.0.1:9081".to_string();
        test_server(addr.clone()).await;
        let client: WebSocketClient = WebSocketClientConfig::builder()
            .url(format!("ws://{}", addr))
            .build()
            .into();

        let mut client = ProtocolClient::new(client);
        let mut receiver = client.connect().await.expect("Can connect");

        let msg = ClientEnvelope::builder()
            .r#type(MessageType::Message as i32)
            .content(HashMap::new())
            .destination_account_id(AccountId::generate().into())
            .source_device_id(1u32)
            .source_account_id(AccountId::generate().into())
            .build();
        client
            .send_message(msg)
            .await
            .expect("Can send and receive Message");
        let envelope = tokio::time::timeout(Duration::from_millis(300), receiver.recv())
            .await
            .expect("Server responds");
        assert!(envelope.is_some());
    }
}
