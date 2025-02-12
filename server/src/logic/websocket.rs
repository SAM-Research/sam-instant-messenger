use axum::extract::ws::Message;
use futures_util::stream::{SplitSink, SplitStream};

use uuid::Uuid;

use crate::{
    auth::authenticated_user::AuthenticatedUser,
    state::{traits::state_type::StateType, ServerState},
};

pub async fn create_websocket<T: StateType>(
    state: ServerState<T>,
    auth_user: AuthenticatedUser,
    tx: SplitSink<T::Socket, Message>,
    rx: SplitStream<T::Socket>,
) {
    todo!()
}

pub async fn websocket_faucet<T: StateType>(
    state: ServerState<T>,
    account_id: Uuid,
    device_id: u32,
    mut rx: SplitStream<T::Socket>,
) {
    todo!()
}

pub async fn websocket_sink<T: StateType>(
    state: ServerState<T>,
    account_id: Uuid,
    device_id: u32,
    tx: SplitSink<T::Socket, Message>,
) {
    todo!()
}
