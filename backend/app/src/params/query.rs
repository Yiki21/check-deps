use axum::extract::FromRequestParts;
use axum_valid::HasValidate;

use crate::common::error::ApiError;

#[derive(Debug, Clone, Copy, Default, FromRequestParts)]
#[from_request(via(axum::extract::Query), rejection(ApiError))]
pub struct Query<T>(pub T);

impl<T> HasValidate for Query<T>
where
    T: HasValidate,
{
    type Validate = T;

    fn get_validate(&self) -> &Self::Validate {
        return &self.0;
    }
}
