use axum::response::IntoResponse;
use serde::Serialize;

#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub code: u16,
    pub message: String,
    pub data: Option<T>,
}

impl<T> ApiResponse<T> {
    pub fn new(code: u16, message: String, data: Option<T>) -> Self {
        Self {
            code,
            message,
            data,
        }
    }

    pub fn ok<M: AsRef<str>>(message: M, data: Option<T>) -> Self {
        Self {
            code: 200,
            message: String::from(message.as_ref()),
            data: data,
        }
    }

    pub fn err<M: AsRef<str>>(code: u16, message: M) -> Self {
        Self {
            code,
            message: String::from(message.as_ref()),
            data: None,
        }
    }
}

impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> axum::response::Response {
        let body = axum::Json(self);

        (axum::http::StatusCode::OK, body).into_response()
    }
}
