use std::cmp::max;

use crate::config;
use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseConnection, Statement};

pub async fn init() -> anyhow::Result<DatabaseConnection> {
    let config = config::get().database();
    let mut options = ConnectOptions::new(format!(
        "postgres://{}:{}@{}:{}/{}",
        config.username(),
        config.password(),
        config.host(),
        config.port(),
        config.database()
    ));
    options
        .min_connections(max(num_cpus::get() as u32 * 4, 10))
        .max_connections(max(num_cpus::get() as u32 * 8, 20))
        .connect_timeout(std::time::Duration::from_secs(8))
        .acquire_timeout(std::time::Duration::from_secs(30))
        .idle_timeout(std::time::Duration::from_secs(300))
        .max_lifetime(std::time::Duration::from_secs(3600))
        .sqlx_logging(true)
        .set_schema_search_path(config.schema());

    let db = Database::connect(options).await?;
    db.ping().await?;

    tracing::info!("Database connected successfully");

    log_db_version(&db).await?;

    Ok(db)
}

async fn log_db_version(db: &DatabaseConnection) -> anyhow::Result<()> {
    let version = db
        .query_one_raw(Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            "SELECT version()".to_owned(),
        ))
        .await?
        .ok_or_else(|| anyhow::anyhow!("Could not retrieve database version"))?;

    let version_str: String = version.try_get_by_index::<String>(0)?;
    tracing::info!("Database version: {}", version_str);

    Ok(())
}
