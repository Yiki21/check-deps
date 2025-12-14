pub mod auth;
pub mod database;
pub mod jwt;
pub mod logger;
pub mod server;
pub mod s3;
pub mod neo4j;
pub mod languages;

pub(crate) use std::sync::LazyLock;

use anyhow::Context;
use config::Config;
use serde::Deserialize;

pub use database::DatabaseConfig;
pub use jwt::JwtConfig;
pub use s3::S3Config;
pub use server::ServerConfig;
pub use neo4j::Neo4jConfig;
pub use languages::LanguagesConfig;

use crate::config::{auth::AuthConfig, logger::LoggerConfig};

static APPCONFIG: LazyLock<AppConfig> =
    LazyLock::new(|| AppConfig::load().expect("Failed to load application configuration"));

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    server: ServerConfig,
    database: DatabaseConfig,
    #[serde(default)]
    jwt: JwtConfig,
    #[serde(default)]
    auth: AuthConfig,
    #[serde(default)]
    s3: S3Config,
    neo4j: Neo4jConfig,
    #[serde(default)]
    languages: LanguagesConfig,
    #[serde(default)]
    logger: LoggerConfig,
}

impl AppConfig {
    pub fn load() -> anyhow::Result<Self> {
        let config = Config::builder()
            .add_source(
                config::File::with_name("application")
                    .format(config::FileFormat::Yaml)
                    .required(true),
            )
            .add_source(
                config::Environment::with_prefix("APP")
                    .try_parsing(true)
                    .separator("_"),
            )
            .build()
            .with_context(|| "Failed to read The Configuration")?
            .try_deserialize()
            .with_context(|| "Failed to deserialize The Configuration");

        config
    }

    pub fn server(&self) -> &ServerConfig {
        &self.server
    }

    pub fn database(&self) -> &DatabaseConfig {
        &self.database
    }

    pub fn jwt(&self) -> &JwtConfig {
        &self.jwt
    }

    pub fn auth(&self) -> &AuthConfig {
        &self.auth
    }

    pub fn s3(&self) -> &S3Config {
        &self.s3
    }

    pub fn neo4j(&self) -> &Neo4jConfig {
        &self.neo4j
    }

    pub fn languages(&self) -> &LanguagesConfig {
        &self.languages
    }

    pub fn logger(&self) -> &LoggerConfig {
        &self.logger
    }
}

pub fn get() -> &'static AppConfig {
    &APPCONFIG
}
