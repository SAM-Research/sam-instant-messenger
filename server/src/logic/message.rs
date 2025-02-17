use sam_common::sam_message::{ClientMessage, ServerMessage};

use crate::{
    state::{state_type::StateType, ServerState},
    ServerError,
};

pub async fn handle_client_message<T: StateType>(
    state: ServerState<T>,
    message: ClientMessage,
) -> Result<ServerMessage, ServerError> {
    todo!()
}
