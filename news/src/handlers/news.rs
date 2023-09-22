use actix_web::{get, web, HttpResponse};
use log::error;

use crate::repositories::news_repository::NewsRepository;

#[get("/news")]
async fn get_news(news_repo: web::Data<NewsRepository>) -> HttpResponse {
    let result = news_repo.list();

    match result {
        Err(err) => {
            error!("failed getting news: {}", err);
            HttpResponse::InternalServerError().finish()
        }
        Ok(news) => HttpResponse::Ok().json(news),
    }
}
