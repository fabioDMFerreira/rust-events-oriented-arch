use actix_web::{get, web, HttpResponse};
use log::error;

use crate::repositories::feed_repository::FeedRepository;

#[get("/feeds")]
async fn get_feeds(feed_repo: web::Data<FeedRepository>) -> HttpResponse {
    let result = feed_repo.list();

    match result {
        Err(err) => {
            error!("failed getting feeds: {}", err);
            return HttpResponse::InternalServerError().finish();
        }
        Ok(feeds) => return HttpResponse::Ok().json(feeds),
    };
}
