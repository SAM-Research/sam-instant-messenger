use sam_common::sam_message::{ClientMessage, ServerEnvelope, ServerMessage};

use crate::{
    state::{state_type::StateType, ServerState},
    ServerError,
};

pub async fn handle_client_message<T: StateType>(
    state: &mut ServerState<T>,
    message: ClientMessage,
) -> Result<ServerMessage, ServerError> {
    todo!()
}

pub async fn handle_server_envelope<T: StateType>(
    state: &mut ServerState<T>,
    envelope: ServerEnvelope,
) -> Result<ServerMessage, ServerError> {
    todo!()
}
