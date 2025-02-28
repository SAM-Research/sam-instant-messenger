use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use futures_util::{lock::Mutex, stream::SplitStream, StreamExt};
use log::error;
use prost::Message as PMessage;
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
            Message::Binary(b) => ServerMessage::decode(b),
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

        match MessageId::try_from(response.id.clone()) {
            Ok(res_id) => {
                if res_id == id && response.r#type() == MessageType::Ack {
                    Ok(())
                } else {
                    Err(SamProtocolError::MalformedServerResponse)
                }
            }
            Err(_) => Err(SamProtocolError::MalformedServerResponse),
        }
    }
}

#[cfg(test)]
mod test {
    use std::{collections::HashMap, time::Duration};

    use futures_util::{SinkExt, StreamExt};
    use prost::{bytes::Bytes, Message as PMessage};
    use rstest::rstest;
    use sam_common::{
        address::{AccountId, MessageId},
        sam_message::{
            ClientEnvelope, ClientMessage, EnvelopeType, MessageType, ServerEnvelope, ServerMessage,
        },
    };
    use tokio::{
        net::{TcpListener, TcpStream},
        sync::oneshot::{self, Receiver, Sender},
    };
    use tokio_tungstenite::{accept_async, tungstenite::Message, WebSocketStream};

    use crate::net::protocol::{
        client::ProtocolClient,
        traits::SamProtocolClient,
        websocket::{WebSocketClient, WebSocketClientConfig},
    };

    fn server_env(id: MessageId) -> ServerMessage {
        ServerMessage::builder()
            .id(id.into())
            .r#type(MessageType::Message as i32)
            .message(
                ServerEnvelope::builder()
                    .id(id.into())
                    .r#type(EnvelopeType::PlaintextContent as i32)
                    .content(vec![1, 2, 3])
                    .destination_account_id(AccountId::generate().into())
                    .destination_device_id(1u32)
                    .source_account_id(AccountId::generate().into())
                    .source_device_id(1u32)
                    .build(),
            )
            .build()
    }

    fn server_ack(id: Vec<u8>) -> ServerMessage {
        ServerMessage::builder()
            .id(id)
            .r#type(MessageType::Ack as i32)
            .build()
    }

    async fn send(stream: &mut WebSocketStream<TcpStream>, msg: ServerMessage) -> Result<(), ()> {
        stream
            .send(Message::Binary(msg.encode_to_vec().into()))
            .await
            .map_err(|_| ())
    }

    fn decode_client_msg(bytes: Bytes) -> Result<ClientMessage, ()> {
        ClientMessage::decode(bytes).map_err(|_| ())
    }

    fn client_env() -> ClientEnvelope {
        ClientEnvelope::builder()
            .r#type(MessageType::Message as i32)
            .content(HashMap::new())
            .destination_account_id(AccountId::generate().into())
            .source_device_id(1u32)
            .source_account_id(AccountId::generate().into())
            .build()
    }

    fn oneshot(tx: &mut Option<Sender<Result<(), String>>>, msg: Result<(), String>) {
        if let Some(tx) = tx.take() {
            let _ = tx.send(msg);
        }
    }

    #[derive(Clone)]
    enum ServerAction {
        Receive,
        Send,
    }

    async fn test_server(addr: String, msg_seq: Vec<ServerAction>) -> Receiver<Result<(), String>> {
        let listener = TcpListener::bind(addr).await.unwrap();

        let (tx, rx) = oneshot::channel();
        tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let mut ws_stream = accept_async(stream).await.unwrap();
            let mut tx = Some(tx);
            for (i, action) in msg_seq.iter().enumerate() {
                match action {
                    ServerAction::Send => {
                        let id = MessageId::generate();

                        // server sends messages to client
                        match send(&mut ws_stream, server_env(id)).await {
                            Ok(_) => {}
                            Err(_) => {
                                oneshot(&mut tx, Err("Failed to send".to_string()));
                                break;
                            }
                        }

                        // server expects an ack message from client
                        if let Some(Ok(msg)) = ws_stream.next().await {
                            let msg_res = match msg {
                                Message::Binary(b) => decode_client_msg(b),
                                _ => {
                                    oneshot(
                                        &mut tx,
                                        Err("Only expects binary messages".to_string()),
                                    );
                                    break;
                                }
                            };

                            let msg = match msg_res {
                                Ok(msg) => msg,
                                Err(_) => {
                                    oneshot(
                                        &mut tx,
                                        Err("Failed to decode client message when receiving"
                                            .to_string()),
                                    );
                                    break;
                                }
                            };

                            if msg.r#type() != MessageType::Ack {
                                oneshot(
                                    &mut tx,
                                    Err("Expected Ack message got something else".to_string()),
                                );
                                break;
                            }

                            if id.into_bytes() != *msg.id {
                                oneshot(
                                    &mut tx,
                                    Err("Ack Id is not the same as sent Id".to_string()),
                                );
                                break;
                            }
                        }
                    }
                    ServerAction::Receive => {
                        if let Some(Ok(msg)) = ws_stream.next().await {
                            let msg_res = match msg {
                                Message::Binary(b) => decode_client_msg(b),
                                _ => return,
                            };

                            let id = match msg_res {
                                Ok(msg) => msg.id,
                                Err(_) => {
                                    oneshot(
                                        &mut tx,
                                        Err("Failed to decode client message when receiving"
                                            .to_string()),
                                    );
                                    break;
                                }
                            };

                            let timeout = tokio::time::timeout(
                                Duration::from_millis(300),
                                send(&mut ws_stream, server_ack(id)),
                            )
                            .await;
                            let res = match timeout {
                                Ok(res) => res,
                                Err(_) => {
                                    oneshot(
                                        &mut tx,
                                        Err("Client failed to send in time interval".to_string()),
                                    );
                                    break;
                                }
                            };
                            match res {
                                Ok(_) => {}
                                Err(_) => {
                                    oneshot(&mut tx, Err("Failed to send".to_string()));
                                    break;
                                }
                            };
                        }
                    }
                }
                if i == msg_seq.len() - 1 {
                    oneshot(&mut tx, Ok(()));
                }
            }
        });
        rx
    }

    #[rstest]
    #[case(vec![ServerAction::Send, ServerAction::Send], "9081")]
    #[case(vec![ServerAction::Receive, ServerAction::Receive],"9082")]
    #[case(vec![ServerAction::Receive, ServerAction::Send], "9083")]
    #[case(vec![ServerAction::Send, ServerAction::Receive], "9084")]
    #[tokio::test]
    async fn test_send_and_ack_and_envelope(
        #[case] actions: Vec<ServerAction>,
        #[case] port: String,
    ) {
        let addr = format!("127.0.0.1:{}", port);
        let shutdown = test_server(addr.clone(), actions.clone()).await;
        let client: WebSocketClient = WebSocketClientConfig::builder()
            .url(format!("ws://{}", addr))
            .build()
            .into();

        let mut client = ProtocolClient::new(client);
        let mut receiver = client.connect().await.expect("Can connect");

        let mut results = vec![];
        for action in actions {
            let ok = match action {
                ServerAction::Send => {
                    tokio::time::timeout(Duration::from_millis(300), receiver.recv())
                        .await
                        .is_ok()
                }
                ServerAction::Receive => client.send_message(client_env()).await.is_ok(),
            };
            results.push(ok);
            if !ok {
                break;
            }
        }

        assert!(results.iter().all(|x| *x));
        let timeout = tokio::time::timeout(Duration::from_millis(300), shutdown).await;
        let recv_res = timeout.expect("Server shutsdown");
        let res = recv_res.expect("Oneshot works");
        res.expect("Client server comms works");
    }
}
