use crate::{errors, startup::AppState};
use axum::{extract::State, http::StatusCode, response::IntoResponse};
use chrono::Utc;
use serde::Deserialize;
use tracing::log;
use uuid::Uuid;

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct CreateSubscriptionForm {
    name: String,
    email: String,
}

pub async fn subscribe(
    State(app_state): State<AppState>,
    errors::Form(form): errors::Form<CreateSubscriptionForm>,
) -> impl IntoResponse {
    log::info!(
        "Attempting to add '{}' '{}' as a new subscriber",
        form.email,
        form.name
    );
    match sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(app_state.db_pool.as_ref())
    .await
    {
        Ok(_) => {
            log::info!("New subscriber saved to database");
            StatusCode::OK
        }
        Err(e) => {
            log::error!("Failed to execute query: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
