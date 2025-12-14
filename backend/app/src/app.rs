use neo4rs::Graph;
use sea_orm::DatabaseConnection;

use crate::{api, database, id, logger, neo4j, server::Server};
use migration::{Migrator, MigratorTrait};

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub neo4j: Option<Graph>,
}

impl AppState {
    pub fn new(db: DatabaseConnection, neo4j: Option<Graph>) -> Self {
        Self { db, neo4j }
    }
}

pub async fn run() -> anyhow::Result<()> {
    let router = api::create_router();

    logger::init();

    id::init()?;

    tracing::info!("Starting application...");

    let db = database::init()
        .await
        .expect("Failed to initialize database");

    Migrator::up(&db, None).await?;

    let neo4j = neo4j::init().await?;

    let state = AppState::new(db, neo4j);

    let server = Server::new(crate::config::get().server());

    server.start(state, router).await?;

    Ok(())
}
