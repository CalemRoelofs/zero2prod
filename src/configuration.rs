use anyhow::{Context, Error};
use config::{Config, File};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application_port: u16,
}

#[derive(Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database_name
        )
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
