use actix_web::{get, HttpResponse, Responder};

#[get("/")]
pub async fn get_index() -> impl Responder {
    HttpResponse::Ok().body("v0.1.0")
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::dev::Service;
    use actix_web::http::StatusCode;
    use actix_web::{test, App};

    #[actix_rt::test]
    async fn test_get_index() {
        let app = test::init_service(App::new().service(get_index)).await;

        let req = test::TestRequest::get().uri("/").to_request();
        let resp = app.call(req).await.unwrap();

        assert_eq!(resp.status(), StatusCode::OK);

        let body = test::read_body(resp).await;
        assert_eq!(body, "v0.1.0");
    }
}
