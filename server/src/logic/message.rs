use sam_common::sam_message::{ClientMessage, ServerMessage};
use uuid::Uuid;

use crate::state::{traits::state_type::StateType, ServerState};

pub async fn receive_client_message<T: StateType>(
    _state: &ServerState<T>,
    _account_id: &Uuid,
    _device_id: u32,
    _msg: ClientMessage,
) -> ServerMessage {
    todo!()
}
