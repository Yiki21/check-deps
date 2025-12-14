use std::{net::SocketAddr, time::Duration};

use axum::{Router, extract::Request};
use reqwest::StatusCode;
use tokio::net::TcpListener;
use tower_http::{normalize_path::NormalizePathLayer, timeout::TimeoutLayer};

use crate::{app::AppState, common::latency::LatencyResponse, config::server::ServerConfig};

pub struct Server {
    config: &'static ServerConfig,
}

impl Server {
    pub fn new(config: &'static ServerConfig) -> Self {
        Self { config }
    }

    pub async fn start(&self, state: AppState, router: Router<AppState>) -> anyhow::Result<()> {
        let router = self.build_router(state, router);
        let port = self.config.port();

        let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;

        tracing::info!("Listening on {}", listener.local_addr()?);


        let server = axum::serve(
            listener,
            router.into_make_service_with_connect_info::<SocketAddr>()
        )
        .await?;

        Ok(server)
    }

    fn build_router(&self, state: AppState, router: Router<AppState>) -> Router {
        let tracing_layer = tower_http::trace::TraceLayer::new_for_http()
            .make_span_with(|request: &Request| {
                let method = request.method();
                let uri = request.uri();
                let id = xid::new();

                tracing::info_span!("Http Request", id = %id ,method = %method, uri = %uri)
            })
            .on_request(())
            .on_failure(())
            .on_response(LatencyResponse);

        let timeout_layer = {
            let timeout = Duration::from_secs(self.config.timeout_seconds());
            TimeoutLayer::with_status_code(StatusCode::REQUEST_TIMEOUT, timeout)
        };

        let body_limit_layer =
            tower_http::limit::RequestBodyLimitLayer::new(self.config.max_body_size_bytes());

        let cors_layer = tower_http::cors::CorsLayer::new()
            .allow_origin(tower_http::cors::Any)
            .allow_methods(tower_http::cors::Any)
            .allow_headers(tower_http::cors::Any)
            .allow_credentials(false)
            .max_age(Duration::from_secs(86400)); // 1 day

        let normalize_path_layer = NormalizePathLayer::trim_trailing_slash();

        router
            .layer(timeout_layer)
            .layer(body_limit_layer)
            .layer(tracing_layer)
            .layer(cors_layer)
            .layer(normalize_path_layer)
            .with_state(state)
    }
}
