use bytesize::ByteSize;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub port: Option<u16>,
    pub timeout_seconds: Option<u64>,
    pub max_body_size_bytes: Option<usize>,
}

impl ServerConfig {
    pub fn port(&self) -> u16 {
        self.port.unwrap_or(8080)
    }

    pub fn timeout_seconds(&self) -> u64 {
        self.timeout_seconds.unwrap_or(120)
    }

    pub fn max_body_size_bytes(&self) -> usize {
        self.max_body_size_bytes
            .unwrap_or(ByteSize::mib(10).as_u64() as usize) // 10 MB
    }
}
