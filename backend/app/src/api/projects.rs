use axum::{routing::{get, post}, Router};
pub mod register;

use crate::{app::AppState, common::{error::ApiError, response::ApiResponse}};
use register::register_project;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_projects))
        .route("/register", post(register_project))
}

async fn list_projects() -> Result<ApiResponse<&'static str>, ApiError> {
    Ok(ApiResponse::ok("project list", Some("stub")))
}
