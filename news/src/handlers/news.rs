use actix_web::HttpMessage;
use actix_web::{get, web, HttpRequest, HttpResponse};
use log::error;
use utils::{error::CommonError, http::middlewares::jwt_auth::JwtMiddleware};

use utils::news::repositories::news_repository::NewsRepository;

#[get("/news")]
async fn get_news(
    r: HttpRequest,
    news_repo: web::Data<dyn NewsRepository>,
    _: JwtMiddleware,
) -> HttpResponse {
    let user_id = *r.extensions().get::<uuid::Uuid>().unwrap();

    let result = news_repo.find_by_user_id(user_id);

    match result {
        Err(err) => {
            error!("failed getting news: {}", CommonError::from(err));
            HttpResponse::InternalServerError().finish()
        }
        Ok(news) => HttpResponse::Ok().json(news),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::http::StatusCode;
    use actix_web::{test, web, App};
    use chrono::NaiveDate;
    use rstest::*;
    use std::str::FromStr;
    use std::sync::Arc;
    use utils::error::DatabaseError;
    use utils::http::services::auth_service::{AuthService, JwtAuthService};
    use utils::http::test_utils::HttpTestCase;
    use uuid::Uuid;

    use utils::news::models::news::News;
    use utils::news::repositories::news_repository::MockNewsRepository;

    struct GetNewsTestCase {
        pub service_result: Result<Vec<News>, DatabaseError>,
        pub http_case: HttpTestCase,
    }

    #[rstest]
    #[case(GetNewsTestCase {
        http_case: HttpTestCase{
            expected_status: StatusCode::OK,
            expected_body:  r#"[{"id":"fdd0a6f3-af61-4760-a789-5b6dd16eb7dc","author":"author1","url":"url1","title":"newspaper1","publish_date":"2022-01-01","feed_id":"fdd0a6f3-af61-4760-a789-5b6dd16eb7dc"},{"id":"b73ccd26-1832-4d10-9251-271ce453cee3","author":"author2","url":"url1","title":"newspaper2","publish_date":"2022-01-01","feed_id":"fdd0a6f3-af61-4760-a789-5b6dd16eb7dc"}]"#,
        },
        service_result: Ok(vec![
            News {
                id: Uuid::from_str("fdd0a6f3-af61-4760-a789-5b6dd16eb7dc").unwrap(),
                title: "newspaper1".to_owned(),
                author: "author1".to_owned(),
                url: "url1".to_owned(),
                feed_id: Uuid::from_str("fdd0a6f3-af61-4760-a789-5b6dd16eb7dc").unwrap(),
                publish_date: NaiveDate::from_ymd_opt(2022, 1, 1),
            },
            News {
                id: Uuid::from_str("b73ccd26-1832-4d10-9251-271ce453cee3").unwrap(),
                title: "newspaper2".to_owned(),
                author: "author2".to_owned(),
                url: "url1".to_owned(),
                feed_id: Uuid::from_str("fdd0a6f3-af61-4760-a789-5b6dd16eb7dc").unwrap(),
                publish_date: NaiveDate::from_ymd_opt(2022, 1, 1),
            },
        ]),
    })]
    #[case(GetNewsTestCase {
        http_case: HttpTestCase{
            expected_status: StatusCode::INTERNAL_SERVER_ERROR,
            expected_body: "",
        },
        service_result: Err(DatabaseError {
            message: "db is down".to_owned(),
        }),
    })]
    #[actix_rt::test]
    async fn test_get_news(#[case] case: GetNewsTestCase) {
        let auth_service: Arc<dyn AuthService> =
            Arc::new(JwtAuthService::new("secret123".to_owned()));
        let mut news_repo = MockNewsRepository::new();

        let service_result = case.service_result.clone();

        news_repo
            .expect_find_by_user_id()
            .returning(move |_| service_result.clone());

        let news_repo: Arc<dyn NewsRepository> = Arc::new(news_repo);

        // Create a test App
        let app = test::init_service(
            App::new()
                .app_data(web::Data::from(news_repo))
                .app_data(web::Data::from(auth_service.clone()))
                .service(web::scope("").service(get_news)),
        )
        .await;

        case.http_case
            .execute(
                test::TestRequest::get().uri("/news"),
                &app,
                Some(auth_service.clone()),
            )
            .await;
    }
}
