use crate::{configuration, routes};
use anyhow::{Context, Result};
use axum::{
    routing::{get, post, IntoMakeService},
    Router, Server,
};
use std::net::SocketAddr;

pub fn new_app() -> IntoMakeService<Router> {
    let router = Router::new()
        .route("/health_check", get(routes::health_check))
        .route("/subscriptions", post(routes::subscribe));
    router.into_make_service()
}

pub async fn run() -> Result<()> {
    let config = match configuration::get_configuration() {
        Ok(config) => config,
        Err(e) => panic!("{}", e),
    };

    let app = new_app();
    tracing_subscriber::fmt::init();

    let addr = SocketAddr::from(([127, 0, 0, 1], config.application_port));
    tracing::debug!("listening on {}", addr);

    Server::bind(&addr)
        .serve(app)
        .await
        .with_context(|| "Failed to start server!")
}
