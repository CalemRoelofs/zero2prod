use anyhow::{Context, Error};
use config::{Config, File};
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application_port: u16,
}

#[derive(Deserialize, Clone)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> Secret<String> {
        format!(
            "{}/{}",
            self.connection_string_without_db().expose_secret(),
            self.database_name
        )
        .into()
    }

    pub fn connection_string_without_db(&self) -> Secret<String> {
        format!(
            "postgres://{}:{}@{}:{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port
        )
        .into()
    }
}

pub fn get_configuration() -> Result<Settings, Error> {
    let config = Config::builder()
        .add_source(File::with_name("configuration"))
        .build()
        .with_context(|| "Failed to read config file 'configuration.toml'")?;
    config
        .try_deserialize::<Settings>()
        .with_context(|| "Failed to deserialize config into `Settings` struct!")
}
