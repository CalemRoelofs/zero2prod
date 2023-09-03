use axum::http::StatusCode;
use tracing::log;

pub async fn health_check() -> StatusCode {
    log::info!("I am alive!");
    StatusCode::OK
}
