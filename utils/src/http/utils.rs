use actix_web::body::MessageBody;
use actix_web::dev::{ServiceFactory, ServiceRequest, ServiceResponse};
use actix_web::middleware::Logger;
use actix_web::{error::Error, App};

use super::middlewares::cors::cors;

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
