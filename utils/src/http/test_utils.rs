use std::sync::Arc;

use actix_http::{Request, StatusCode};
use actix_web::{
    dev::{Service, ServiceResponse},
    http::{self, header::TryIntoHeaderPair},
    test,
    test::TestRequest,
    Error,
};

use crate::http::services::auth_service::AuthService;

pub fn get_authorization_header(auth_service: Arc<dyn AuthService>) -> impl TryIntoHeaderPair {
    let token = auth_service
        .encode_token("b73ccd26-1832-4d10-9251-271ce453cee3".to_owned(), 10)
        .unwrap();

    (http::header::AUTHORIZATION, format!("Bearer {}", token))
}

pub struct HttpTestCase {
    pub expected_status: StatusCode,
    pub expected_body: &'static str,
}

impl HttpTestCase {
    pub async fn execute(
        &self,
        mut req: TestRequest,
        app: impl Service<Request, Response = ServiceResponse, Error = Error>,
        auth_service: Option<Arc<dyn AuthService>>,
    ) {
        if let Some(auth_service) = auth_service {
            req = req.append_header(get_authorization_header(auth_service));
        }

        let req = req.to_request();

        // Send the request to the app
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), self.expected_status);

        let body = test::read_body(resp).await;
        assert_eq!(body, self.expected_body);
    }
}
