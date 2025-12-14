use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config;

pub fn init() {
    let log_config = config::get().logger();
    let level = log_config
        .level()
        .map(|lvl| lvl.to_string())
        .or_else(|| std::env::var("RUST_LOG").ok())
        .unwrap_or_else(|| "info".to_string());

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(level))
        .with(tracing_subscriber::fmt::layer())
        .init();
}
