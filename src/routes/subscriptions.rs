use crate::{errors, startup::AppState};
use anyhow::Result;
use axum::{extract::State, http::StatusCode, response::IntoResponse};
use chrono::Utc;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct CreateSubscriptionForm {
    name: String,
    email: String,
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form,app_state),
    fields(
        subscriber_email = %form.email,
        subscriber_name= %form.name
    )
)]
pub async fn subscribe(
    State(app_state): State<AppState>,
    errors::Form(form): errors::Form<CreateSubscriptionForm>,
) -> impl IntoResponse {
    match insert_subscriber(&form, &app_state.db_pool.as_ref()).await {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            tracing::error!("Failed to execute query: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

#[tracing::instrument(name = "Saving new subscriber in database", skip(form, pool))]
pub async fn insert_subscriber(form: &CreateSubscriptionForm, pool: &PgPool) -> Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {}", e);
        e
    })?;
    Ok(())
}
