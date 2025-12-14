use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct LoggerConfig {
    level: Option<String>,
}

impl LoggerConfig {
    pub fn level(&self) -> Option<&str> {
        self.level.as_deref()
    }
}

impl Default for LoggerConfig {
    fn default() -> Self {
        LoggerConfig { level: None }
    }
}