use actix_web::{delete, get, post, put, web, HttpResponse};
use log::error;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::services::user_service::UserService;

#[derive(Debug, Serialize, Deserialize, Validate)]
struct CreateUserPayload {
    #[validate(required, length(min = 2))]
    name: Option<String>,
    #[validate(required, length(min = 6))]
    password: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
struct UpdateUserPayload {
    #[validate(required, length(min = 2))]
    name: Option<String>,
}

#[get("/users")]
async fn get_users(user_service: web::Data<dyn UserService>) -> HttpResponse {
    let result = user_service.list().await;

    match result {
        Err(err) => {
            error!("failed getting users: {}", err);
            return HttpResponse::InternalServerError().finish();
        }
        Ok(users) => return HttpResponse::Ok().json(users),
    };
}

#[get("/users/{id}")]
async fn get_user_by_id(
    user_service: web::Data<dyn UserService>,
    id: web::Path<Uuid>,
) -> HttpResponse {
    let result = user_service.get_by_id(id.into_inner()).await;

    match result {
        Err(err) => {
            error!("failed getting user: {}", err);
            return HttpResponse::InternalServerError().finish();
        }
        Ok(user) => return HttpResponse::Ok().json(user),
    }
}

#[post("/users")]
async fn create_user(
    user_service: web::Data<dyn UserService>,
    payload: Option<web::Json<CreateUserPayload>>,
) -> HttpResponse {
    if let None = payload {
        return HttpResponse::BadRequest().body("empty body");
    }

    let payload = payload.unwrap();
    if let Err(err) = payload.validate() {
        return HttpResponse::BadRequest().json(err);
    }

    let CreateUserPayload { name, password } = payload.into_inner();

    match user_service.create(name.unwrap(), password.unwrap()).await {
        Err(err) => {
            error!("failed creating user: {}", err);
            return HttpResponse::InternalServerError().finish();
        }
        Ok(new_user) => return HttpResponse::Ok().json(new_user),
    }
}

#[put("/users/{id}")]
async fn update_user(
    user_service: web::Data<dyn UserService>,
    payload: Option<web::Json<UpdateUserPayload>>,
    id: web::Path<Uuid>,
) -> HttpResponse {
    if let None = payload {
        return HttpResponse::BadRequest().body("empty body");
    }

    let payload = payload.unwrap();
    if let Err(err) = payload.validate() {
        return HttpResponse::BadRequest().json(err);
    }

    let name = payload.into_inner().name;

    match user_service.update(id.into_inner(), name.unwrap()).await {
        Err(err) => {
            error!("failed updating user: {}", err);
            return HttpResponse::InternalServerError().finish();
        }
        Ok(updated_user) => return HttpResponse::Ok().json(updated_user),
    }
}

#[delete("/users/{id}")]
async fn delete_user(
    user_service: web::Data<dyn UserService>,
    id: web::Path<Uuid>,
) -> HttpResponse {
    match user_service.delete(id.into_inner()).await {
        Err(err) => {
            error!("failed deleting user: {}", err);
            return HttpResponse::InternalServerError().finish();
        }
        Ok(_) => return HttpResponse::Ok().finish(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::http::StatusCode;
    use actix_web::{test, web, App};
    use mockall::predicate::eq;
    use rstest::*;
    use serde_json;
    use std::str::FromStr;
    use std::sync::Arc;
    use uuid::Uuid;

    use crate::error::CommonError;
    use crate::models::user::User;
    use crate::services::user_service::MockUserService;

    struct ListUsersTestCase {
        expected_status: StatusCode,
        expected_body: &'static str,
        service_result: Result<Vec<User>, CommonError>,
    }

    #[rstest]
    #[case(ListUsersTestCase {
        expected_status: StatusCode::OK,
        expected_body: r#"[{"id":"b73ccd26-1832-4d10-9251-271ce453cee3","name":"Alice"},{"id":"fdd0a6f3-af61-4760-a789-5b6dd16eb7dc","name":"Bob"}]"#,
        service_result: Ok(vec![
            User {
                id: Uuid::from_str("b73ccd26-1832-4d10-9251-271ce453cee3").unwrap(),
                name: "Alice".to_owned(),
                password: "1234".to_owned(),
            },
            User {
                id: Uuid::from_str("fdd0a6f3-af61-4760-a789-5b6dd16eb7dc").unwrap(),
                name: "Bob".to_owned(),
                password: "1234".to_owned(),
            },
        ]),
    })]
    #[case(ListUsersTestCase {
        expected_status: StatusCode::INTERNAL_SERVER_ERROR,
        expected_body: "",
        service_result: Err(CommonError {
            message: "db is down".to_owned(),
            code: 1,
        }),
    })]
    #[actix_rt::test]
    async fn test_get_users(#[case] case: ListUsersTestCase) {
        let mut user_service = MockUserService::new();

        user_service
            .expect_list()
            .returning(move || case.service_result.clone());

        let user_service: Arc<dyn UserService> = Arc::new(user_service);

        let mut app = test::init_service(
            App::new()
                .app_data(web::Data::from(user_service.clone()))
                .service(get_users),
        )
        .await;

        let req = test::TestRequest::get().uri("/users").to_request();
        let resp = test::call_service(&mut app, req).await;

        assert_eq!(resp.status(), case.expected_status);

        let body = test::read_body(resp).await;
        assert_eq!(body, case.expected_body);
    }

    struct GetUserTestCase {
        id: Uuid,
        expected_status: StatusCode,
        expected_body: &'static str,
        service_result: Result<User, CommonError>,
    }

    #[rstest]
    #[case(GetUserTestCase {
        id: Uuid::from_str("b73ccd26-1832-4d10-9251-271ce453cee3").unwrap(),
        expected_status: StatusCode::OK,
        expected_body: r#"{"id":"b73ccd26-1832-4d10-9251-271ce453cee3","name":"Alice"}"#,
        service_result: Ok(
            User {
                id: Uuid::from_str("b73ccd26-1832-4d10-9251-271ce453cee3").unwrap(),
                name: "Alice".to_owned(),
                password: "1234".to_owned(),
            },
        ),
    })]
    #[case(GetUserTestCase {
        id: Uuid::from_str("b73ccd26-1832-4d10-9251-271ce453cee3").unwrap(),
        expected_status: StatusCode::INTERNAL_SERVER_ERROR,
        expected_body: "",
        service_result: Err(CommonError {
            message: "db is down".to_owned(),
            code: 1,
        }),
    })]
    #[actix_rt::test]
    async fn test_get_user_by_id(#[case] case: GetUserTestCase) {
        let mut user_service = MockUserService::new();

        user_service
            .expect_get_by_id()
            .with(eq(case.id))
            .times(1)
            .returning(move |_| case.service_result.clone());

        let user_service: Arc<dyn UserService> = Arc::new(user_service);

        let mut app = test::init_service(
            App::new()
                .app_data(web::Data::from(user_service.clone()))
                .service(get_user_by_id),
        )
        .await;

        let req = test::TestRequest::get()
            .uri(format!("/users/{}", case.id).as_str())
            .to_request();
        let resp = test::call_service(&mut app, req).await;

        assert_eq!(resp.status(), case.expected_status);

        let body = test::read_body(resp).await;
        assert_eq!(body, case.expected_body);
    }

    struct CreateUserTestCase {
        payload: Option<CreateUserPayload>,
        expected_status: StatusCode,
        expected_body: &'static str,
        service_result: Option<Result<User, CommonError>>,
    }

    #[rstest]
    #[case::success(CreateUserTestCase {
        payload: Some(CreateUserPayload { name: Some("Alice".to_owned()), password: Some("1234567".to_owned()) }),
        expected_status: StatusCode::OK,
        expected_body: r#"{"id":"b73ccd26-1832-4d10-9251-271ce453cee3","name":"Alice"}"#,
        service_result: Some(Ok(
            User {
                id: Uuid::from_str("b73ccd26-1832-4d10-9251-271ce453cee3").unwrap(),
                name: "Alice".to_owned(),
                password: "1234".to_owned(),
            },
        )),
    })]
    #[case::error_db(CreateUserTestCase {
        payload: Some(CreateUserPayload { name: Some("Alice".to_owned()), password: Some("1234567".to_owned()) }),
        expected_status: StatusCode::INTERNAL_SERVER_ERROR,
        expected_body: r#""#,
        service_result: Some(Err(CommonError{
            code:1,
            message: "db is down".to_owned(),
    })),
    })]
    #[case::no_name(CreateUserTestCase {
        payload: Some(CreateUserPayload { name: None, password: Some("1234567".to_owned()) }),
        expected_status: StatusCode::BAD_REQUEST,
        expected_body: r#"{"name":[{"code":"required","message":null,"params":{"value":null}}]}"#,
        service_result: None,
    })]
    #[case::no_password(CreateUserTestCase {
        payload: Some(CreateUserPayload { name: Some("Bob".to_owned()), password: None }),
        expected_status: StatusCode::BAD_REQUEST,
        expected_body: r#"{"password":[{"code":"required","message":null,"params":{"value":null}}]}"#,
        service_result: None,
    })]
    #[case::no_payload(CreateUserTestCase {
        payload: None,
        expected_status: StatusCode::BAD_REQUEST,
        expected_body: r#"empty body"#,
        service_result: None,
    })]
    #[actix_rt::test]
    async fn test_create_user(#[case] case: CreateUserTestCase) {
        let mut user_service = MockUserService::new();

        if let Some(result) = case.service_result {
            user_service
                .expect_create()
                .times(1)
                .returning(move |_, _| result.clone());
        }

        let user_service: Arc<dyn UserService> = Arc::new(user_service);

        let mut app = test::init_service(
            App::new()
                .app_data(web::Data::from(user_service.clone()))
                .service(create_user),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/users")
            .insert_header(("content-type", "application/json"))
            .set_payload(serde_json::to_vec(&case.payload).unwrap())
            .to_request();
        let resp = test::call_service(&mut app, req).await;

        assert_eq!(resp.status(), case.expected_status);

        let body = test::read_body(resp).await;
        assert_eq!(body, case.expected_body);
    }

    struct UpdateUserTestCase {
        id: Uuid,
        payload: Option<UpdateUserPayload>,
        expected_status: StatusCode,
        expected_body: &'static str,
        service_result: Option<Result<User, CommonError>>,
    }

    #[rstest]
    #[case::success(UpdateUserTestCase {
        id: Uuid::from_str("b73ccd26-1832-4d10-9251-271ce453cee3").unwrap(),
        payload: Some(UpdateUserPayload { name: Some("Alice".to_owned()) }),
        expected_status: StatusCode::OK,
        expected_body: r#"{"id":"b73ccd26-1832-4d10-9251-271ce453cee3","name":"Alice"}"#,
        service_result: Some(Ok(
            User {
                id: Uuid::from_str("b73ccd26-1832-4d10-9251-271ce453cee3").unwrap(),
                name: "Alice".to_owned(),
                password: "1234".to_owned(),
            },
        )),
    })]
    #[case::error_db(UpdateUserTestCase {
        id: Uuid::from_str("b73ccd26-1832-4d10-9251-271ce453cee3").unwrap(),
        payload: Some(UpdateUserPayload { name: Some("Alice".to_owned()) }),
        expected_status: StatusCode::INTERNAL_SERVER_ERROR,
        expected_body: r#""#,
        service_result: Some(Err(CommonError{
            code:1,
            message: "db is down".to_owned(),
    })),
    })]
    #[case::no_name(UpdateUserTestCase {
        id: Uuid::from_str("b73ccd26-1832-4d10-9251-271ce453cee3").unwrap(),
        payload: Some(UpdateUserPayload { name: None }),
        expected_status: StatusCode::BAD_REQUEST,
        expected_body: r#"{"name":[{"code":"required","message":null,"params":{"value":null}}]}"#,
        service_result: None,
    })]
    #[case::no_content(UpdateUserTestCase {
        id: Uuid::from_str("b73ccd26-1832-4d10-9251-271ce453cee3").unwrap(),
        payload: None,
        expected_status: StatusCode::BAD_REQUEST,
        expected_body: r#"empty body"#,
        service_result: None,
    })]
    #[actix_rt::test]
    async fn test_update_user(#[case] case: UpdateUserTestCase) {
        let mut user_service = MockUserService::new();

        if let Some(result) = case.service_result {
            user_service
                .expect_update()
                .times(1)
                .returning(move |_, _| result.clone());
        }

        let user_service: Arc<dyn UserService> = Arc::new(user_service);

        let mut app = test::init_service(
            App::new()
                .app_data(web::Data::from(user_service.clone()))
                .service(update_user),
        )
        .await;

        let req = test::TestRequest::put()
            .uri(format!("/users/{}", case.id).as_str())
            .insert_header(("content-type", "application/json"))
            .set_payload(serde_json::to_vec(&case.payload).unwrap())
            .to_request();
        let resp = test::call_service(&mut app, req).await;

        assert_eq!(resp.status(), case.expected_status);

        let body = test::read_body(resp).await;
        assert_eq!(body, case.expected_body);
    }

    struct DeleteUserTestCase {
        id: Uuid,
        expected_status: StatusCode,
        expected_body: &'static str,
        service_result: Result<usize, CommonError>,
    }

    #[rstest]
    #[case(DeleteUserTestCase {
        id: Uuid::from_str("b73ccd26-1832-4d10-9251-271ce453cee3").unwrap(),
        expected_status: StatusCode::OK,
        expected_body: r#""#,
        service_result: Ok(1),
    })]
    #[case(DeleteUserTestCase {
        id: Uuid::from_str("b73ccd26-1832-4d10-9251-271ce453cee3").unwrap(),
        expected_status: StatusCode::INTERNAL_SERVER_ERROR,
        expected_body: "",
        service_result: Err(CommonError {
            message: "db is down".to_owned(),
            code: 1,
        }),
    })]
    #[actix_rt::test]
    async fn test_delete_user_by_id(#[case] case: DeleteUserTestCase) {
        let mut user_service = MockUserService::new();

        user_service
            .expect_delete()
            .with(eq(case.id))
            .times(1)
            .returning(move |_| case.service_result.clone());

        let user_service: Arc<dyn UserService> = Arc::new(user_service);

        let mut app = test::init_service(
            App::new()
                .app_data(web::Data::from(user_service.clone()))
                .service(delete_user),
        )
        .await;

        let req = test::TestRequest::delete()
            .uri(format!("/users/{}", case.id).as_str())
            .to_request();
        let resp = test::call_service(&mut app, req).await;

        assert_eq!(resp.status(), case.expected_status);

        let body = test::read_body(resp).await;
        assert_eq!(body, case.expected_body);
    }
}
