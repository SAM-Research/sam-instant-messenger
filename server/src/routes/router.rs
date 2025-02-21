use axum::Router;

use crate::state::{state_type::StateType, ServerState};

use super::{
    account::account_routes, device::device_routes, keys::key_routes, websocket::websocket_routes,
};

type SAMRouter<T> = Router<ServerState<T>>;

struct RouterBuilder<T: StateType> {
    routes: Vec<fn(SAMRouter<T>) -> SAMRouter<T>>,
}

impl<T: StateType> RouterBuilder<T> {
    fn add_routes(mut self, f: fn(Router<ServerState<T>>) -> Router<ServerState<T>>) -> Self {
        self.routes.push(f);
        self
    }
    fn new() -> Self {
        Self { routes: Vec::new() }
    }

    fn build(self) -> Router<ServerState<T>> {
        let mut router = Router::new();
        for f in self.routes {
            router = f(router)
        }
        router
    }
}

pub fn router<T: StateType>() -> Router<ServerState<T>> {
    RouterBuilder::new()
        .add_routes(account_routes)
        .add_routes(key_routes)
        .add_routes(device_routes)
        .add_routes(websocket_routes)
        .build()
}
