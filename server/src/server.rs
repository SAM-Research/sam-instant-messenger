use crate::routes::router;
use crate::state::state_type::StateType;
use crate::state::ServerState;
use axum::extract::Request;
use axum::middleware::{from_fn, Next};
use axum::response::IntoResponse;
use axum::Router;
use axum_server::tls_rustls::RustlsConfig;
use std::net::SocketAddr;
use utoipa::OpenApi;
use utoipa_swagger_ui::{Config, SwaggerUi};

pub struct ServerConfig<T: StateType> {
    pub state: ServerState<T>,
    pub addr: SocketAddr,
    pub tls: Option<RustlsConfig>,
}

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "SAM Instant Messaging API", description = "This is the SAM IM API documentation")
    )
)]
struct ApiDoc;

async fn log_request(req: Request, next: Next) -> impl IntoResponse {
    let method = req.method().clone();
    let path = req.uri().path().to_string();

    println!("{} '{}'", method, path);
    // Call the next handler in the chain
    next.run(req).await
}

fn swagger_ui() -> axum::Router {
    let config = Config::from("/api-docs/openapi.json"); // <-- let the Swagger UI know the openapi json location
    utoipa_swagger_ui::SwaggerUi::new("/swagger-ui")
        .url("/openapi.json", ApiDoc::openapi())
        .config(config)
        .into()
}

pub async fn start_server<T: StateType>(config: ServerConfig<T>) -> Result<(), std::io::Error> {
    let mut state = config.state;
    state.init().await;

    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("api-docs/openapi.json", ApiDoc::openapi()))
        .nest("/api", router(state.clone()))
        .layer(from_fn(log_request))
        .with_state(state.clone());

    println!(
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
