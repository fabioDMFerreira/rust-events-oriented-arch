use actix_cors::Cors;
use actix_web::body::MessageBody;
use actix_web::dev::{ServiceFactory, ServiceRequest, ServiceResponse};
use actix_web::{error::Error, App};
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

pub fn build_server(
    cors_origin: String,
) -> App<
    impl ServiceFactory<
        ServiceRequest,
        Response = ServiceResponse<impl MessageBody>,
        Config = (),
        InitError = (),
        Error = Error,
    >,
> {
    App::new().wrap(cors(cors_origin.clone())).wrap(logger())
}
