pub mod database;
pub mod logger;
pub mod server;
use std::sync::LazyLock;

use anyhow::Context;
use config::Config;
use serde::Deserialize;
use server::ServerConfig;

use crate::config::database::DatabaseConfig;

static APPCONFIG: LazyLock<AppConfig> =
    LazyLock::new(|| AppConfig::load().expect("Failed to load application configuration"));

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
}

impl AppConfig {
    pub fn load() -> anyhow::Result<Self> {
        Config::builder()
            .add_source(
                config::File::with_name("application")
                    .format(config::FileFormat::Yaml)
                    .required(true),
            )
            .add_source(
                config::Environment::with_prefix("APP")
                    .try_parsing(true)
                    .separator("_")
                    .list_separator(","),
            )
            .build()
            .with_context(|| "Failed to read The Configuration")?
            .try_deserialize()
            .with_context(|| "Failed to deserialize The Configuration")
    }

    pub fn server(&self) -> &ServerConfig {
        &self.server
    }

    pub fn database(&self) -> &DatabaseConfig {
        &self.database
    }
}

pub fn get() -> &'static AppConfig {
    &APPCONFIG
}
