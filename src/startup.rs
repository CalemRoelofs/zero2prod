use crate::{configuration::Settings, routes};
use anyhow::{Context, Result};
use axum::{
    routing::{get, post, IntoMakeService},
    Router, Server,
};
use sqlx::PgPool;
use std::{net::SocketAddr, sync::Arc};

pub fn new_app(state: AppState) -> IntoMakeService<Router> {
    let router = Router::new()
        .route("/health_check", get(routes::health_check))
        .route("/subscriptions", post(routes::subscribe))
        .with_state(state);
    router.into_make_service()
}

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Settings>,
    pub db_pool: Arc<PgPool>,
}

pub async fn run(config: Settings, db_pool: PgPool) -> Result<()> {
    let state = AppState {
        config: Arc::new(config.clone()),
        db_pool: Arc::new(db_pool),
    };

    let app = new_app(state);
    tracing_subscriber::fmt::init();

    let addr = SocketAddr::from(([127, 0, 0, 1], config.clone().application_port));
    tracing::debug!("listening on {}", addr);

    Server::bind(&addr)
        .serve(app)
        .await
        .with_context(|| "Failed to start server!")
}
