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
    use futures_util::{SinkExt, StreamExt};

    use maplit::hashmap;
    use prost::Message;
    use rand::rngs::OsRng;
    use sam_common::{
        address::{AccountId, MessageId},
        sam_message::{ClientEnvelope, ClientMessage, EnvelopeType, MessageType},
    };

    use tokio::{sync::oneshot, task::JoinHandle};
    use tokio_tungstenite::{
        connect_async, tungstenite::client::IntoClientRequest, MaybeTlsStream, WebSocketStream,
    };

    use crate::{
        managers::in_memory::test_utils::LINK_SECRET,
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
        username: &str,
        password: &str,
        address: &str,
    ) -> WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>> {
        let mut req = format!("ws://{}/api/v1/websocket", address)
            .into_client_request()
            .expect("Can make url into ws upgrade req");
        let basic = format!(
            "Basic {}",
            BASE64_STANDARD.encode(format!("{}.1:{}", account_id, password))
        );
        req.headers_mut()
            .insert("Authorization", basic.parse().unwrap());
        let (ws, _) = connect_async(req)
            .await
            .inspect_err(|e| println!("{}", e))
            .expect(&format!("{} can make connection", username));
        ws
    }

    #[tokio::test]
    async fn test_websocket_alice_send_to_bob() {
        let mut state = ServerState::in_memory(LINK_SECRET.to_owned(), 10);
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

        let mut alice = connect_user(alice_id, "alice", "bob", &address).await;
        let mut bob = connect_user(bob_id, "bob", "cheeseburger", &address).await;

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

        assert!(bob_received.is_ok(), "Bob timed out");
        assert!(
            bob_received.is_ok_and(|op| op.is_some_and(|res| res.is_ok())),
            "Bob could not received"
        )
    }

    #[tokio::test]
    async fn test_websocket_alice_send_to_bob_offline() {
        let mut state = ServerState::in_memory(LINK_SECRET.to_owned(), 10);
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

        let mut alice = connect_user(alice_id, "alice", "bob", &address).await;

        let alice_send = tokio::time::timeout(
            Duration::from_millis(300),
            alice.send(tokio_tungstenite::tungstenite::Message::Binary(
                msg.encode_to_vec().into(),
            )),
        );
        let alice_sent = alice_send.await;

        // bob goes online to receive message
        let mut bob = connect_user(bob_id, "bob", "cheeseburger", &address).await;
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
}
