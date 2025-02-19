use axum::{
    extract::{State, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use futures_util::StreamExt;
use log::info;
use tokio::sync::mpsc;

use crate::{
    auth::authenticated_user::AuthenticatedUser,
    logic::websocket::{
        websocket_dispatcher, websocket_message_receiver, websocket_message_sender,
    },
    managers::traits::message_manager::MessageManager,
    state::{state_type::StateType, ServerState},
    ServerError,
};

async fn websocket_endpoint<T: StateType>(
    State(mut state): State<ServerState<T>>,
    auth_user: AuthenticatedUser,
    ws: WebSocketUpgrade,
) -> Result<impl IntoResponse, ServerError> {
    // TODO: find a solution to encapsulate this in a logic function
    let dispatch = state
        .messages
        .subscribe(*auth_user.account().id(), auth_user.device().id())
        .await?;

    Ok(ws.on_upgrade(move |socket| async move {
        info!("{} Connected!", auth_user.account().username());

        let (sender, receiver) = socket.split();
        let (msg_producer, msg_consumer) = mpsc::channel(5);

        tokio::spawn(websocket_message_receiver(
            state.clone(),
            receiver,
            msg_producer.clone(),
            auth_user.clone(),
        ));
        tokio::spawn(websocket_dispatcher(
            state.clone(),
            dispatch,
            msg_producer,
            auth_user.clone(),
        ));

        tokio::spawn(websocket_message_sender(
            state,
            sender,
            msg_consumer,
            auth_user,
        ));
    }))
}

pub fn websocket_routes<T: StateType>(router: Router<ServerState<T>>) -> Router<ServerState<T>> {
    router.route("/api/v1/websocket", get(websocket_endpoint))
}

#[cfg(test)]
mod test {
    use std::{net::SocketAddr, time::Duration};

    use axum::Router;
    use base64::{prelude::BASE64_STANDARD, Engine};
    use futures_util::{SinkExt, StreamExt};
    use maplit::hashmap;
    use prost::Message;
    use rand::rngs::OsRng;
    use sam_common::sam_message::{ClientEnvelope, ClientMessage, EnvelopeType, MessageType};
    use tokio::sync::oneshot;
    use tokio_tungstenite::{
        connect_async, tungstenite::client::IntoClientRequest, MaybeTlsStream, WebSocketStream,
    };
    use uuid::Uuid;

    use crate::{
        managers::in_memory::test_utils::LINK_SECRET,
        routes::{test_utils::create_user, websocket::websocket_routes},
        state::{state_type::StateType, ServerState},
    };
    use tokio::sync::oneshot::Receiver;

    fn start_websocket_server<T: StateType>(
        state: ServerState<T>,
        address: String,
    ) -> Receiver<()> {
        let app = websocket_routes(Router::new()).with_state(state);
        let (tx, rx) = oneshot::channel::<()>();
        tokio::spawn(async move {
            let server = axum_server::bind(address.parse().expect("Can make socket addr from str"))
                .serve(app.into_make_service_with_connect_info::<SocketAddr>());
            tx.send(()).expect("Can oneshot");
            server.await
        });
        rx
    }

    async fn connect_user(
        account_id: Uuid,
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
        let (ws, _) = connect_async(req).await.expect("Can make connection");
        ws
    }

    #[tokio::test]
    async fn test_websocket_alice_send_to_bob() {
        let mut state = ServerState::in_memory_default(LINK_SECRET.to_owned());
        let (_, id) = create_user(&mut state, "alice", "phone", "bob", OsRng).await;
        let (_, bob_id) = create_user(&mut state, "bob", "laptop", "cheeseburger", OsRng).await;

        let address = "127.0.0.1:8888".to_string();
        start_websocket_server(state.clone(), address.clone())
            .await
            .expect("Server can start");

        let envelope = ClientEnvelope::builder()
            .destination(bob_id.into())
            .source(id.into())
            .r#type(EnvelopeType::PlaintextContent as i32)
            .content(hashmap! {1u32 => "hi bob<3".into()})
            .build();

        let msg_id = Uuid::new_v4();
        let msg = ClientMessage::builder()
            .id(msg_id.into())
            .message(envelope)
            .r#type(MessageType::Message as i32)
            .build();

        let mut alice = connect_user(id, "bob", &address).await;
        let mut bob = connect_user(bob_id, "cheeseburger", &address).await;

        let alice_send = tokio::time::timeout(
            Duration::from_millis(300),
            alice.send(tokio_tungstenite::tungstenite::Message::Binary(
                msg.encode_to_vec().into(),
            )),
        );

        let bob_recv = tokio::time::timeout(Duration::from_millis(300), bob.next());

        let alice_sent = alice_send.await;
        assert!(alice_sent.is_ok(), "Alice timed out");
        assert!(
            alice_sent.is_ok_and(|res| res.is_ok()),
            "Alice could not send"
        );

        let bob_received = bob_recv.await;
        assert!(bob_received.is_ok(), "Bob timed out");
        assert!(
            bob_received.is_ok_and(|op| op.is_some_and(|res| res.is_ok())),
            "Bob could not received"
        )
    }
}
