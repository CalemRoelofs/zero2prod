use crate::errors;
use axum::{http::StatusCode, response::IntoResponse};
use serde::Deserialize;

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct CreateSubscriptionForm {
    name: String,
    email: String,
}

pub async fn subscribe(
    errors::Form(_payload): errors::Form<CreateSubscriptionForm>,
) -> impl IntoResponse {
    StatusCode::OK
}
