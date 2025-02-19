use axum::Router;
use axum_test::TestServer;
use libsignal_protocol::IdentityKeyPair;
use rand::rngs::OsRng;
use uuid::Uuid;

use crate::{
    auth::password::Password,
    managers::{
        entities::{account::Account, device::Device},
        traits::{account_manager::AccountManager, device_manager::DeviceManager},
    },
    state::{state_type::StateType, ServerState},
};

pub fn test_server<T: StateType>(
    state: ServerState<T>,
    routes: fn(Router<ServerState<T>>) -> Router<ServerState<T>>,
) -> TestServer {
    TestServer::new(routes(Router::new()).with_state(state).into_make_service())
        .expect("Can make test server")
}

pub async fn create_user<T: StateType>(
    state: &mut ServerState<T>,
    username: &str,
    device_name: &str,
    password: &str,
    mut rng: OsRng,
) -> (IdentityKeyPair, Uuid) {
    let id_pair = IdentityKeyPair::generate(&mut rng);
    let account = Account::builder()
        .id(Uuid::new_v4())
        .identity(id_pair.identity_key().clone())
        .username(username.to_string())
        .build();
    let device = Device::builder()
        .creation(0)
        .id(1)
        .registration_id(1)
        .name(device_name.to_string())
        .password(Password::generate(password.to_string()).expect("Password can be generated"))
        .build();
    state
        .accounts
        .add_account(&account)
        .await
        .expect("Account can be added");
    state
        .devices
        .add_device(*account.id(), &device)
        .await
        .expect("Device can be added");
    (id_pair, account.id().clone())
}
