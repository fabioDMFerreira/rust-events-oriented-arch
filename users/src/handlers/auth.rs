use actix_web::{
    cookie::{time::Duration as ActixWebDuration, Cookie},
    get, post, web, HttpMessage, HttpRequest, HttpResponse, Responder,
};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use log::error;
use serde::{Deserialize, Serialize};
use serde_json::json;
use validator::Validate;

use crate::{
    config::Config, middlewares::jwt_auth::JwtMiddleware, models::token_claims::TokenClaims,
    services::user_service::UserService,
};

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct LoginPayload {
    #[validate(required, length(min = 2))]
    name: Option<String>,
    #[validate(required, length(min = 6))]
    password: Option<String>,
}

#[get("/auth/logout")]
pub async fn logout_handler(_: JwtMiddleware) -> impl Responder {
    let cookie = Cookie::build("token", "")
        .path("/")
        .max_age(ActixWebDuration::new(-1, 0))
        .http_only(true)
        .finish();

    HttpResponse::Ok()
        .cookie(cookie)
        .json(json!({"status": "success"}))
}

#[post("/auth/login")]
pub async fn login_handler(
    user_service: web::Data<dyn UserService>,
    config: web::Data<Config>,
    payload: Option<web::Json<LoginPayload>>,
) -> HttpResponse {
    if payload.is_none() {
        return HttpResponse::BadRequest().body("empty body");
    }

    let payload = payload.unwrap().into_inner();

    if let Err(e) = payload.validate() {
        return HttpResponse::BadRequest().json(e);
    }

    let LoginPayload { name, password } = payload;
    let name = name.unwrap();

    let user = match user_service.get_by_name(name.clone()).await {
        Ok(user) => user,
        Err(err) => {
            error!("failed getting user by name {}: {}", name, err);
            return HttpResponse::InternalServerError()
                .json(json!({"status": "fail", "message": "Invalid user"}));
        }
    };

    let parsed_hash = PasswordHash::new(&user.password).unwrap();
    let password_is_valid = Argon2::default()
        .verify_password(password.unwrap().as_bytes(), &parsed_hash)
        .map_or(false, |_| true);

    if !password_is_valid {
        return HttpResponse::BadRequest()
            .json(json!({"status": "fail", "message": "Invalid password"}));
    }

    let now = Utc::now();
    let iat = now.timestamp() as usize;
    let exp = (now + Duration::minutes(config.jwt_expires_in)).timestamp() as usize;
    let claims: TokenClaims = TokenClaims {
        sub: user.id.to_string(),
        exp,
        iat,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_ref()),
    )
    .unwrap();

    let cookie = Cookie::build("token", token.to_owned())
        .path("/")
        .max_age(ActixWebDuration::new(config.jwt_max_age, 0))
        .http_only(true)
        .finish();

    return HttpResponse::Ok()
        .cookie(cookie)
        .json(json!({"status": "success", "token": token}));
}

#[get("/auth/me")]
pub async fn me_handler(
    user_service: web::Data<dyn UserService>,
    r: HttpRequest,
    _: JwtMiddleware,
) -> HttpResponse {
    let user_id = *r.extensions().get::<uuid::Uuid>().unwrap();

    match user_service.get_by_id(user_id).await {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(err) => {
            error!("failed getting user {}: {}", user_id, err);
            HttpResponse::InternalServerError().finish()
        }
    }
}
