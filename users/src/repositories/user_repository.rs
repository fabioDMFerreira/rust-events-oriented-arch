use std::sync::Arc;

use crate::error::DieselRepositoryError;
use crate::models::user::User;
use crate::schema::users;
use actix_threadpool::run;
use async_trait::async_trait;
use diesel::prelude::*;
use mockall::automock;
use utils::db::PgPool;
use utils::error::DatabaseError;
use uuid::Uuid;

#[automock]
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(&self, name: String, password: String) -> Result<User, DatabaseError>;
    async fn list(&self) -> Result<Vec<User>, DatabaseError>;
    async fn get_by_id(&self, user_id: Uuid) -> Result<User, DatabaseError>;
    async fn get_by_name(&self, name: String) -> Result<User, DatabaseError>;
    async fn update(&self, user_id: Uuid, name: String) -> Result<User, DatabaseError>;
    async fn delete(&self, user_id: Uuid) -> Result<usize, DatabaseError>;
}

pub struct UserDieselRepository {
    pool: Arc<PgPool>,
}

impl UserDieselRepository {
    pub fn new(db: Arc<PgPool>) -> Self {
        UserDieselRepository { pool: db }
    }
}

#[async_trait]
impl UserRepository for UserDieselRepository {
    async fn create(&self, name: String, password: String) -> Result<User, DatabaseError> {
        let new_user = User {
            id: Uuid::new_v4(),
            name,
            password,
        };

        let pool = self.pool.clone();

        let inserted_user = run(move || {
            let mut conn = pool.get().unwrap();

            diesel::insert_into(users::table)
                .values(new_user)
                .get_result(&mut conn)
        })
        .await
        .map_err(|v| DieselRepositoryError::from(v).into_inner())?;

        Ok(inserted_user)
    }

    async fn list(&self) -> Result<Vec<User>, DatabaseError> {
        let pool = self.pool.clone();

        let users = run(move || {
            let mut conn = pool.get().unwrap();

            users::table.load::<User>(&mut conn)
        })
        .await
        .map_err(|v| DieselRepositoryError::from(v).into_inner())?;

        Ok(users)
    }

    async fn get_by_id(&self, user_id: Uuid) -> Result<User, DatabaseError> {
        let pool = self.pool.clone();

        let user = run(move || {
            let mut conn = pool.get().unwrap();

            users::table.filter(users::id.eq(user_id)).first(&mut conn)
        })
        .await
        .map_err(|v| DieselRepositoryError::from(v).into_inner())?;

        Ok(user)
    }

    async fn get_by_name(&self, name: String) -> Result<User, DatabaseError> {
        let pool = self.pool.clone();

        let user = run(move || {
            let mut conn = pool.get().unwrap();

            users::table.filter(users::name.eq(name)).first(&mut conn)
        })
        .await
        .map_err(|v| DieselRepositoryError::from(v).into_inner())?;

        Ok(user)
    }

    async fn update(&self, user_id: Uuid, name: String) -> Result<User, DatabaseError> {
        let pool = self.pool.clone();

        let user = run(move || {
            let mut conn = pool.get().unwrap();

            diesel::update(users::table.find(user_id))
                .set(users::name.eq(name))
                .get_result(&mut conn)
        })
        .await
        .map_err(|v| DieselRepositoryError::from(v).into_inner())?;

        Ok(user)
    }

    async fn delete(&self, user_id: Uuid) -> Result<usize, DatabaseError> {
        let pool = self.pool.clone();

        let user_id = run(move || {
            let mut conn = pool.get().unwrap();

            diesel::delete(users::table.find(user_id)).execute(&mut conn)
        })
        .await
        .map_err(|v| DieselRepositoryError::from(v).into_inner())?;

        Ok(user_id)
    }
}
