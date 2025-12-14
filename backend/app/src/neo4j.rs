use anyhow::{anyhow, Context};
use neo4rs::{ConfigBuilder, Graph, query};

use crate::config;

pub async fn init() -> anyhow::Result<Option<Graph>> {
    let cfg = config::get().neo4j();

    if !cfg.enabled() {
        tracing::info!("Neo4j disabled; skipping graph initialization");
        return Ok(None);
    }

    let uri = cfg
        .uri()
        .ok_or_else(|| anyhow!("neo4j.uri is required when neo4j.enabled=true"))?;
    let username = cfg
        .username()
        .ok_or_else(|| anyhow!("neo4j.username is required when neo4j.enabled=true"))?;
    let password = cfg
        .password()
        .ok_or_else(|| anyhow!("neo4j.password is required when neo4j.enabled=true"))?;

    let mut builder = ConfigBuilder::new()
        .uri(uri)
        .user(username)
        .password(password);

    if let Some(db) = cfg.database() {
        builder = builder.db(db);
    }

    let config = builder
        .build()
        .context("failed to build Neo4j config")?;

    let graph = Graph::connect(config).await?;
    tracing::info!("Connected to Neo4j at {}", uri);

    tracing::debug!("userName={}, password ={}", username, password);

    let version = get_neo4j_version(&graph).await?;
    tracing::info!("Connected to Neo4j version {}", version);

    Ok(Some(graph))
}


async fn get_neo4j_version(graph: &Graph) -> anyhow::Result<String> {
    let query = query(
        "CALL dbms.components() YIELD name, versions, edition RETURN name, versions, edition"
    );

    let mut result = graph.execute(query).await?;

    if let Some(row) = result.next().await? {
        // row.get 返回 Result<T, DeError>
        let versions: Vec<String> = row.get("versions")?; 
        if let Some(version) = versions.first() {
            return Ok(version.clone());
        }
    }

    Err(anyhow!("Failed to retrieve Neo4j version"))
}
