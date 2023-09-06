use actix_web::{get, web, HttpResponse, Responder};
use log::error;
use serde::{Deserialize, Serialize};
use utils;

#[derive(Serialize, Deserialize)]
struct HealthResponse {
    status: String,
    services: ServiceStatus,
}

#[derive(Serialize, Deserialize, Clone)]
struct ServiceStatus {
    postgres: String,
}

const SERVICE_STATUS_OK: &str = "ok";
const SERVICE_STATUS_ERROR: &str = "error";

#[get("/health")]
pub async fn get_health(pool: web::Data<utils::db::PgPool>) -> impl Responder {
    let db_status = match pool.get() {
        Ok(_) => SERVICE_STATUS_OK,
        Err(err) => {
            error!("failed getting connection: {}", err);
            SERVICE_STATUS_ERROR
        }
    };

    let status = db_status.to_string();
    let service_status = ServiceStatus {
        postgres: status.clone(),
    };

    let response = HealthResponse {
        status: status.clone(),
        services: service_status.clone(),
    };

    let http_response = if db_status == SERVICE_STATUS_OK {
        HttpResponse::Ok().json(response)
    } else {
        HttpResponse::InternalServerError().json(response)
    };

    return http_response;
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use actix_web::dev::Service;
//     use actix_web::http::StatusCode;
//     use actix_web::{test, App};
//     use sqlx::postgres::PgPoolOptions;

//     #[actix_rt::test]
//     async fn test_get_health_success() {
//         // Create a test pool
//         let pool = PgPoolOptions::new()
//             .connect("postgres://user:pass@localhost/testdb")
//             .await
//             .expect("Failed to create pool");

//         // Create an instance of the   App
//         let app = test::init_service(
//             App::new()
//                 .app_data(web::Data::new(pool.clone()))
//                 .service(get_health),
//         )
//         .await;

//         // Send a test request to /health
//         let req = test::TestRequest::get().uri("/health").to_request();
//         let resp = app.call(req).await.unwrap();

//         // Assert the response status is 200 OK
//         assert_eq!(resp.status(), StatusCode::OK);

//         // Assert the response body matches the expected JSON
//         let body = test::read_body(resp).await;
//         let expected = json!({
//             "status": "ok",
//             "services": {
//                 "postgres": "ok"
//             }
//         });
//         assert_eq!(body, expected.to_string());
//     }

//     #[actix_rt::test]
//     async fn test_get_health_error() {
//         // Create an instance of the App without a pool to intentionally cause an error
//         let app = test::init_service(App::new().service(get_health)).await;

//         // Send a test request to /health
//         let req = test::TestRequest::get().uri("/health").to_request();
//         let resp = app.call(req).await.unwrap();

//         // Assert the response status is 500 Internal Server Error
//         assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);

//         // Assert the response body matches the expected JSON
//         let body = test::read_body(resp).await;
//         let expected = json!({
//             "status": "error",
//             "services": {
//                 "postgres": "error"
//             }
//         });
//         assert_eq!(body, expected.to_string());
//     }
// }
