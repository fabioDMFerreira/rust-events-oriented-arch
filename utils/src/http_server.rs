use actix_cors::Cors;
use actix_web::{http::header, middleware::Logger};

pub fn cors(origin: String) -> Cors {
    Cors::default()
        .allowed_origin(&origin)
        .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
        .allowed_headers(vec![
            header::CONTENT_TYPE,
            header::AUTHORIZATION,
            header::ACCEPT,
        ])
        .supports_credentials()
}

pub fn logger() -> Logger {
    Logger::default()
}
