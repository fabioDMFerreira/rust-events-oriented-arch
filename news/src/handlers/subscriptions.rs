use actix_web::{delete, get, web, HttpRequest, HttpResponse};
use actix_web::{post, HttpMessage};
use log::error;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use utils::{error::CommonError, http::middlewares::jwt_auth::JwtMiddleware};
use uuid::Uuid;
use validator::Validate;

use crate::models::subscription::Subscription;
use crate::repositories::subscription_repository::SubscriptionRepository;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateSubscriptionPayload {
    #[validate(required)]
    pub feed_id: Option<String>,
}

#[derive(Deserialize)]
struct DeleteSubscriptionPayload {
    feed_id: Option<String>,
}

#[get("/subscriptions")]
async fn get_subscriptions(
    r: HttpRequest,
    subscription_repo: web::Data<dyn SubscriptionRepository>,
    _: JwtMiddleware,
) -> HttpResponse {
    let user_id = *r.extensions().get::<uuid::Uuid>().unwrap();

    let result = subscription_repo.list_by_user(user_id);

    match result {
        Err(err) => {
            error!("failed getting subscriptions: {}", CommonError::from(err));
            HttpResponse::InternalServerError().finish()
        }
        Ok(feeds) => HttpResponse::Ok().json(feeds),
    }
}

#[post("/subscriptions")]
async fn create_subscription(
    r: HttpRequest,
    subscription_repo: web::Data<dyn SubscriptionRepository>,
    payload: Option<web::Json<CreateSubscriptionPayload>>,
    _: JwtMiddleware,
) -> HttpResponse {
    let user_id = *r.extensions().get::<uuid::Uuid>().unwrap();

    if payload.is_none() {
        return HttpResponse::BadRequest().body("empty body");
    }

    let payload = payload.unwrap();
    if let Err(err) = payload.validate() {
        return HttpResponse::BadRequest().json(err);
    }

    let CreateSubscriptionPayload { feed_id } = payload.into_inner();

    if feed_id.is_none() {
        return HttpResponse::BadRequest().body("Missing 'feed_id' in body");
    }

    let feed_id = Uuid::from_str(&feed_id.unwrap().to_string()).unwrap();

    let result = subscription_repo.create(&Subscription { feed_id, user_id });

    match result {
        Err(err) => {
            error!("failed creating subscription: {}", CommonError::from(err));
            HttpResponse::InternalServerError().finish()
        }
        Ok(subscription) => HttpResponse::Ok().json(subscription),
    }
}

#[delete("/subscriptions")]
async fn delete_subscription(
    r: HttpRequest,
    subscription_repo: web::Data<dyn SubscriptionRepository>,
    _: JwtMiddleware,
    payload: web::Query<DeleteSubscriptionPayload>,
) -> HttpResponse {
    let user_id = *r.extensions().get::<uuid::Uuid>().unwrap();

    if let Some(feed_id) = &payload.feed_id {
        let feed_id = Uuid::from_str(&feed_id.clone()).unwrap();

        let result = subscription_repo.delete(feed_id, user_id);

        match result {
            Err(err) => {
                error!("failed deleting subscription: {}", CommonError::from(err));
                HttpResponse::InternalServerError().finish()
            }
            Ok(subscription) => HttpResponse::Ok().json(subscription),
        }
    } else {
        HttpResponse::BadRequest().body("Missing 'feed_id' query parameter")
    }
}
