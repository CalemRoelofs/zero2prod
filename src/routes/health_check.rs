use axum::http::StatusCode;

pub async fn health_check() -> StatusCode {
    tracing::info_span!("Health check");
    StatusCode::OK
}
