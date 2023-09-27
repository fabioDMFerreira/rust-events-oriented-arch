use core::fmt;
use std::future::{ready, Ready};

use actix_web::error::{Error as ActixError, ErrorUnauthorized};
use actix_web::{dev::Payload, Error as ActixWebError};
use actix_web::{http, web, FromRequest, HttpMessage, HttpRequest};
use serde::{Deserialize, Serialize};

use crate::http::services::auth_service::AuthService;

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String,
    pub iat: usize,
    pub exp: usize,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    status: String,
    message: String,
}

impl fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(&self).unwrap())
    }
}

pub struct JwtMiddleware {
    pub user_id: uuid::Uuid,
}

impl FromRequest for JwtMiddleware {
    type Error = ActixWebError;
    type Future = Ready<Result<Self, Self::Error>>;
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let auth_service = req.app_data::<web::Data<dyn AuthService>>();

        let json_error = |message: String| -> ActixError {
            ErrorUnauthorized(ErrorResponse {
                status: "fail".to_string(),
                message,
            })
        };

        if auth_service.is_none() {
            return ready(Err(json_error("invalid configuration".to_string())));
        }

        let auth_service = auth_service.unwrap();

        let token = req
            .cookie("token")
            .map(|c| c.value().to_string())
            .or_else(|| {
                req.headers()
                    .get(http::header::AUTHORIZATION)
                    .map(|h| h.to_str().unwrap().split_at(7).1.to_string())
            });

        if let Some(token) = token {
            if let Ok(claims) = auth_service.decode_token(token) {
                let user_id = uuid::Uuid::parse_str(&claims.sub).unwrap();
                req.extensions_mut()
                    .insert::<uuid::Uuid>(user_id.to_owned());
                ready(Ok(JwtMiddleware { user_id }))
            } else {
                ready(Err(json_error("Invalid token".to_string())))
            }
        } else {
            ready(Err(json_error(
                "You are not logged in, please provide token".to_string(),
            )))
        }
    }
}
