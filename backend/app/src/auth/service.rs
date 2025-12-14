use std::sync::LazyLock;

use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, encode, get_current_timestamp};

use crate::config::jwt::JwtConfig;

static DEFAULT_JWT: LazyLock<JwtService> = LazyLock::new(|| JwtService::new(crate::config::get().jwt()));

#[derive(Debug, Clone)]
pub struct Principal {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Claims {
    pub jti: String,
    pub sub: String,
    pub aud: String,
    pub iss: String,
    pub iat: u64,
    pub exp: u64,
}

#[derive(Debug, Clone)]
pub struct JwtService {
    config: &'static JwtConfig,
    encode_secret: EncodingKey,
    decode_secret: DecodingKey,
    header: Header,
    validation: Validation,
}

impl JwtService {
    pub fn new(jwt_config: &'static JwtConfig) -> Self {
        let mut validation = Validation::new(jwt_config.algorithm());
        validation.set_audience(&[&jwt_config.audience()]);
        validation.set_issuer(&[&jwt_config.issuer()]);
        validation.set_required_spec_claims(&["jti", "sub", "aud", "iss", "iat", "exp"]);

        let secret_bytes = jwt_config.secret().as_bytes();

        JwtService {
            config: jwt_config,
            encode_secret: EncodingKey::from_secret(secret_bytes),
            decode_secret: DecodingKey::from_secret(secret_bytes),
            header: Header::new(jwt_config.algorithm()),
            validation,
        }
    }

    pub fn encode(&self, principal: Principal) -> anyhow::Result<String> {
        let current_timestamp = get_current_timestamp();

        let claims = Claims {
            jti: xid::new().to_string(),
            sub: format!("{}:{}", principal.id, principal.name),
            aud: self.config.audience().to_string(),
            iss: self.config.issuer().to_string(),
            iat: current_timestamp,
            exp: current_timestamp + self.config.expiration().as_secs(),
        };

        Ok(encode(&self.header, &claims, &self.encode_secret)?)
    }

    pub fn decode(&self, token: &str) -> anyhow::Result<Principal> {
        let claims =
            jsonwebtoken::decode::<Claims>(token, &self.decode_secret, &self.validation)?.claims;

        let mut parts = claims.sub.splitn(2, ':');
        let principal = Principal {
            id: parts.next().unwrap().to_string(),
            name: parts.next().unwrap().to_string(),
        };

        Ok(principal)
    }
}

pub fn jwt_service() -> &'static JwtService {
    &DEFAULT_JWT
}
