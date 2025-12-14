use axum::{routing::get, Router};

use crate::{app::AppState, common::error::ApiError, common::response::ApiResponse};

pub fn routes() -> Router<AppState> {
    Router::new().route("/packages", get(list_packages))
}

async fn list_packages() -> Result<ApiResponse<&'static str>, ApiError> {
    Ok(ApiResponse::ok("package list", Some("stub")))
}
