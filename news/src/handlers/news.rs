use actix_web::HttpMessage;
use actix_web::{get, web, HttpRequest, HttpResponse};
use log::error;
use utils::{error::CommonError, http::middlewares::jwt_auth::JwtMiddleware};

use crate::repositories::news_repository::NewsRepository;

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
    use serde_json;
    use std::str::FromStr;
    use std::sync::Arc;
    use utils::error::DatabaseError;
    use uuid::Uuid;

    use crate::models::news::News;
    use crate::repositories::news_repository::MockNewsRepository;

    #[actix_rt::test]
    async fn test_get_news_success() {
        let mut news_repo = MockNewsRepository::new();

        news_repo.expect_list().returning(|| {
            Ok(vec![
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
            ])
        });

        let news_repo: Arc<dyn NewsRepository> = Arc::new(news_repo);

        // Create a test App
        let app = test::init_service(
            App::new()
                .app_data(web::Data::from(news_repo))
                .service(web::scope("").service(get_news)),
        )
        .await;

        // Create a GET request to /news
        let req = test::TestRequest::get().uri("/news").to_request();

        // Send the request to the app
        let resp = test::call_service(&app, req).await;

        // Assert that the response status is Ok (200)
        assert_eq!(resp.status(), StatusCode::OK);

        // Assert the response body contains the expected data
        let body = test::read_body(resp).await;
        let news: Vec<News> = serde_json::from_slice(&body).unwrap();

        // Assert the expected number of news
        assert_eq!(news.len(), 2);
        // Add more assertions to validate the returned feed data
    }

    #[actix_rt::test]
    async fn test_get_news_error() {
        let mut news_repo = MockNewsRepository::new();

        news_repo.expect_list().returning(|| {
            Err(DatabaseError {
                message: "db is down".to_string(),
            })
        });

        let news_repo: Arc<dyn NewsRepository> = Arc::new(news_repo);

        // Create a test App
        let app = test::init_service(
            App::new()
                .app_data(web::Data::from(news_repo))
                .service(web::scope("").service(get_news)),
        )
        .await;

        // Create a GET request to /news
        let req = test::TestRequest::get().uri("/news").to_request();

        // Send the request to the app
        let resp = test::call_service(&app, req).await;

        // Assert that the response status is Ok (200)
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
