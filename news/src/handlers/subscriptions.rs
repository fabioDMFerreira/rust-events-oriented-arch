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

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::http::StatusCode;
    use actix_web::{test, web, App};
    use rstest::*;
    use std::str::FromStr;
    use std::sync::Arc;
    use utils::error::DatabaseError;
    use utils::http::services::auth_service::{AuthService, JwtAuthService};
    use utils::http::test_utils::HttpTestCase;
    use uuid::Uuid;

    use crate::repositories::subscription_repository::MockSubscriptionRepository;

    struct GetSubscriptionsTestCase {
        http_case: HttpTestCase,
        service_result: Result<Vec<Subscription>, DatabaseError>,
    }

    #[rstest]
    #[case(GetSubscriptionsTestCase {
        http_case: HttpTestCase {
            expected_status: StatusCode::OK,
            expected_body: r#"[{"feed_id":"fdd0a6f3-af61-4760-a789-5b6dd16eb7dc","user_id":"b73ccd26-1832-4d10-9251-271ce453cee3"},{"feed_id":"fdd0a6f3-af61-4760-a789-5b6dd16eb7dc","user_id":"b73ccd26-1832-4d10-9251-271ce453cee3"}]"#,
        },
        service_result: Ok(vec![
            Subscription {
                feed_id: Uuid::from_str("fdd0a6f3-af61-4760-a789-5b6dd16eb7dc").unwrap(),
                user_id: Uuid::from_str("b73ccd26-1832-4d10-9251-271ce453cee3").unwrap(),
            },
            Subscription {
                feed_id: Uuid::from_str("fdd0a6f3-af61-4760-a789-5b6dd16eb7dc").unwrap(),
                user_id: Uuid::from_str("b73ccd26-1832-4d10-9251-271ce453cee3").unwrap(),
            },
        ])
    })]
    #[case(GetSubscriptionsTestCase {
        http_case : HttpTestCase {
            expected_status: StatusCode::INTERNAL_SERVER_ERROR,
            expected_body: r#""#,
        },
        service_result: Err(DatabaseError { message: "db is down".to_owned() })
    })]
    #[actix_rt::test]
    async fn test_get_subscriptions(#[case] case: GetSubscriptionsTestCase) {
        let mut subscriptions_repo = MockSubscriptionRepository::new();
        let auth_service: Arc<dyn AuthService> =
            Arc::new(JwtAuthService::new("secret123".to_owned()));

        let service_result = case.service_result.clone();

        subscriptions_repo
            .expect_list_by_user()
            .returning(move |_| service_result.clone());

        let subscriptions_repo: Arc<dyn SubscriptionRepository> = Arc::new(subscriptions_repo);

        // Create a test App
        let app = test::init_service(
            App::new()
                .app_data(web::Data::from(subscriptions_repo))
                .app_data(web::Data::from(auth_service.clone()))
                .service(web::scope("").service(get_subscriptions)),
        )
        .await;

        case.http_case
            .execute(
                test::TestRequest::get().uri("/subscriptions"),
                &app,
                Some(auth_service.clone()),
            )
            .await;
    }

    struct CreateSubscriptionTestCase {
        http_case: HttpTestCase,
        service_result: Result<Subscription, DatabaseError>,
        request_payload: Option<CreateSubscriptionPayload>,
    }

    #[rstest]
    #[case(CreateSubscriptionTestCase {
        http_case: HttpTestCase {
            expected_status: StatusCode::OK,
            expected_body: r#"{"feed_id":"fdd0a6f3-af61-4760-a789-5b6dd16eb7dc","user_id":"b73ccd26-1832-4d10-9251-271ce453cee3"}"#,
        },
        service_result: Ok(
            Subscription {
                feed_id: Uuid::from_str("fdd0a6f3-af61-4760-a789-5b6dd16eb7dc").unwrap(),
                user_id: Uuid::from_str("b73ccd26-1832-4d10-9251-271ce453cee3").unwrap(),
            }
        ),
        request_payload: Some(CreateSubscriptionPayload{
            feed_id:Some("fdd0a6f3-af61-4760-a789-5b6dd16eb7dc".to_owned()),
        })
    })]
    #[case(CreateSubscriptionTestCase {
        http_case : HttpTestCase {
            expected_status: StatusCode::BAD_REQUEST,
            expected_body: r#"empty body"#,
        },
        service_result: Err(DatabaseError { message: "db is down".to_owned() }),
        request_payload: None
    })]
    #[case(CreateSubscriptionTestCase {
        http_case : HttpTestCase {
            expected_status: StatusCode::INTERNAL_SERVER_ERROR,
            expected_body: r#""#,
        },
        service_result: Err(DatabaseError { message: "db is down".to_owned() }),
        request_payload: Some(CreateSubscriptionPayload{
            feed_id:Some("fdd0a6f3-af61-4760-a789-5b6dd16eb7dc".to_owned()),
        })
    })]
    #[actix_rt::test]
    async fn test_create_subscriptions(#[case] case: CreateSubscriptionTestCase) {
        let mut subscriptions_repo = MockSubscriptionRepository::new();
        let auth_service: Arc<dyn AuthService> =
            Arc::new(JwtAuthService::new("secret123".to_owned()));

        let service_result = case.service_result.clone();

        subscriptions_repo
            .expect_create()
            .returning(move |_| service_result.clone());

        let subscriptions_repo: Arc<dyn SubscriptionRepository> = Arc::new(subscriptions_repo);

        // Create a test App
        let app = test::init_service(
            App::new()
                .app_data(web::Data::from(subscriptions_repo))
                .app_data(web::Data::from(auth_service.clone()))
                .service(web::scope("").service(create_subscription)),
        )
        .await;

        let mut req = test::TestRequest::post().uri("/subscriptions");

        if let Some(payload) = case.request_payload {
            req = req
                .set_payload(serde_json::to_vec(&payload).unwrap())
                .insert_header(("content-type", "application/json"))
        }

        case.http_case
            .execute(req, &app, Some(auth_service.clone()))
            .await;
    }

    struct DeleteSubscriptionTestCase {
        http_case: HttpTestCase,
        service_result: Result<usize, DatabaseError>,
        feed_id: Option<String>,
    }

    #[rstest]
    #[case(DeleteSubscriptionTestCase {
        http_case: HttpTestCase {
            expected_status: StatusCode::OK,
            expected_body: r#"1"#,
        },
        service_result: Ok(1),
        feed_id: Some("fdd0a6f3-af61-4760-a789-5b6dd16eb7dc".to_owned()),
    })]
    #[case(DeleteSubscriptionTestCase {
        http_case : HttpTestCase {
            expected_status: StatusCode::BAD_REQUEST,
            expected_body: r#"Missing 'feed_id' query parameter"#,
        },
        service_result: Err(DatabaseError { message: "db is down".to_owned() }),
        feed_id: None
    })]
    #[case(DeleteSubscriptionTestCase {
        http_case : HttpTestCase {
            expected_status: StatusCode::INTERNAL_SERVER_ERROR,
            expected_body: r#""#,
        },
        service_result: Err(DatabaseError { message: "db is down".to_owned() }),
        feed_id: Some("fdd0a6f3-af61-4760-a789-5b6dd16eb7dc".to_owned()),
    })]
    #[actix_rt::test]
    async fn test_delete_subscriptions(#[case] case: DeleteSubscriptionTestCase) {
        let mut subscriptions_repo = MockSubscriptionRepository::new();
        let auth_service: Arc<dyn AuthService> =
            Arc::new(JwtAuthService::new("secret123".to_owned()));

        let service_result = case.service_result.clone();

        subscriptions_repo
            .expect_delete()
            .returning(move |_, _| service_result.clone());

        let subscriptions_repo: Arc<dyn SubscriptionRepository> = Arc::new(subscriptions_repo);

        // Create a test App
        let app = test::init_service(
            App::new()
                .app_data(web::Data::from(subscriptions_repo))
                .app_data(web::Data::from(auth_service.clone()))
                .service(web::scope("").service(delete_subscription)),
        )
        .await;

        let mut uri = "/subscriptions".to_owned();

        if let Some(feed_id) = case.feed_id {
            uri = format!("{}?feed_id={}", uri.to_string(), feed_id.to_string());
        }

        let req = test::TestRequest::delete().uri(&uri);

        case.http_case
            .execute(req, &app, Some(auth_service.clone()))
            .await;
    }
}
