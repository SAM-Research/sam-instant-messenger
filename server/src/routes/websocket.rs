use axum::{
    extract::{State, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};

use crate::protocol::websocket::init_websocket;
use crate::{
    auth::authenticated_user::AuthenticatedUser,
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

    use prost::Message as _;
    use rand::rngs::OsRng;
    use sam_common::{
        address::{AccountId, DeviceId, MessageId},
        sam_message::{
            ClientEnvelope, ClientMessage, ClientMessageType, SamMessage, SamMessageType,
            ServerMessage, ServerMessageType,
        },
    };

    use sam_test_utils::get_next_port;
    use tokio::{sync::oneshot, task::JoinHandle};
    use tokio_tungstenite::{
        connect_async, tungstenite::client::IntoClientRequest, MaybeTlsStream, WebSocketStream,
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
    async fn alice_send_to_bob_does_not_need_sync() {
        // TODO: Move this to the E2E when client supports sending to self in the same message as two recipient
        let mut state = ServerState::in_memory_test();
        let (_, alice_id, _) = create_user(&mut state, "alice", "phone", "bob", OsRng).await;
        let (_, bob_id, bob_device) =
            create_user(&mut state, "bob", "laptop", "cheeseburger", OsRng).await;

        state
            .devices
            .add_device(
                alice_id,
                &Device::builder()
                    .creation(0)
                    .id(27.into())
                    .registration_id(43284.into())
                    .name("Device 27".to_string())
                    .password(
                        Password::generate("password".to_string(), &mut state.rng)
                            .expect("Password can be generated"),
                    )
                    .build(),
            )
            .await
            .expect("can add extra device");

        let address = format!("127.0.0.1:{}", get_next_port());
        let (thread, axum, started) = start_websocket_server(state.clone(), address.clone());
        started.await.expect("Server can start");

        let message = SamMessage::builder()
            .r#type(SamMessageType::PlaintextContent.into())
            .destination_account_id(bob_id.into())
            .destination_device_id(bob_device.into())
            .content("hi bob<3".into())
            .build();
        let sync_message = SamMessage::builder()
            .r#type(SamMessageType::PlaintextContent.into())
            .destination_account_id(alice_id.into())
            .destination_device_id(27)
            .content("hi bob<3".into())
            .build();
        let messages = vec![message, sync_message];

        let envelope = ClientEnvelope::builder().messages(messages).build();

        let msg_id = MessageId::generate();
        let msg = ClientMessage::builder()
            .id(msg_id.into())
            .message(envelope)
            .r#type(ClientMessageType::ClientMessage.into())
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
        assert!(alice_sent.is_ok(), "Alice timed out");
        assert!(alice_received.is_ok(), "Alice receive time out");
        assert!(
            alice_sent.is_ok_and(|res| res.is_ok()),
            "Alice could not send"
        );

        let ws_msg = alice_received
            .unwrap()
            .expect("Alices connection is open")
            .expect("Alice receives message");
        assert!(ws_msg.is_binary());

        let server_msg =
            ServerMessage::decode(ws_msg.into_data()).expect("Server sends wellformed data");

        println!("MESSAGE: {:?}", server_msg);

        assert_eq!(server_msg.r#type(), ServerMessageType::ServerAck);
    }
}
