use axum::{
    extract::{State, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};

use crate::{
    auth::authenticated_user::AuthenticatedUser,
    logic::websocket::init_websocket,
    managers::traits::message_manager::MessageManager,
    state::{state_type::StateType, ServerState},
    ServerError,
};

async fn websocket_endpoint<T: StateType>(
    State(mut state): State<ServerState<T>>,
    auth_user: AuthenticatedUser,
    ws: WebSocketUpgrade,
) -> Result<impl IntoResponse, ServerError> {
    let account_id = auth_user.account().id();
    let device_id = auth_user.device().id();
    let dispatch = state.messages.subscribe(account_id, device_id).await?;
    state
        .messages
        .dispatch_envelopes(account_id, device_id)
        .await?;

    Ok(ws.on_upgrade(move |socket| async move {
        init_websocket(state, auth_user, socket, dispatch).await
    }))
}

pub fn websocket_routes<T: StateType>(router: Router<ServerState<T>>) -> Router<ServerState<T>> {
    router.route("/api/v1/websocket", get(websocket_endpoint))
}

#[cfg(test)]
mod test {
    use std::{io::Error, net::SocketAddr, time::Duration};

    use axum::Router;
    use axum_server::Handle;
    use base64::{prelude::BASE64_STANDARD, Engine};
    use bon::vec;
    use futures_util::{SinkExt, StreamExt};

    use maplit::hashmap;
    use prost::Message as _;
    use rand::rngs::OsRng;
    use sam_common::{
        address::{AccountId, DeviceId, MessageId},
        sam_message::{
            server_message::Content, ClientEnvelope, ClientMessage, EnvelopeType, MessageType,
            ServerMessage,
        },
    };

    use tokio::{sync::oneshot, task::JoinHandle};
    use tokio_tungstenite::{
        connect_async,
        tungstenite::{client::IntoClientRequest, Message},
        MaybeTlsStream, WebSocketStream,
    };

    use crate::{
        auth::password::Password,
        managers::{entities::device::Device, traits::device_manager::DeviceManager},
        routes::{test_utils::create_user, websocket::websocket_routes},
        state::{state_type::StateType, ServerState},
    };
    use tokio::sync::oneshot::Receiver;

    fn start_websocket_server<T: StateType>(
        state: ServerState<T>,
        address: String,
    ) -> (JoinHandle<Result<(), Error>>, Handle, Receiver<()>) {
        let app = websocket_routes(Router::new()).with_state(state);
        let (tx, started_rx) = oneshot::channel::<()>();
        let axum = Handle::new();
        let axum_handle = axum.clone();
        let thread = tokio::spawn(async move {
            let server = axum_server::bind(address.parse().expect("Can make socket addr from str"))
                .handle(axum_handle)
                .serve(app.into_make_service_with_connect_info::<SocketAddr>());
            tx.send(()).expect("Can oneshot");
            server.await
        });
        (thread, axum, started_rx)
    }

    async fn connect_user(
        account_id: AccountId,
        device_id: DeviceId,
        username: &str,
        password: &str,
        address: &str,
    ) -> WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>> {
        let mut req = format!("ws://{}/api/v1/websocket", address)
            .into_client_request()
            .expect("Can make url into ws upgrade req");
        let basic = format!(
            "Basic {}",
            BASE64_STANDARD.encode(format!("{}.{}:{}", account_id, device_id, password))
        );
        req.headers_mut()
            .insert("Authorization", basic.parse().unwrap());
        let (ws, _) = connect_async(req)
            .await
            .inspect_err(|e| println!("{}", e))
            .unwrap_or_else(|_| {
                panic!("{} can make connection with device {}", username, device_id)
            });
        ws
    }

    #[tokio::test]
    async fn test_websocket_alice_send_to_bob() {
        let mut state = ServerState::in_memory_test();
        let (_, alice_id, alice_device) =
            create_user(&mut state, "alice", "phone", "bob", OsRng).await;
        let (_, bob_id, bob_device) =
            create_user(&mut state, "bob", "laptop", "cheeseburger", OsRng).await;

        let address = "127.0.0.1:8001".to_string();
        let (thread, axum, started) = start_websocket_server(state.clone(), address.clone());
        started.await.expect("Server can start");

        let envelope = ClientEnvelope::builder()
            .destination_account_id(bob_id.into())
            .source_account_id(alice_id.into())
            .source_device_id(alice_device.into())
            .r#type(EnvelopeType::PlaintextContent as i32)
            .content(hashmap! {bob_device.into() => "hi bob<3".into()})
            .build();

        let msg_id = MessageId::generate();
        let msg = ClientMessage::builder()
            .id(msg_id.into())
            .message(envelope)
            .r#type(MessageType::Message as i32)
            .build();

        let mut alice = connect_user(alice_id, 1.into(), "alice", "bob", &address).await;
        let mut bob = connect_user(bob_id, 1.into(), "bob", "cheeseburger", &address).await;

        let alice_send = tokio::time::timeout(
            Duration::from_millis(300),
            alice.send(tokio_tungstenite::tungstenite::Message::Binary(
                msg.encode_to_vec().into(),
            )),
        );

        let bob_recv = tokio::time::timeout(Duration::from_millis(300), bob.next());

        let alice_sent = alice_send.await;
        let bob_received = bob_recv.await;

        axum.shutdown();
        let _ = thread.await;
        assert!(alice_sent.is_ok(), "Alice timed out");
        assert!(
            alice_sent.is_ok_and(|res| res.is_ok()),
            "Alice could not send"
        );

        assert!(bob_received.is_ok(), "{}", bob_received.unwrap_err());
        assert!(
            bob_received.is_ok_and(|op| op.is_some_and(|res| res.is_ok())),
            "Bob could not received"
        )
    }

    #[tokio::test]
    async fn test_websocket_alice_send_to_bob_offline() {
        let mut state = ServerState::in_memory_test();
        let (_, alice_id, alice_device) =
            create_user(&mut state, "alice", "phone", "bob", OsRng).await;
        let (_, bob_id, bob_device) =
            create_user(&mut state, "bob", "laptop", "cheeseburger", OsRng).await;

        let address = "127.0.0.1:8002".to_string();
        let (thread, axum, started) = start_websocket_server(state.clone(), address.clone());
        started.await.expect("Server can start");

        let envelope = ClientEnvelope::builder()
            .destination_account_id(bob_id.into())
            .source_account_id(alice_id.into())
            .source_device_id(alice_device.into())
            .r#type(EnvelopeType::PlaintextContent as i32)
            .content(hashmap! {bob_device.into() => "hi bob<3".into()})
            .build();

        let msg_id = MessageId::generate();
        let msg = ClientMessage::builder()
            .id(msg_id.into())
            .message(envelope)
            .r#type(MessageType::Message as i32)
            .build();

        let mut alice = connect_user(alice_id, 1.into(), "alice", "bob", &address).await;

        let alice_send = tokio::time::timeout(
            Duration::from_millis(300),
            alice.send(tokio_tungstenite::tungstenite::Message::Binary(
                msg.encode_to_vec().into(),
            )),
        );
        let alice_sent = alice_send.await;

        // bob goes online to receive message
        let mut bob = connect_user(bob_id, 1.into(), "bob", "cheeseburger", &address).await;
        let bob_recv = tokio::time::timeout(Duration::from_millis(300), bob.next());
        let bob_received = bob_recv.await;

        axum.shutdown();
        let _ = thread.await;
        assert!(alice_sent.is_ok(), "Alice timed out");
        assert!(
            alice_sent.is_ok_and(|res| res.is_ok()),
            "Alice could not send"
        );
        assert!(bob_received.is_ok(), "Bob timed out");
        assert!(
            bob_received.is_ok_and(|op| op.is_some_and(|res| res.is_ok())),
            "Bob could not received"
        )
    }

    #[tokio::test]
    async fn alice_send_to_bob_missing_devices() {
        let mut state = ServerState::in_memory_test();
        let (_, alice_id, alice_device) =
            create_user(&mut state, "alice", "phone", "bob", OsRng).await;
        let (_, bob_id, bob_device) =
            create_user(&mut state, "bob", "laptop", "cheeseburger", OsRng).await;

        state
            .devices
            .add_device(
                bob_id,
                &Device::builder()
                    .creation(0)
                    .id(27.into())
                    .registration_id(1.into())
                    .name("Device 27".to_string())
                    .password(
                        Password::generate("password".to_string())
                            .expect("Password can be generated"),
                    )
                    .build(),
            )
            .await
            .expect("can add extra device");

        let address = "127.0.0.1:8003".to_string();
        let (thread, axum, started) = start_websocket_server(state.clone(), address.clone());
        started.await.expect("Server can start");

        let envelope = ClientEnvelope::builder()
            .destination_account_id(bob_id.into())
            .source_account_id(alice_id.into())
            .source_device_id(alice_device.into())
            .r#type(EnvelopeType::PlaintextContent as i32)
            .content(hashmap! {bob_device.into() => "hi bob<3".into()})
            .build();

        let msg_id = MessageId::generate();
        let msg = ClientMessage::builder()
            .id(msg_id.into())
            .message(envelope)
            .r#type(MessageType::Message as i32)
            .build();

        let mut alice = connect_user(alice_id, 1.into(), "alice", "bob", &address).await;

        let alice_send = tokio::time::timeout(
            Duration::from_millis(300),
            alice.send(tokio_tungstenite::tungstenite::Message::Binary(
                msg.encode_to_vec().into(),
            )),
        );
        let alice_sent = alice_send.await;

        let alice_recv = tokio::time::timeout(Duration::from_millis(300), alice.next());
        let alice_received = alice_recv.await;

        axum.shutdown();
        let _ = thread.await;
        assert!(alice_sent.is_ok(), "Alice timed out while sending");
        assert!(
            alice_sent.is_ok_and(|res| res.is_ok()),
            "Alice could not send"
        );
        assert!(alice_received.is_ok(), "Alice timed out while receiving");

        let msg = match alice_received {
            Ok(Some(Ok(Message::Binary(msg)))) => Some(msg),
            _ => None,
        }
        .expect("alice should recieve a binary WebSocketMessage");

        let content = ServerMessage::decode(msg)
            .expect("should be able to decode message")
            .content
            .expect("message should contain content");

        let missing_devices = match content {
            Content::Error(error) => error.device_ids.ids,
            _ => vec![],
        };

        let expected: Vec<u32> = vec![27u32];

        assert!(missing_devices == expected)
    }

    #[tokio::test]
    async fn alice_send_to_bob_two_devices() {
        let mut state = ServerState::in_memory_test();
        let (_, alice_id, alice_device) =
            create_user(&mut state, "alice", "phone", "bob", OsRng).await;
        let (_, bob_id, bob_device) =
            create_user(&mut state, "bob", "laptop", "cheeseburger", OsRng).await;

        state
            .devices
            .add_device(
                bob_id,
                &Device::builder()
                    .creation(0)
                    .id(27.into())
                    .registration_id(43284.into())
                    .name("Device 27".to_string())
                    .password(
                        Password::generate("password".to_string())
                            .expect("Password can be generated"),
                    )
                    .build(),
            )
            .await
            .expect("can add extra device");

        let address = "127.0.0.1:8004".to_string();
        let (thread, axum, started) = start_websocket_server(state.clone(), address.clone());
        started.await.expect("Server can start");

        let envelope = ClientEnvelope::builder()
            .destination_account_id(bob_id.into())
            .source_account_id(alice_id.into())
            .source_device_id(alice_device.into())
            .r#type(EnvelopeType::PlaintextContent as i32)
            .content(hashmap! {
                bob_device.into() => "hi bob<3".into(),
                27u32 => "Hello, World!".into()
            })
            .build();

        let msg_id = MessageId::generate();
        let msg = ClientMessage::builder()
            .id(msg_id.into())
            .message(envelope)
            .r#type(MessageType::Message as i32)
            .build();

        let mut alice = connect_user(alice_id, 1.into(), "alice", "bob", &address).await;

        let alice_send = tokio::time::timeout(
            Duration::from_millis(300),
            alice.send(tokio_tungstenite::tungstenite::Message::Binary(
                msg.encode_to_vec().into(),
            )),
        );
        let alice_sent = alice_send.await;

        let alice_recv = tokio::time::timeout(Duration::from_millis(300), alice.next());
        let alice_received = alice_recv.await;

        let mut bob1 = connect_user(bob_id, 1.into(), "bob", "cheeseburger", &address).await;
        let bob_recv1 = tokio::time::timeout(Duration::from_millis(300), bob1.next());
        let bob_received1 = bob_recv1.await;

        let mut bob27 = connect_user(bob_id, 27.into(), "bob", "password", &address).await;
        let bob_recv27 = tokio::time::timeout(Duration::from_millis(300), bob27.next());
        let bob_received27 = bob_recv27.await;

        axum.shutdown();
        let _ = thread.await;
        assert!(alice_sent.is_ok(), "Alice timed out while sending");
        assert!(
            alice_sent.is_ok_and(|res| res.is_ok()),
            "Alice could not send"
        );
        assert!(alice_received.is_ok(), "Alice timed out while receiving");

        let ws_msg = match alice_received {
            Ok(Some(Ok(Message::Binary(msg)))) => Some(msg),
            _ => None,
        }
        .expect("alice should recieve a binary WebSocketMessage");

        let serv_msg = ServerMessage::decode(ws_msg).expect("should be able to decode message");

        assert!(matches!(serv_msg.r#type(), MessageType::Ack));

        assert!(bob_received1.is_ok(), "Bob device 1 timed out");
        assert!(
            bob_received1.is_ok_and(|op| op.is_some_and(|res| res.is_ok())),
            "Bob device 1 could not received"
        );

        assert!(bob_received27.is_ok(), "Bob device 27 timed out");
        assert!(
            bob_received27.is_ok_and(|op| op.is_some_and(|res| res.is_ok())),
            "Bob device 27 could not received"
        );
    }

    /// Alice sends a message to bob's device 1 and 27, but Bob does not have a device 27.
    /// The server responds with a message saying that device 27 does not exist, but the message is
    /// still delivered to bob's device 1.
    #[tokio::test]
    async fn alice_send_to_bob_extra_device() {
        let mut state = ServerState::in_memory_test();
        let (_, alice_id, alice_device) =
            create_user(&mut state, "alice", "phone", "bob", OsRng).await;
        let (_, bob_id, bob_device) =
            create_user(&mut state, "bob", "laptop", "cheeseburger", OsRng).await;

        let address = "127.0.0.1:8005".to_string();
        let (thread, axum, started) = start_websocket_server(state.clone(), address.clone());
        started.await.expect("Server can start");

        let envelope = ClientEnvelope::builder()
            .destination_account_id(bob_id.into())
            .source_account_id(alice_id.into())
            .source_device_id(alice_device.into())
            .r#type(EnvelopeType::PlaintextContent as i32)
            .content(hashmap! {
                bob_device.into() => "hi bob<3".into(),
                27u32 => "Hello, World!".into()
            })
            .build();

        let msg_id = MessageId::generate();
        let msg = ClientMessage::builder()
            .id(msg_id.into())
            .message(envelope)
            .r#type(MessageType::Message as i32)
            .build();

        let mut alice = connect_user(alice_id, 1.into(), "alice", "bob", &address).await;

        let alice_send = tokio::time::timeout(
            Duration::from_millis(300),
            alice.send(tokio_tungstenite::tungstenite::Message::Binary(
                msg.encode_to_vec().into(),
            )),
        );
        let alice_sent = alice_send.await;

        let alice_recv = tokio::time::timeout(Duration::from_millis(300), alice.next());
        let alice_received = alice_recv.await;

        // bob goes online to receive message
        let mut bob = connect_user(bob_id, 1.into(), "bob", "cheeseburger", &address).await;
        let bob_recv = tokio::time::timeout(Duration::from_millis(300), bob.next());
        let bob_received = bob_recv.await;

        axum.shutdown();
        let _ = thread.await;
        assert!(alice_sent.is_ok(), "Alice timed out while sending");
        assert!(
            alice_sent.is_ok_and(|res| res.is_ok()),
            "Alice could not send"
        );
        assert!(alice_received.is_ok(), "Alice timed out while receiving");

        let msg = match alice_received {
            Ok(Some(Ok(Message::Binary(msg)))) => Some(msg),
            _ => None,
        }
        .expect("alice should recieve a binary WebSocketMessage");

        let content = ServerMessage::decode(msg)
            .expect("should be able to decode message")
            .content
            .expect("message should contain content");

        let missing_devices = match content {
            Content::Error(error) => error.device_ids.ids,
            _ => vec![],
        };

        let expected: Vec<u32> = vec![27u32];

        assert!(missing_devices == expected);
        assert!(bob_received.is_ok(), "Bob timed out");
        assert!(
            bob_received.is_ok_and(|op| op.is_some_and(|res| res.is_ok())),
            "Bob could not received"
        )
    }
}
