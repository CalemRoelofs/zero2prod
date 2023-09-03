use axum::{body::Body, http::Request, Router};
use tower_http::trace::{self, TraceLayer};
use tower_request_id::{RequestId, RequestIdLayer};
use tracing::{info_span, Level};

pub fn add_request_id(router: Router) -> Router {
    router
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<Body>| {
                    let request_id = request
                        .extensions()
                        .get::<RequestId>()
                        .map(ToString::to_string)
                        .unwrap_or_else(|| "unknown".into());
                    info_span!(
                        "request",
                        id = %request_id,
                        method = %request.method(),
                        uri = %request.uri(),
                    )
                })
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        )
        .layer(RequestIdLayer)
}
