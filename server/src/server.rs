use crate::routes::router;
use crate::state::traits::state_type::StateType;
use crate::state::ServerState;
use axum_server::tls_rustls::RustlsConfig;
use std::net::SocketAddr;

pub struct ServerConfig<T: StateType> {
    pub state: ServerState<T>,
    pub addr: SocketAddr,
    pub tls: Option<RustlsConfig>,
}

pub async fn start_server<T: StateType>(config: ServerConfig<T>) -> Result<(), std::io::Error> {
    let mut state = config.state;
    state.init().await;
    let app = router().with_state(state.clone());

    if let Some(tls_config) = config.tls {
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
