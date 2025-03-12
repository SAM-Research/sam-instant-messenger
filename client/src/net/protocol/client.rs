use std::sync::Arc;

use futures_util::{lock::Mutex, stream::SplitStream, StreamExt};
use log::error;
use prost::Message as PMessage;
use sam_common::{
    address::MessageId,
    sam_message::{
        self, server_message::Content, ClientEnvelope, ClientMessage, MessageType, ServerEnvelope,
        ServerMessage, Status,
    },
};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio_tungstenite::tungstenite::{
    protocol::{frame::coding::CloseCode, CloseFrame},
    Message,
};

use super::{
    error::ProtocolError,
    traits::{MessageStatus, SamProtocolClient},
    websocket::{WebSocket, WebSocketClient, WebSocketError, WebSocketReceiver},
};

enum ServerStatus {
    Ack(MessageId),
    Status(MessageId, Status),
}

struct SamProtocolReceiver {
    client: Arc<Mutex<WebSocketClient>>,
    enqueue_status: Sender<ServerStatus>,
    enqueue_envelope: Option<Sender<ServerEnvelope>>,
}

impl SamProtocolReceiver {
    fn new(client: Arc<Mutex<WebSocketClient>>, enqueue_status: Sender<ServerStatus>) -> Self {
        Self {
            client,
            enqueue_status,
            enqueue_envelope: None,
        }
    }

    async fn send_ack(&self, id: MessageId) -> Result<(), ProtocolError> {
        self.client
            .lock()
            .await
            .send(Message::Binary(
                ClientMessage::builder()
                    .id(id.into())
                    .r#type(MessageType::Ack as i32)
                    .build()
                    .encode_to_vec()
                    .into(),
            ))
            .await
            .map_err(ProtocolError::WebSocketError)
    }

    async fn handle_server_message(
        &mut self,
        message: ServerMessage,
    ) -> Result<Option<MessageId>, ProtocolError> {
        let id = MessageId::try_from(message.id.clone())
            .map_err(|_| ProtocolError::MalformedServerMessage)?;

        let content = match message.r#type() {
            MessageType::Message | MessageType::Status => message
                .content
                .ok_or(ProtocolError::MalformedServerMessage)?,
            MessageType::Ack => {
                return self
                    .handle_server_status(ServerStatus::Ack(id))
                    .await
                    .map(|_| None);
            }
        };

        match content {
            Content::Message(envelope) => self
                .handle_server_envelope(envelope)
                .await
                .map(|_| Some(id)),
            Content::Status(status) => self
                .handle_server_status(ServerStatus::Status(id, status))
                .await
                .map(|_| None),
        }
    }

    async fn handle_server_envelope(
        &mut self,
        envelope: ServerEnvelope,
    ) -> Result<(), ProtocolError> {
        match &self.enqueue_envelope {
            Some(sender) => sender
                .send(envelope)
                .await
                .map_err(|_| ProtocolError::WebSocketError(WebSocketError::Disconnected)),
            None => Err(ProtocolError::WebSocketError(WebSocketError::Disconnected)),
        }
    }

    async fn handle_server_status(&mut self, status: ServerStatus) -> Result<(), ProtocolError> {
        self.enqueue_status
            .send(status)
            .await
            .map_err(|_| ProtocolError::WebSocketError(WebSocketError::Disconnected))
    }
}

pub struct ProtocolClient {
    client: Arc<Mutex<WebSocketClient>>,
    status_messages: Option<Receiver<ServerStatus>>,
}

impl ProtocolClient {
    pub fn new(client: WebSocketClient) -> Self {
        Self {
            client: Arc::new(Mutex::new(client)),
            status_messages: None,
        }
    }

    async fn send_client_message(
        &mut self,
        id: MessageId,
        envelope: ClientEnvelope,
    ) -> Result<(), ProtocolError> {
        let message = ClientMessage::builder()
            .id(id.into())
            .r#type(MessageType::Message as i32)
            .message(envelope)
            .build();
        self.client
            .lock()
            .await
            .send(Message::Binary(message.encode_to_vec().into()))
            .await
            .map_err(ProtocolError::WebSocketError)
    }

    async fn handle_server_status(
        &mut self,
        req_id: MessageId,
        status: ServerStatus,
    ) -> Result<MessageStatus, ProtocolError> {
        match status {
            ServerStatus::Ack(message_id) => self
                .check_id(req_id, message_id)
                .await
                .map(|_| MessageStatus::Ok),
            ServerStatus::Status(message_id, status) => {
                self.handle_status(req_id, message_id, status).await
            }
        }
    }

    async fn check_id(
        &mut self,
        req_id: MessageId,
        res_id: MessageId,
    ) -> Result<(), ProtocolError> {
        if res_id != req_id {
            self.client
                .lock()
                .await
                .send(Message::Close(Some(CloseFrame {
                    code: CloseCode::Error,
                    reason: "Request and Response Id did not match".into(),
                })))
                .await
                .map_err(ProtocolError::WebSocketError)
        } else {
            Ok(())
        }
    }

    async fn handle_status(
        &mut self,
        req_id: MessageId,
        res_id: MessageId,
        status: Status,
    ) -> Result<MessageStatus, ProtocolError> {
        self.check_id(req_id, res_id).await?;

        match status.code() {
            sam_message::StatusCode::EmptyMessage => Err(ProtocolError::EmptyMessage),
            sam_message::StatusCode::NotEncryptedForAllDevices => {
                Ok(MessageStatus::MissingDevices(status.device_lists))
            }

            sam_message::StatusCode::EncryptedForExtraDevices => {
                Ok(MessageStatus::ExtraDevices(status.device_lists))
            }

            sam_message::StatusCode::NeedsSync => Ok(MessageStatus::NeedsSync),
        }
    }
}

#[async_trait::async_trait]
impl WebSocketReceiver<ServerEnvelope> for SamProtocolReceiver {
    async fn handler(
        &mut self,
        mut receiver: SplitStream<WebSocket>,
        enqueue: Sender<ServerEnvelope>,
    ) {
        self.enqueue_envelope = Some(enqueue);
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

            let res = match self.handle_server_message(msg).await {
                Ok(Some(id)) => self.send_ack(id).await,
                Err(x) => {
                    error!("Failed to handle server message '{x}', disconnecting...");
                    break;
                }
                Ok(None) => continue,
            };

            if res.is_err() {
                break; // disconnecting
            }
        }
        self.enqueue_envelope = None;
    }
}

#[async_trait::async_trait]
impl SamProtocolClient for ProtocolClient {
    async fn connect(&mut self) -> Result<Receiver<ServerEnvelope>, ProtocolError> {
        let (status_sender, status_receiver) = channel(10);

        let handler = SamProtocolReceiver::new(self.client.clone(), status_sender);

        self.status_messages = Some(status_receiver);
        self.client
            .lock()
            .await
            .connect(handler)
            .await
            .inspect_err(|e| error!("ProtocolClient Error: {e}"))
            .map_err(ProtocolError::WebSocketError)
    }
    async fn disconnect(&mut self) -> Result<(), ProtocolError> {
        self.status_messages = None;
        self.client
            .lock()
            .await
            .send(Message::Close(Some(CloseFrame {
                code: CloseCode::Normal,
                reason: "bye!".into(),
            })))
            .await
            .map_err(ProtocolError::WebSocketError)
    }

    async fn is_connected(&self) -> bool {
        self.client.lock().await.is_connected()
    }

    async fn send_message(
        &mut self,
        message: ClientEnvelope,
    ) -> Result<MessageStatus, ProtocolError> {
        let id = MessageId::generate();

        self.send_client_message(id, message).await?;

        let response = match &mut self.status_messages {
            // Client can only send one message at a time, and receive a response to that message
            // This means that the next status in the queue is always for the current message
            Some(status) => status
                .recv()
                .await
                .ok_or(ProtocolError::WebSocketError(WebSocketError::Disconnected)),
            None => Err(ProtocolError::WebSocketError(WebSocketError::Disconnected)),
        }?;

        self.handle_server_status(id, response).await
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use futures_util::{SinkExt, StreamExt};
    use prost::{bytes::Bytes, Message as PMessage};
    use rstest::rstest;
    use sam_common::{
        address::{AccountId, MessageId},
        sam_message::{
            server_message::Content, ClientEnvelope, ClientMessage, MessageType, SamMessageType,
            ServerEnvelope, ServerMessage,
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
            .content(Content::Message(
                ServerEnvelope::builder()
                    .id(id.into())
                    .r#type(SamMessageType::PlaintextContent as i32)
                    .content(vec![1, 2, 3])
                    .destination_account_id(AccountId::generate().into())
                    .destination_device_id(1u32)
                    .source_account_id(AccountId::generate().into())
                    .source_device_id(1u32)
                    .build(),
            ))
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
        ClientEnvelope::builder().messages(vec![]).build()
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

    async fn test_server(addr: String, actions: Vec<ServerAction>) -> Receiver<Result<(), String>> {
        let listener = TcpListener::bind(addr).await.unwrap();

        let (tx, rx) = oneshot::channel();
        tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let mut ws_stream = accept_async(stream).await.unwrap();
            let mut tx = Some(tx);
            for action in actions {
                let success = match action {
                    ServerAction::Send => server_send(&mut ws_stream, &mut tx).await,
                    ServerAction::Receive => server_receive(&mut ws_stream, &mut tx).await,
                };
                if !success {
                    break;
                }
            }
            oneshot(&mut tx, Ok(()))
        });
        rx
    }

    async fn server_send(
        ws_stream: &mut WebSocketStream<TcpStream>,
        tx: &mut Option<Sender<Result<(), String>>>,
    ) -> bool {
        let id = MessageId::generate();

        // server sends messages to client
        match send(ws_stream, server_env(id)).await {
            Ok(_) => {}
            Err(_) => {
                oneshot(tx, Err("Failed to send".to_string()));
                return false;
            }
        }

        // server expects an ack message from client
        if let Some(Ok(msg)) = ws_stream.next().await {
            let msg_res = match msg {
                Message::Binary(b) => decode_client_msg(b),
                _ => {
                    oneshot(tx, Err("Only expects binary messages".to_string()));
                    return false;
                }
            };

            let msg = match msg_res {
                Ok(msg) => msg,
                Err(_) => {
                    oneshot(
                        tx,
                        Err("Failed to decode client message when receiving".to_string()),
                    );
                    return false;
                }
            };

            if msg.r#type() != MessageType::Ack {
                oneshot(
                    tx,
                    Err("Expected Ack message got something else".to_string()),
                );
                return false;
            }

            if id.into_bytes() != *msg.id {
                oneshot(tx, Err("Ack Id is not the same as sent Id".to_string()));
                return false;
            }
        }
        true
    }

    async fn server_receive(
        ws_stream: &mut WebSocketStream<TcpStream>,
        tx: &mut Option<Sender<Result<(), String>>>,
    ) -> bool {
        if let Some(Ok(msg)) = ws_stream.next().await {
            let msg_res = match msg {
                Message::Binary(b) => decode_client_msg(b),
                frame => {
                    oneshot(
                        tx,
                        Err(format!("Received '{}' expected Message::Binary", frame)),
                    );
                    return false;
                }
            };

            let id = match msg_res {
                Ok(msg) => msg.id,
                Err(_) => {
                    oneshot(
                        tx,
                        Err("Failed to decode client message when receiving".to_string()),
                    );
                    return false;
                }
            };

            let timeout =
                tokio::time::timeout(Duration::from_millis(300), send(ws_stream, server_ack(id)))
                    .await;
            let res = match timeout {
                Ok(res) => res,
                Err(_) => {
                    oneshot(
                        tx,
                        Err("Client failed to send in time interval".to_string()),
                    );
                    return false;
                }
            };
            match res {
                Ok(_) => {}
                Err(_) => {
                    oneshot(tx, Err("Failed to send".to_string()));
                    return false;
                }
            };
        }
        true
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
