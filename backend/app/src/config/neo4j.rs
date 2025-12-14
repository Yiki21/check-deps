use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Neo4jConfig {
    pub enabled: Option<bool>,
    pub uri: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub database: Option<String>,
}

impl Neo4jConfig {
    pub fn enabled(&self) -> bool {
        self.enabled.unwrap_or(false)
    }

    pub fn uri(&self) -> Option<&str> {
        self.uri.as_deref()
    }

    pub fn username(&self) -> Option<&str> {
        self.username.as_deref()
    }

    pub fn password(&self) -> Option<&str> {
        self.password.as_deref()
    }

    pub fn database(&self) -> Option<&str> {
        self.database.as_deref()
    }
}
