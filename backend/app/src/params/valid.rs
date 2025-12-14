use axum::extract::{FromRequest, FromRequestParts};

use crate::
    common::error::ApiError
;

#[derive(Debug, Clone, Default, FromRequest, FromRequestParts)]
#[from_request(via(axum_valid::Valid), rejection(ApiError))]
pub struct Valid<T>(pub T);
