use argon2::password_hash::{rand_core::OsRng, PasswordHasher, SaltString};
use argon2::Argon2;
use async_trait::async_trait;
use log::error;
use std::sync::Arc;
use uuid::Uuid;

use crate::error::CommonError;
use crate::models::user::User;
use crate::repositories::user_repository::UserRepository;

use super::event_service::EventService;

#[async_trait]
pub trait UserService: Send + Sync {
    async fn create(&self, name: String, password: String) -> Result<User, CommonError>;
    async fn list(&self) -> Result<Vec<User>, CommonError>;
    async fn get_by_id(&self, user_id: Uuid) -> Result<User, CommonError>;
    async fn get_by_name(&self, name: String) -> Result<User, CommonError>;
    async fn update(&self, user_id: Uuid, name: String) -> Result<User, CommonError>;
    async fn delete(&self, user_id: Uuid) -> Result<usize, CommonError>;
}

pub struct UserServiceImpl {
    repo: Arc<dyn UserRepository>,
    broker: Arc<dyn EventService>,
}

impl UserServiceImpl {
    pub fn new(repo: Arc<dyn UserRepository>, broker: Arc<dyn EventService>) -> Self {
        UserServiceImpl {
            repo: repo,
            broker: broker,
        }
    }
}

#[async_trait]
impl UserService for UserServiceImpl {
    async fn create(&self, name: String, password: String) -> Result<User, CommonError> {
        let salt = SaltString::generate(&mut OsRng);
        let hashed_password = Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .expect("Error while hashing password")
            .to_string();

        let result = self.repo.create(name, hashed_password).await;

        match result {
            Ok(user) => {
                let result = self.broker.user_created(user.clone()).await;

                if result.is_err() {
                    error!(
                        "failed sending user created event: {}",
                        result.err().unwrap()
                    );
                }

                return Ok(user.clone());
            }
            Err(error) => {
                return Err(error.into());
            }
        }
    }

    async fn list(&self) -> Result<Vec<User>, CommonError> {
        self.repo
            .list()
            .await
            .map_err(|e| -> CommonError { e.into() })
    }

    async fn get_by_id(&self, user_id: Uuid) -> Result<User, CommonError> {
        self.repo
            .get_by_id(user_id)
            .await
            .map_err(|e| -> CommonError { e.into() })
    }

    async fn get_by_name(&self, name: String) -> Result<User, CommonError> {
        self.repo
            .get_by_name(name)
            .await
            .map_err(|e| -> CommonError { e.into() })
    }

    async fn update(&self, user_id: Uuid, name: String) -> Result<User, CommonError> {
        self.repo
            .update(user_id, name)
            .await
            .map_err(|e| -> CommonError { e.into() })
    }

    async fn delete(&self, user_id: Uuid) -> Result<usize, CommonError> {
        self.repo
            .delete(user_id)
            .await
            .map_err(|e| -> CommonError { e.into() })
    }
}
