use axum::{
    routing::{delete, get, post, put},
    Router,
};

use crate::state::ServerState;

use super::account::{account_register_endpoint, delete_account_endpoint};
use super::device::{delete_device_endpoint, link_device_endpoint, provision_device_endpoint};
use super::keys::{keys_bundles_endpoint, publish_keys_endpoint};
use super::websocket::websocket_endpoint;

pub fn router() -> Router<ServerState> {
    Router::new()
        .route("/", get(|| async { "Hello From SAM Service" }))
        .route("/api/v1/account", post(account_register_endpoint))
        .route("/api/v1/account", delete(delete_account_endpoint))
        .route("/api/v1/keys/:account_id", get(keys_bundles_endpoint))
        .route("/api/v1/keys", put(publish_keys_endpoint))
        .route("/api/v1/devices/provision", get(provision_device_endpoint))
        .route("/api/v1/devices/link", post(link_device_endpoint))
        .route("/api/v1/device/:id", delete(delete_device_endpoint))
        .route("/api/v1/websocket", get(websocket_endpoint))
}
