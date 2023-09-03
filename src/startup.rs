use crate::{configuration::Settings, routes};
use anyhow::{Context, Result};
use axum::{
    body::Body,
    http::Request,
    routing::{get, post},
    Router, Server,
};
use sqlx::PgPool;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::trace::{self, TraceLayer};
use tower_request_id::{RequestId, RequestIdLayer};
use tracing::{error_span, Level};

pub fn new_app(state: AppState) -> Router {
    let router = Router::new()
        .route("/health_check", get(routes::health_check))
        .route("/subscriptions", post(routes::subscribe))
        .with_state(state);
    router
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

    tracing_subscriber::fmt().compact().init();

    let app = new_app(state)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<Body>| {
                    // We get the request id from the extensions
                    let request_id = request
                        .extensions()
                        .get::<RequestId>()
                        .map(ToString::to_string)
                        .unwrap_or_else(|| "unknown".into());
                    // And then we put it along with other information into the `request` span
                    error_span!(
                        "request",
                        id = %request_id,
                        method = %request.method(),
                        uri = %request.uri(),
                    )
                })
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        )
        .layer(RequestIdLayer)
        .into_make_service();

    let addr = SocketAddr::from(([127, 0, 0, 1], config.clone().application_port));
    tracing::debug!("listening on {}", addr);

    Server::bind(&addr)
        .serve(app)
        .await
        .with_context(|| "Failed to start server!")
}
