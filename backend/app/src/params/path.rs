use axum::extract::FromRequestParts;
use axum_valid::HasValidate;

use crate::common::error::ApiError;

#[derive(Debug, Clone, Default, FromRequestParts)]
#[from_request(via(axum::extract::Path), rejection(ApiError))]
pub struct Path<T>(pub T);

impl<T> HasValidate for Path<T>
where
    T: HasValidate,
{
    type Validate = T;

    fn get_validate(&self) -> &Self::Validate {
        return &self.0;
    }
}
