use actix_web::{get, web, HttpResponse};
use log::error;
use utils::error::CommonError;

use crate::repositories::feed_repository::FeedRepository;

#[get("/feeds")]
async fn get_feeds(feed_repo: web::Data<dyn FeedRepository>) -> HttpResponse {
    let result = feed_repo.list();

    match result {
        Err(err) => {
            error!("failed getting feeds: {}", CommonError::from(err));
            HttpResponse::InternalServerError().finish()
        }
        Ok(feeds) => HttpResponse::Ok().json(feeds),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::http::StatusCode;
    use actix_web::{test, web, App};
    use serde_json;
    use std::str::FromStr;
    use std::sync::Arc;
    use utils::error::DatabaseError;
    use uuid::Uuid;

    use crate::models::feed::Feed;
    use crate::repositories::feed_repository::MockFeedRepository;

    #[actix_rt::test]
    async fn test_get_feeds_success() {
        let mut feeds_repo = MockFeedRepository::new();

        feeds_repo.expect_list().returning(|| {
            Ok(vec![
                Feed {
                    id: Uuid::from_str("fdd0a6f3-af61-4760-a789-5b6dd16eb7dc").unwrap(),
                    title: "newspaper1".to_owned(),
                    author: "author1".to_owned(),
                    url: "url1".to_owned(),
                },
                Feed {
                    id: Uuid::from_str("b73ccd26-1832-4d10-9251-271ce453cee3").unwrap(),
                    title: "newspaper2".to_owned(),
                    author: "author2".to_owned(),
                    url: "url1".to_owned(),
                },
            ])
        });

        let feeds_repo: Arc<dyn FeedRepository> = Arc::new(feeds_repo);

        // Create a test App
        let app = test::init_service(
            App::new()
                .app_data(web::Data::from(feeds_repo))
                .service(web::scope("").service(get_feeds)),
        )
        .await;

        // Create a GET request to /feeds
        let req = test::TestRequest::get().uri("/feeds").to_request();

        // Send the request to the app
        let resp = test::call_service(&app, req).await;

        // Assert that the response status is Ok (200)
        assert_eq!(resp.status(), StatusCode::OK);

        // Assert the response body contains the expected data
        let body = test::read_body(resp).await;
        let feeds: Vec<Feed> = serde_json::from_slice(&body).unwrap();

        // Assert the expected number of feeds
        assert_eq!(feeds.len(), 2);
        // Add more assertions to validate the returned feed data
    }

    #[actix_rt::test]
    async fn test_get_feeds_error() {
        let mut feeds_repo = MockFeedRepository::new();

        feeds_repo.expect_list().returning(|| {
            Err(DatabaseError {
                message: "db is down".to_string(),
            })
        });

        let feeds_repo: Arc<dyn FeedRepository> = Arc::new(feeds_repo);

        // Create a test App
        let app = test::init_service(
            App::new()
                .app_data(web::Data::from(feeds_repo))
                .service(web::scope("").service(get_feeds)),
        )
        .await;

        // Create a GET request to /feeds
        let req = test::TestRequest::get().uri("/feeds").to_request();

        // Send the request to the app
        let resp = test::call_service(&app, req).await;

        // Assert that the response status is Ok (200)
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
