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
    use rstest::*;
    use std::str::FromStr;
    use std::sync::Arc;
    use utils::error::DatabaseError;
    use utils::http::test_utils::HttpTestCase;
    use uuid::Uuid;

    use crate::models::feed::Feed;
    use crate::repositories::feed_repository::MockFeedRepository;

    struct GetFeedsTestCase {
        http_case: HttpTestCase,
        service_result: Result<Vec<Feed>, DatabaseError>,
    }

    #[rstest]
    #[case(GetFeedsTestCase {
        http_case: HttpTestCase {
            expected_status: StatusCode::OK,
            expected_body: r#"[{"id":"fdd0a6f3-af61-4760-a789-5b6dd16eb7dc","author":"author1","title":"newspaper1","url":"url1"},{"id":"b73ccd26-1832-4d10-9251-271ce453cee3","author":"author2","title":"newspaper2","url":"url1"}]"#,
        },
        service_result: Ok(vec![
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
    })]
    #[case(GetFeedsTestCase {
        http_case : HttpTestCase {
            expected_status: StatusCode::INTERNAL_SERVER_ERROR,
            expected_body: r#""#,
        },
        service_result: Err(DatabaseError { message: "db is down".to_owned() })
    })]
    #[actix_rt::test]
    async fn test_get_feeds(#[case] case: GetFeedsTestCase) {
        let mut feeds_repo = MockFeedRepository::new();

        let service_result = case.service_result.clone();

        feeds_repo
            .expect_list()
            .returning(move || service_result.clone());

        let feeds_repo: Arc<dyn FeedRepository> = Arc::new(feeds_repo);

        // Create a test App
        let app = test::init_service(
            App::new()
                .app_data(web::Data::from(feeds_repo))
                .service(web::scope("").service(get_feeds)),
        )
        .await;

        case.http_case
            .execute(test::TestRequest::get().uri("/feeds"), &app, None)
            .await;
    }
}
