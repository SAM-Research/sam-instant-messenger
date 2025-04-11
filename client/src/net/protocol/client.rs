use std::sync::Arc;

use async_trait::async_trait;
use futures_util::{lock::Mutex, stream::SplitStream, StreamExt};
use log::{debug, error};
use prost::Message as PMessage;
use sam_common::{
    address::MessageId,
    sam_message::{
        ClientEnvelope, ClientMessage, ClientMessageType, ServerEnvelope, ServerMessage,
    },
};
use tokio::sync::mpsc::{self, channel, Receiver, Sender};
use tokio_tungstenite::tungstenite::{
    protocol::{frame::coding::CloseCode, CloseFrame},
    Message,
};

use super::{
    decode::{EnvelopeOrStatus, MessageStatus, ServerStatus},
    error::ProtocolError,
    websocket::{WebSocket, WebSocketClient, WebSocketError, WebSocketReceiver},
    SamProtocolClient,
};

struct SamProtocolReceiver {
    client: Arc<Mutex<WebSocketClient>>,
    enqueue_status: Sender<ServerStatus>,
    enqueue_envelope: Sender<ServerEnvelope>,
}

impl SamProtocolReceiver {
    fn new(
        client: Arc<Mutex<WebSocketClient>>,
        enqueue_status: Sender<ServerStatus>,
        enqueue_envelope: Sender<ServerEnvelope>,
    ) -> Self {
        Self {
            client,
            enqueue_status,
            enqueue_envelope,
        }
    }

    async fn send_ack(&self, id: MessageId) -> Result<(), ProtocolError> {
        self.client
            .lock()
            .await
            .send(Message::Binary(
                ClientMessage::builder()
                    .id(id.into())
                    .r#type(ClientMessageType::ClientAck.into())
                    .build()
                    .encode_to_vec()
                    .into(),
            ))
            .await
            .map_err(ProtocolError::WebSocketError)
    }

    async fn validate_and_enqueue(
        &mut self,
        message: ServerMessage,
    ) -> Result<Option<MessageId>, ProtocolError> {
        match EnvelopeOrStatus::try_from(message)? {
            EnvelopeOrStatus::Envelope(id, envelope) => self.dispatch_envelope(id, envelope).await,
            EnvelopeOrStatus::Status(status) => self.dispatch_server_status(status).await,
        }
    }

    async fn dispatch_envelope(
        &mut self,
        id: MessageId,
        envelope: ServerEnvelope,
    ) -> Result<Option<MessageId>, ProtocolError> {
        self.enqueue_envelope
            .send(envelope)
            .await
            .inspect_err(|e| debug!("{e}"))
            .map_err(|_| ProtocolError::WebSocketError(WebSocketError::Disconnected))
            .map(|_| Some(id))
    }

    async fn dispatch_server_status(
        &mut self,
        status: ServerStatus,
    ) -> Result<Option<MessageId>, ProtocolError> {
        self.enqueue_status
            .send(status)
            .await
            .inspect_err(|e| debug!("{e}"))
            .map_err(|_| ProtocolError::WebSocketError(WebSocketError::Disconnected))
            .map(|_| None)
    }
}

#[async_trait]
impl WebSocketReceiver for SamProtocolReceiver {
    async fn handler(&mut self, mut receiver: SplitStream<WebSocket>) {
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

            let res = match self.validate_and_enqueue(msg).await {
                // Some(id) is when the server has sent a message to the client that needs to be acknowledged
                Ok(Some(id)) => self.send_ack(id).await,
                Err(ProtocolError::WebSocketError(WebSocketError::Disconnected)) => {
                    break;
                }
                Err(x) => {
                    error!("Failed to handle server message '{x}', disconnecting...");
                    break;
                }
                // Ok(None) is when the server sends acks/status messages
                Ok(None) => continue,
            };

            if res.is_err() {
                break; // disconnecting
            }
        }
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
            .r#type(ClientMessageType::ClientMessage.into())
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
        match status.validate(req_id)? {
            Some(status) => Ok(status),
            None => {
                let res = self
                    .client
                    .lock()
                    .await
                    .send(Message::Close(Some(CloseFrame {
                        code: CloseCode::Error,
                        reason: "Request and Response Id did not match".into(),
                    })))
                    .await;
                match res {
                    Ok(()) => Err(ProtocolError::ReceivedWrongResponseId),
                    Err(err) => Err(ProtocolError::WebSocketError(err)),
                }
            }
        }
    }
}

#[async_trait]
impl SamProtocolClient for ProtocolClient {
    async fn connect(&mut self) -> Result<Receiver<ServerEnvelope>, ProtocolError> {
        let (status_sender, status_receiver) = channel(10);

        let (tx, rx) = mpsc::channel(10);
        let handler = SamProtocolReceiver::new(self.client.clone(), status_sender, tx);

        self.status_messages = Some(status_receiver);
        self.client
            .lock()
            .await
            .connect(handler)
            .await
            .inspect_err(|e| error!("ProtocolClient Error: {e}"))
            .map_err(ProtocolError::WebSocketError)?;
        Ok(rx)
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
    use log::debug;
    use prost::{bytes::Bytes, Message as PMessage};
    use rstest::rstest;
    use sam_common::{
        address::{AccountId, MessageId},
        sam_message::{
            server_message::Content, ClientEnvelope, ClientMessage, ClientMessageType,
            SamMessageType, ServerEnvelope, ServerMessage, ServerMessageType,
        },
    };
    use test_utils::get_next_port;
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
            .r#type(ServerMessageType::ServerMessage.into())
            .content(Content::ServerEnvelope(
                ServerEnvelope::builder()
                    .id(id.into())
                    .r#type(SamMessageType::PlaintextContent.into())
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
            .r#type(ServerMessageType::ServerAck.into())
            .build()
    }

    async fn send(stream: &mut WebSocketStream<TcpStream>, msg: ServerMessage) -> Result<(), ()> {
        stream
            .send(Message::Binary(msg.encode_to_vec().into()))
            .await
            .inspect_err(|e| debug!("{e}"))
            .map_err(|_| ())
    }

    fn decode_client_msg(bytes: Bytes) -> Result<ClientMessage, ()> {
        ClientMessage::decode(bytes)
            .inspect_err(|e| debug!("{e}"))
            .map_err(|_| ())
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

            if msg.r#type() != ClientMessageType::ClientAck {
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
    #[case(vec![ServerAction::Send, ServerAction::Send], get_next_port())]
    #[case(vec![ServerAction::Receive, ServerAction::Receive], get_next_port())]
    #[case(vec![ServerAction::Receive, ServerAction::Send], get_next_port())]
    #[case(vec![ServerAction::Send, ServerAction::Receive], get_next_port())]
    #[tokio::test]
    async fn test_send_and_ack_and_envelope(#[case] actions: Vec<ServerAction>, #[case] port: u16) {
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
