use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct S3Config {
    pub enabled: Option<bool>,
    pub bucket: Option<String>,
    pub region: Option<String>,
    pub access_key_id: Option<String>,
    pub secret_access_key: Option<String>,
    pub endpoint: Option<String>,
    pub prefix: Option<String>,
}

impl S3Config {
    pub fn enabled(&self) -> bool {
        self.enabled.unwrap_or(false)
    }

    pub fn bucket(&self) -> Option<&str> {
        self.bucket.as_deref()
    }

    pub fn region(&self) -> Option<&str> {
        self.region.as_deref()
    }

    pub fn access_key_id(&self) -> Option<&str> {
        self.access_key_id.as_deref()
    }

    pub fn secret_access_key(&self) -> Option<&str> {
        self.secret_access_key.as_deref()
    }

    pub fn endpoint(&self) -> Option<&str> {
        self.endpoint.as_deref()
    }

    pub fn prefix(&self) -> Option<&str> {
        self.prefix.as_deref()
    }
}
