use actix::Actor;
use actix_web::http::StatusCode;
use actix_web::test;
use serde::Deserialize;
use serde_json::from_slice;
use users::app::setup_app;
use users::config::Config;
use users::handlers::user::CreateUserPayload;
use utils::http::websockets::ws_server::WebsocketServer;

#[derive(Debug, Deserialize, PartialEq, Clone)]
struct UserResponse {
    pub name: String,
    pub id: String,
}

type UsersListResponse = Vec<UserResponse>;

#[actix_rt::test]
async fn user_create() {
    let config = Config::init();

    let ws_server = WebsocketServer::new().start();

    let app = setup_app(&config, ws_server.clone());

    let mut app_server = test::init_service(app).await;

    let req = test::TestRequest::post()
        .uri("/users")
        .insert_header(("content-type", "application/json"))
        .set_payload(
            serde_json::to_vec(&CreateUserPayload {
                name: Some("Alice".to_owned()),
                password: Some("1234567".to_owned()),
            })
            .unwrap(),
        )
        .to_request();
    let resp = test::call_service(&mut app_server, req).await;

    assert_eq!(resp.status(), StatusCode::OK);

    let body = test::read_body(resp).await;

    let result: Result<UserResponse, _> = serde_json::from_slice(&body);
    assert!(result.is_ok());
    let user = result.unwrap();
    assert_eq!(user.name, String::from("Alice"));
    assert_eq!(user.id.len(), 36);

    // fail creating user that already exists
    let req = test::TestRequest::post()
        .uri("/users")
        .insert_header(("content-type", "application/json"))
        .set_payload(
            serde_json::to_vec(&CreateUserPayload {
                name: Some("Alice".to_owned()),
                password: Some("1234567".to_owned()),
            })
            .unwrap(),
        )
        .to_request();
    let resp = test::call_service(&mut app_server, req).await;

    assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let body = test::read_body(resp).await;
    assert_eq!(body, "");

    // get list of users
    let req = test::TestRequest::get().uri("/users").to_request();
    let resp = test::call_service(&mut app_server, req).await;

    assert_eq!(resp.status(), StatusCode::OK);
    let body = test::read_body(resp).await;
    let result: UsersListResponse = from_slice(&body).unwrap();
    assert_eq!(result, vec![user.clone()]);

    // delete user
    let req = test::TestRequest::delete()
        .uri(format!("/users/{}", user.id).as_str())
        .to_request();
    let resp = test::call_service(&mut app_server, req).await;

    assert_eq!(resp.status(), StatusCode::OK);
    let body = test::read_body(resp).await;
    assert_eq!(body, "");

    // get empty list of users
    let req = test::TestRequest::get().uri("/users").to_request();
    let resp = test::call_service(&mut app_server, req).await;

    assert_eq!(resp.status(), StatusCode::OK);
    let body = test::read_body(resp).await;
    let result: UsersListResponse = from_slice(&body).unwrap();
    assert_eq!(result, vec![]);
}
