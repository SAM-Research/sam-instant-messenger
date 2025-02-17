use crate::routes::router;
use crate::state::state_type::StateType;
use crate::state::ServerState;
use axum::extract::Request;
use axum::middleware::{from_fn, Next};
use axum::response::IntoResponse;
use axum_server::tls_rustls::RustlsConfig;
use log::info;
use std::net::SocketAddr;

pub struct ServerConfig<T: StateType> {
    pub state: ServerState<T>,
    pub addr: SocketAddr,
    pub tls: Option<RustlsConfig>,
}

async fn log_request(req: Request, next: Next) -> impl IntoResponse {
    let method = req.method().clone();
    let path = req.uri().path().to_string();

    info!("{} '{}'", method, path);
    // Call the next handler in the chain
    next.run(req).await
}

pub async fn start_server<T: StateType>(config: ServerConfig<T>) -> Result<(), std::io::Error> {
    let mut state = config.state;
    state.init().await;
    let app = router()
        .layer(from_fn(log_request))
        .with_state(state.clone());

    info!(
        "Starting SAM Server on http{}://{}...",
        if config.tls.is_some() { "s" } else { "" },
        config.addr
    );
    if let Some(tls_config) = config.tls {
        rustls::crypto::ring::default_provider()
            .install_default()
            .expect("Failed to install rustls crypto provider");
        axum_server::bind_rustls(config.addr, tls_config)
            .serve(app.into_make_service_with_connect_info::<SocketAddr>())
            .await?;
    } else {
        axum_server::bind(config.addr)
            .serve(app.into_make_service_with_connect_info::<SocketAddr>())
            .await?;
    };

    state.cleanup().await;
    Ok(())
}
