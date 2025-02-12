use axum::extract::ws::Message;
use futures_util::stream::{SplitSink, SplitStream};

use uuid::Uuid;

use crate::{
    auth::authenticated_user::AuthenticatedUser,
    state::{state_type::StateType, ServerState},
};

pub async fn create_websocket<T: StateType>(
    _state: ServerState<T>,
    _auth_user: AuthenticatedUser,
    _tx: SplitSink<T::Socket, Message>,
    _rx: SplitStream<T::Socket>,
) {
    todo!()
}

pub async fn websocket_faucet<T: StateType>(
    _state: ServerState<T>,
    _account_id: Uuid,
    _device_id: u32,
    _rx: SplitStream<T::Socket>,
) {
    todo!()
}

pub async fn websocket_sink<T: StateType>(
    _state: ServerState<T>,
    _account_id: Uuid,
    _device_id: u32,
    _tx: SplitSink<T::Socket, Message>,
) {
    todo!()
}
