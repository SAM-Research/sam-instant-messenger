use crate::routes::router;
use crate::state::state_type::StateType;
use crate::state::ServerState;
use axum::extract::Request;
use axum::middleware::{from_fn, Next};
use axum::response::IntoResponse;
use axum_server::tls_rustls::RustlsConfig;
use log::info;
use std::net::SocketAddr;
use std::sync::Arc;

pub struct ServerConfig<T: StateType> {
    pub state: ServerState<T>,
    pub addr: SocketAddr,
    pub tls_config: Option<Arc<rustls::ServerConfig>>,
}

async fn log_request(req: Request, next: Next) -> impl IntoResponse {
    let method = req.method().clone();
    let path = req.uri().path().to_string();

    info!("{} '{}'", method, path);
    next.run(req).await
}

pub async fn start_server<T: StateType>(config: ServerConfig<T>) -> Result<(), std::io::Error> {
    let state = config.state;

    let app = router()
        .layer(from_fn(log_request))
        .with_state(state.clone());

    info!(
        "Starting SAM Server on http{}://{}...",
        if config.tls_config.is_some() { "s" } else { "" },
        config.addr
    );
    if let Some(tls_config) = config.tls_config {
        let axum_tls_config = RustlsConfig::from_config(tls_config);
        axum_server::bind_rustls(config.addr, axum_tls_config)
            .serve(app.into_make_service_with_connect_info::<SocketAddr>())
            .await?;
    } else {
        axum_server::bind(config.addr)
            .serve(app.into_make_service_with_connect_info::<SocketAddr>())
            .await?;
    };

    Ok(())
}
