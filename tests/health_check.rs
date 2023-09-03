use axum_test::{TestServer, TestServerConfig};
use zero2prod::startup::new_app;

#[cfg(test)]
fn new_test_app() -> TestServer {
    let app = crate::new_app();

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
    .unwrap()
}

#[cfg(test)]
mod health_check_endpoint {
    use crate::new_test_app;

    #[tokio::test]
    async fn health_check_works() {
        // Given
        let server = new_test_app();

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
    use sqlx::{Connection, PgConnection};

    use crate::new_test_app;

    #[derive(Serialize)]
    struct SubscriberForm {
        name: String,
        email: String,
    }

    #[tokio::test]
    async fn subscribe_returns_a_200_for_valid_form_data() {
        // Given
        let server = new_test_app();

        let config =
            zero2prod::configuration::get_configuration().expect("Failed to load configuration!");
        let conn_string = config.database.connection_string();
        let mut conn = PgConnection::connect(&conn_string)
            .await
            .expect("Failed to connect to Postgres!");

        // When
        let body = &SubscriberForm {
            name: "Le Guin".to_string(),
            email: "ursula_le_guin@gmail.com".to_string(),
        };
        let response = server
            .post(&"/subscriptions")
            .form::<SubscriberForm>(body)
            .await;

        // Then
        assert!(response.status_code().is_success());

        let saved = sqlx::query!("SELECT email, name FROM subscriptions")
            .fetch_one(&mut conn)
            .await
            .expect("Failed to fetch saved subscription");

        assert_eq!(saved.email, "ursula_le_guin@gmail.com");
        assert_eq!(saved.name, "Le Guin");
    }

    #[derive(Serialize)]
    struct Name {
        name: String,
    }

    #[tokio::test]
    async fn subscribe_returns_a_400_for_missing_form_data() {
        // Given
        let server = new_test_app();

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
