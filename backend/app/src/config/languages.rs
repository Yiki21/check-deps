use std::collections::HashMap;

use serde::Deserialize;

pub const DEFAULT_SBOM_TIMEOUT_SECONDS: u64 = 120;

#[derive(Debug, Deserialize, Clone, Default)]
pub struct LanguageServiceConfig {
    enabled: Option<bool>,
    cdxgen_url: Option<String>,
}

impl LanguageServiceConfig {
    pub fn enabled(&self) -> bool {
        self.enabled.unwrap_or(false)
    }

    pub fn cdxgen_url(&self) -> Option<&str> {
        self.cdxgen_url.as_deref()
    }
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct LanguagesConfig {
    #[serde(default)]
    timeout_seconds: Option<u64>,
    #[serde(default)]
    #[serde(flatten)]
    entries: HashMap<String, LanguageServiceConfig>,
}

impl LanguagesConfig {
    pub fn get(&self, key: &str) -> Option<&LanguageServiceConfig> {
        self.entries.get(key).or_else(|| {
            let lower = key.to_ascii_lowercase();
            if lower == key {
                None
            } else {
                self.entries.get(&lower)
            }
        })
    }

    pub fn resolve_base_url<'a>(&'a self, candidates: &[&str]) -> Option<&'a str> {
        if self.entries.is_empty() {
            return None;
        }

        for key in candidates {
            if let Some(cfg) = self.get(key) {
                if cfg.enabled() {
                    if let Some(url) = cfg.cdxgen_url() {
                        return Some(url);
                    }
                }
            }
        }

        None
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn timeout_seconds(&self) -> u64 {
        self.timeout_seconds.unwrap_or(DEFAULT_SBOM_TIMEOUT_SECONDS)
    }
}
