use axum_test::{TestServer, TestServerConfig};
use once_cell::sync::Lazy;
use secrecy::ExposeSecret;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use zero2prod::{
    configuration::DatabaseSettings,
    middleware,
    startup::{new_app, AppState},
    telemetry,
};

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = telemetry::get_tracing_subscriber(
            subscriber_name,
            default_filter_level,
            std::io::stdout,
        );
        tracing::subscriber::set_global_default(subscriber.unwrap())
            .expect("Failed to initialise tracing subscriber");
    } else {
        let subscriber =
            telemetry::get_tracing_subscriber(subscriber_name, default_filter_level, std::io::sink)
                .unwrap();
        tracing::subscriber::set_global_default(subscriber)
            .expect("Failed to initialise tracing subscriber");
    };
});

#[cfg(test)]
async fn new_test_app() -> (TestServer, AppState) {
    use std::sync::Arc;
    use uuid::Uuid;

    Lazy::force(&TRACING);

    let mut config = match zero2prod::configuration::get_configuration() {
        Ok(config) => config,
        Err(e) => panic!("{}", e),
    };
    config.database.database_name = Uuid::new_v4().to_string();
    let db_pool = configure_database(&config.database).await;

    let state = zero2prod::startup::AppState {
        config: Arc::new(config),
        db_pool: Arc::new(db_pool),
    };

    let router = new_app(state.clone());
    let app = middleware::add_request_id(router).into_make_service();

    (
        TestServer::new_with_config(
            app,
            TestServerConfig {
                // Preserve cookies across requests
                // for the session cookie to work.
                save_cookies: true,
                expect_success_by_default: true,
                ..TestServerConfig::default()
            },
        )
        .unwrap(),
        state.clone(),
    )
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Create database
    let mut connection =
        PgConnection::connect(&config.connection_string_without_db().expose_secret())
            .await
            .expect("Failed to connect to Postgres");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database.");

    // Migrate database
    let connection_pool = PgPool::connect(&config.connection_string().expose_secret())
        .await
        .expect("Failed to connect to Postgres.");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}

#[cfg(test)]
mod health_check_endpoint {
    use crate::new_test_app;

    #[tokio::test]
    async fn health_check_works() {
        // Given
        let (server, _) = new_test_app().await;

        // When
        let response = server.get(&"/health_check").await;

        // Then
        assert!(response.status_code().is_success());
        response.assert_text(&"");
    }
}

#[cfg(test)]
mod subscriptions_endpoint {
    use axum::http::StatusCode;
    use serde::Serialize;

    use crate::new_test_app;

    #[derive(Serialize)]
    struct SubscriberForm {
        name: String,
        email: String,
    }

    #[tokio::test]
    async fn subscribe_returns_a_200_for_valid_form_data() {
        // Given
        let (server, state) = new_test_app().await;

        // When
        let body = &SubscriberForm {
            name: "Le Guin".to_string(),
            email: "ursula_le_guin+{}@gmail.com".to_string(),
        };
        let response = server
            .post(&"/subscriptions")
            .form::<SubscriberForm>(body)
            .await;

        // Then
        assert!(response.status_code().is_success());

        let saved = sqlx::query!("SELECT email, name FROM subscriptions")
            .fetch_one(state.db_pool.as_ref())
            .await
            .expect("Failed to fetch saved subscription");

        assert_eq!(saved.email, body.email);
        assert_eq!(saved.name, "Le Guin");
    }

    #[derive(Serialize)]
    struct Name {
        name: String,
    }

    #[tokio::test]
    async fn subscribe_returns_a_400_for_missing_form_data() {
        // Given
        let (server, _) = new_test_app().await;

        // When
        let body = &Name {
            name: "Le Guin".to_string(),
        };
        let response = server
            .post(&"/subscriptions")
            .expect_failure()
            .form::<Name>(body)
            .await;

        // Then
        assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
    }
}
