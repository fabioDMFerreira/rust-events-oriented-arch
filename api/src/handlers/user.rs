use actix_web::{delete, get, post, put, web, HttpResponse};
use log::error;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::services::user_service::UserService;

#[derive(Debug, Serialize, Deserialize)]
struct CreateUserPayload {
    name: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct UpdateUserPayload {
    name: String,
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
    payload: web::Json<CreateUserPayload>,
) -> HttpResponse {
    let CreateUserPayload { name, password } = payload.into_inner();

    match user_service.create(name, password).await {
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
    payload: web::Json<UpdateUserPayload>,
    id: web::Path<Uuid>,
) -> HttpResponse {
    let name = payload.into_inner().name;

    match user_service.update(id.into_inner(), name).await {
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
        Ok(_) => return HttpResponse::NoContent().finish(),
    }
}
