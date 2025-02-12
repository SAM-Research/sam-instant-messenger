use sam_common::sam_message::{ClientMessage, ServerMessage};
use uuid::Uuid;

use crate::state::{traits::state_type::StateType, ServerState};

pub async fn receive_client_message<T: StateType>(
    state: &ServerState<T>,
    account_id: &Uuid,
    device_id: u32,
    msg: ClientMessage,
) -> ServerMessage {
    todo!()
}
