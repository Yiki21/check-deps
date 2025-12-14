use axum::{
    extract::rejection::{PathRejection, QueryRejection},
    response::{IntoResponse, Response},
};
use axum_valid::ValidationRejection;

use crate::common::ApiResponse;

pub type ApiResult<T> = Result<T, ApiError>;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Not Found")]
    NotFound,

    #[error("{0}")]
    Biz(String),

    #[error("Database Error: {0}")]
    DataBase(#[from] sea_orm::DbErr),

    #[error("Internal Server Error: {0}")]
    InternalServerError(#[from] anyhow::Error),

    #[error("Bad Query Params: {0}")]
    Query(#[from] QueryRejection),

    #[error("Bad Path Params: {0}")]
    Path(#[from] PathRejection),

    #[error("Validation Error: {0}")]
    Validation(String),

    #[error("Bcrypt Error: {0}")]
    Bcrypt(#[from] bcrypt::BcryptError),

    #[error("JWT Error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error("Unauthorized {0}")]
    UnAuthorized(String),

    #[error("Method Not Allowed")]
    MethodNotAllowed,
}

impl From<axum_valid::ValidRejection<ApiError>> for ApiError {
    fn from(rejection: axum_valid::ValidRejection<ApiError>) -> Self {
        match rejection {
            ValidationRejection::Valid(error) => ApiError::Validation(error.to_string()),
            ValidationRejection::Inner(error) => error,
        }
    }
}

impl ApiError {
    pub fn status_code(&self) -> axum::http::StatusCode {
        match self {
            ApiError::NotFound => axum::http::StatusCode::NOT_FOUND,
            ApiError::Biz(_) => axum::http::StatusCode::OK,
            ApiError::InternalServerError(_) | ApiError::DataBase(_) | ApiError::Bcrypt(_) => {
                axum::http::StatusCode::INTERNAL_SERVER_ERROR
            }
            ApiError::Query(_) | ApiError::Path(_) | ApiError::Validation(_) => {
                axum::http::StatusCode::BAD_REQUEST
            }
            ApiError::Jwt(_) | ApiError::UnAuthorized(_) => axum::http::StatusCode::UNAUTHORIZED,
            ApiError::MethodNotAllowed => axum::http::StatusCode::METHOD_NOT_ALLOWED,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status_code = self.status_code();

        let body = axum::Json(ApiResponse::<()>::err(
            status_code.as_u16(),
            self.to_string(),
        ));

        (status_code, body).into_response()
    }
}

impl From<ApiError> for Response {
    fn from(value: ApiError) -> Self {
        value.into_response()
    }
}
