use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};

use crate::{
    error::{CommonError, AUTH_TOKEN_ENCODING_CODE},
    http::middlewares::jwt_auth::TokenClaims,
};

pub trait AuthService {
    fn encode_token(&self, user_id: String, expires_in: i64) -> Result<String, CommonError>;
    fn decode_token(&self, token: String) -> Result<TokenClaims, CommonError>;
}

pub struct JwtAuthService {
    jwt_secret: String,
}

impl JwtAuthService {
    pub fn new(jwt_secret: String) -> Self {
        JwtAuthService { jwt_secret }
    }
}

impl AuthService for JwtAuthService {
    fn encode_token(&self, user_id: String, expires_in: i64) -> Result<String, CommonError> {
        let now = Utc::now();
        let iat = now.timestamp() as usize;
        let exp = (now + Duration::minutes(expires_in)).timestamp() as usize;
        let claims: TokenClaims = TokenClaims {
            sub: user_id.to_string(),
            exp,
            iat,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_ref()),
        )
        .map_err(|err| CommonError {
            message: err.to_string(),
            code: AUTH_TOKEN_ENCODING_CODE,
        })
    }

    fn decode_token(&self, token: String) -> Result<TokenClaims, CommonError> {
        decode::<TokenClaims>(
            &token,
            &DecodingKey::from_secret(self.jwt_secret.as_ref()),
            &Validation::default(),
        )
        .map(|c| c.claims)
        .map_err(|err| CommonError {
            message: err.to_string(),
            code: AUTH_TOKEN_ENCODING_CODE,
        })
    }
}
