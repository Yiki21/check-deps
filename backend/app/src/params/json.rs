use axum::extract::FromRequest;
use axum_valid::HasValidate;

#[derive(Debug, Clone, FromRequest)]
#[from_request(via(axum::extract::Json), rejection(crate::common::error::ApiError))]
pub struct Json<T>(pub T);

impl<T> HasValidate for Json<T>
where
    T: HasValidate,
{
    type Validate = T;

    fn get_validate(&self) -> &Self::Validate {
        return &self.0;
    }
}
