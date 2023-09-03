use crate::{configuration::Settings, middleware, routes, telemetry};
use anyhow::{Context, Result};
use axum::{
    routing::{get, post},
    Router, Server,
};
use sqlx::PgPool;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::util::SubscriberInitExt;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Settings>,
    pub db_pool: Arc<PgPool>,
}

pub fn new_app(state: AppState) -> Router {
    Router::new()
        .route("/health_check", get(routes::health_check))
        .route("/subscriptions", post(routes::subscribe))
        .with_state(state)
}

pub async fn run(config: Settings, db_pool: PgPool) -> Result<()> {
    let state = AppState {
        config: Arc::new(config.clone()),
        db_pool: Arc::new(db_pool),
    };

    let subscriber =
        telemetry::get_tracing_subscriber("zero2prod".into(), "info".into(), std::io::stdout)?;
    subscriber.try_init()?;

    let router = new_app(state);
    let app = middleware::add_request_id(router).into_make_service();

    let addr = SocketAddr::from(([127, 0, 0, 1], config.clone().application_port));
    tracing::info!("Server starting on {}", addr);

    Server::bind(&addr)
        .serve(app)
        .await
        .with_context(|| "Failed to start server")
}
