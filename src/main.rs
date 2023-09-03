use secrecy::ExposeSecret;
use sqlx::PgPool;
use zero2prod::configuration;
use zero2prod::startup::run;

#[tokio::main]
async fn main() {
    let config = match configuration::get_configuration() {
        Ok(config) => config,
        Err(e) => panic!("{}", e),
    };
    let db_pool = PgPool::connect(&config.database.connection_string().expose_secret())
        .await
        .expect("Failed to connect to postgres!");
    let _ = run(config, db_pool).await;
}
