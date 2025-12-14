use axum::Router;

use crate::common::ApiError;

pub mod packages;
pub mod projects;

pub fn create_router() -> Router<crate::app::AppState> {
    Router::new()
        .nest("/api", 
            Router::new().
            nest("/package", packages::routes()).
            nest("/projects", projects::routes()))
        .fallback(async || -> ApiError {
            tracing::info!("Not Found!");
            ApiError::NotFound
        })
        .method_not_allowed_fallback(async || -> ApiError {
            tracing::info!("Method Not Allowed!");
            ApiError::MethodNotAllowed
        })
}
