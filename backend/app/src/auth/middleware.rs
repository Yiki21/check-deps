use std::sync::LazyLock;
use std::{future::Future, pin::Pin};

use axum::body::Body;
use axum::http::{Request, Response, header::AUTHORIZATION};
use tower_http::auth::{AsyncAuthorizeRequest, AsyncRequireAuthorizationLayer};

use crate::auth::Principal;
use crate::common::error::ApiError;
use crate::config;
use crate::config::auth::AuthConfig;

use super::service::{JwtService, jwt_service};

pub struct JwtAuthLayer {
    jwt: &'static JwtService,
    auth_config: &'static AuthConfig,
}

impl JwtAuthLayer {
    pub fn new(jwt: &'static JwtService, auth_config: &'static AuthConfig) -> Self {
        JwtAuthLayer { jwt, auth_config }
    }
}

pub fn get_jwt_auth_layer() -> AsyncRequireAuthorizationLayer<JwtAuthLayer> {
    AsyncRequireAuthorizationLayer::new(JwtAuthLayer::new(jwt_service(), &config::get().auth()))
}

static GUEST_PRINCIPAL: LazyLock<Principal> = LazyLock::new(|| Principal {
    id: "guest".to_string(),
    name: "guest".to_string(),
});

static USERLESS_PRINCIPAL: LazyLock<Principal> = LazyLock::new(|| Principal {
    id: "userless".to_string(),
    name: "userless".to_string(),
});

impl AsyncAuthorizeRequest<Body> for JwtAuthLayer {
    type RequestBody = Body;

    type ResponseBody = Body;

    type Future = Pin<
        Box<
            dyn Future<Output = Result<Request<Self::RequestBody>, Response<Self::ResponseBody>>>
                + Send,
        >,
    >;

    fn authorize(&mut self, mut request: Request<Body>) -> Self::Future {
        if self.auth_config.userless() {
            request.extensions_mut().insert(USERLESS_PRINCIPAL.clone());

            return Box::pin(async move { Ok(request) });
        }

        let jwt = self.jwt;
        let path = request.uri().path();

        if self
            .auth_config
            .allow_list()
            .iter()
            .any(|prefix| path.starts_with(prefix))
        {
            request.extensions_mut().insert(GUEST_PRINCIPAL.clone());

            return Box::pin(async move { Ok(request) });
        }

        Box::pin(async move {
            let token = match extract_bearer(&request) {
                Ok(token) => token,
                Err(err) => return Err(err.into()),
            };

            let principal = match jwt.decode(&token) {
                Ok(principal) => principal,
                Err(err) => return Err(ApiError::from(err).into()),
            };

            request.extensions_mut().insert(principal);

            Ok(request)
        })
    }
}

fn extract_bearer(request: &Request<Body>) -> Result<String, ApiError> {
    let header = request
        .headers()
        .get(AUTHORIZATION)
        .ok_or_else(|| ApiError::UnAuthorized("Authorization header missing".to_string()))?;

    let raw = header
        .to_str()
        .map_err(|_| ApiError::UnAuthorized("Invalid Authorization header".to_string()))?;

    let token = raw
        .strip_prefix("Bearer ")
        .ok_or_else(|| ApiError::UnAuthorized("Malformed Authorization header".to_string()))?;

    Ok(token.to_string())
}
