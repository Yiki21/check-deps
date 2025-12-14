use std::time::Duration;

use jsonwebtoken::Algorithm;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, Default)]
pub struct JwtConfig {
    secret: Option<String>,
    expiration_seconds: Option<u64>,
    audience: Option<String>,
    issuer: Option<String>,
    algorithm: Option<String>,
}

impl JwtConfig {
    pub fn secret(&self) -> &str {
        self.secret.as_deref().unwrap_or("default_secret_key")
    }

    pub fn expiration(&self) -> Duration {
        Duration::from_secs(self.expiration_seconds.unwrap_or(3600)) // Default to 1 hour
    }

    pub fn audience(&self) -> &str {
        self.audience.as_deref().unwrap_or("my_audience")
    }

    pub fn issuer(&self) -> &str {
        self.issuer.as_deref().unwrap_or("my_issuer")
    }

    pub fn algorithm(&self) -> Algorithm {
        match self.algorithm.as_deref().unwrap_or("HS256") {
            "HS256" => Algorithm::HS256,
            "HS384" => Algorithm::HS384,
            "HS512" => Algorithm::HS512,
            _ => Algorithm::HS256, // Default to HS256
        }
    }
}
