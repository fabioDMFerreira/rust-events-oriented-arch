use argon2::password_hash::{rand_core::OsRng, PasswordHasher, SaltString};
use argon2::Argon2;
use async_trait::async_trait;
use log::error;
use mockall::automock;
use std::sync::Arc;
use uuid::Uuid;

use crate::error::CommonError;
use crate::models::user::User;
use crate::repositories::user_repository::UserRepository;

use super::event_service::EventService;

#[automock]
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
                    let err: CommonError = result.err().unwrap().into();
                    error!("failed sending user created event: {}", err);
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

#[cfg(test)]
mod tests {
    use crate::error::BrokerError;
    use crate::{
        error::RepositoryError, models::user::User,
        repositories::user_repository::MockUserRepository,
        services::event_service::MockEventService,
    };

    use super::*;
    use mockall::predicate::eq;
    use rstest::*;
    use std::str::FromStr;
    use uuid::Uuid;

    struct CreateUserTestCase {
        user: User,
        expected_result: Result<User, CommonError>,
        service_result: Result<User, RepositoryError>,
        broker_result: Result<(), BrokerError>,
    }

    #[rstest]
    #[case::success(CreateUserTestCase{
        user: User {
            id:
            Uuid::from_str("b73ccd26-1832-4d10-9251-271ce453cee3").unwrap(),
            name: "John Doe".to_string(),
            password: "1234".to_string(),
        },
        service_result: Ok(User {
            id:
            Uuid::from_str("b73ccd26-1832-4d10-9251-271ce453cee3").unwrap(),
            name: "John Doe".to_string(),
            password: "1234".to_string(),
        }),
        expected_result: Ok(User {
            id:
            Uuid::from_str("b73ccd26-1832-4d10-9251-271ce453cee3").unwrap(),
            name: "John Doe".to_string(),
            password: "1234".to_string(),
        }),
        broker_result: Ok(()),
    })]
    #[case::error_db(CreateUserTestCase{
        user: User {
            id:
            Uuid::from_str("b73ccd26-1832-4d10-9251-271ce453cee3").unwrap(),
            name: "John Doe".to_string(),
            password: "1234".to_string(),
        },
        service_result: Err(RepositoryError { message: "db is down".to_owned() }),
        expected_result: Err(CommonError { message: "db is down".to_owned(), code:1}),
        broker_result: Ok(()),
    })]
    #[case::success(CreateUserTestCase{
        user: User {
            id:
            Uuid::from_str("b73ccd26-1832-4d10-9251-271ce453cee3").unwrap(),
            name: "John Doe".to_string(),
            password: "1234".to_string(),
        },
        service_result: Ok(User {
            id:
            Uuid::from_str("b73ccd26-1832-4d10-9251-271ce453cee3").unwrap(),
            name: "John Doe".to_string(),
            password: "1234".to_string(),
        }),
        expected_result: Ok(User {
            id:
            Uuid::from_str("b73ccd26-1832-4d10-9251-271ce453cee3").unwrap(),
            name: "John Doe".to_string(),
            password: "1234".to_string(),
        }),
        broker_result: Ok(()),
    })]
    #[case::error_broker(CreateUserTestCase{
        user: User {
            id:
            Uuid::from_str("b73ccd26-1832-4d10-9251-271ce453cee3").unwrap(),
            name: "John Doe".to_string(),
            password: "1234".to_string(),
        },
        service_result: Ok(User {
            id:
            Uuid::from_str("b73ccd26-1832-4d10-9251-271ce453cee3").unwrap(),
            name: "John Doe".to_string(),
            password: "1234".to_string(),
        }),
        expected_result: Ok(User {
            id:
            Uuid::from_str("b73ccd26-1832-4d10-9251-271ce453cee3").unwrap(),
            name: "John Doe".to_string(),
            password: "1234".to_string(),
        }),
        broker_result: Err(BrokerError { message: "server is down".to_string() }),
    })]
    #[tokio::test]
    async fn test_create_user(#[case] case: CreateUserTestCase) {
        // Create mocks for UserRepository and EventService
        let mut repo_mock = MockUserRepository::new();
        let mut event_mock = MockEventService::new();

        repo_mock
            .expect_create()
            .returning(move |_, _| case.service_result.clone());
        event_mock
            .expect_user_created()
            .returning(move |_| case.broker_result.clone());

        // Create the UserServiceImpl with the mocks
        let service = UserServiceImpl::new(Arc::new(repo_mock), Arc::new(event_mock));

        // Call the create method and check the result
        let result = service
            .create(case.user.name.to_string(), case.user.password.to_string())
            .await;
        assert_eq!(result, case.expected_result);
    }

    struct ListUsersTestCase {
        expected_result: Result<Vec<User>, CommonError>,
        service_result: Result<Vec<User>, RepositoryError>,
    }

    #[rstest]
    #[case::success(ListUsersTestCase{
        expected_result: Ok(vec![User {
            id:
            Uuid::from_str("b73ccd26-1832-4d10-9251-271ce453cee3").unwrap(),
            name: "John Doe".to_string(),
            password: "1234".to_string(),
        }]),
        service_result: Ok(vec![User {
            id: Uuid::from_str("b73ccd26-1832-4d10-9251-271ce453cee3").unwrap(),
            name: "John Doe".to_string(),
            password: "1234".to_string(),
        }]),
    })]
    #[case::error(ListUsersTestCase{
        expected_result: Err(CommonError { message: "db is down".to_owned(), code:1 }),
        service_result: Err(RepositoryError { message: "db is down".to_owned() }),
    })]
    #[tokio::test]
    async fn test_list_users(#[case] case: ListUsersTestCase) {
        // Create mocks for UserRepository and EventService
        let mut repo_mock = MockUserRepository::new();
        let event_mock = MockEventService::new();

        // Set up expected behavior for the mocks

        let service_result = case.service_result.clone();
        repo_mock
            .expect_list()
            .returning(move || service_result.clone());

        // Create the UserServiceImpl with the mocks
        let service = UserServiceImpl::new(Arc::new(repo_mock), Arc::new(event_mock));

        // Call the create method and check the result
        let result = service.list().await;
        assert_eq!(result, case.expected_result);
    }

    struct GetUserByIdTestCase {
        id: uuid::Uuid,
        expected_result: Result<User, CommonError>,
        service_result: Result<User, RepositoryError>,
    }

    #[rstest]
    #[case::success(GetUserByIdTestCase{
        id:     Uuid::from_str("b73ccd26-1832-4d10-9251-271ce453cee3").unwrap(),
        expected_result: Ok(User {
            id:
            Uuid::from_str("b73ccd26-1832-4d10-9251-271ce453cee3").unwrap(),
            name: "John Doe".to_string(),
            password: "1234".to_string(),
        }),
        service_result: Ok(User {
            id: Uuid::from_str("b73ccd26-1832-4d10-9251-271ce453cee3").unwrap(),
            name: "John Doe".to_string(),
            password: "1234".to_string(),
        }),
    })]
    #[case::error(GetUserByIdTestCase{
        id:     Uuid::from_str("b73ccd26-1832-4d10-9251-271ce453cee3").unwrap(),
        expected_result: Err(CommonError { message: "db is down".to_owned(), code:1 }),
        service_result: Err(RepositoryError { message: "db is down".to_owned() }),
    })]
    #[tokio::test]
    async fn test_get_user_by_id(#[case] case: GetUserByIdTestCase) {
        // Create mocks for UserRepository and EventService
        let mut repo_mock = MockUserRepository::new();
        let event_mock = MockEventService::new();

        // Set up expected behavior for the mocks

        let service_result = case.service_result.clone();
        repo_mock
            .expect_get_by_id()
            .with(eq(case.id))
            .returning(move |_| service_result.clone());

        // Create the UserServiceImpl with the mocks
        let service = UserServiceImpl::new(Arc::new(repo_mock), Arc::new(event_mock));

        // Call the create method and check the result
        let result = service.get_by_id(case.id).await;
        assert_eq!(result, case.expected_result);
    }

    struct GetUserByNameTestCase {
        name: String,
        expected_result: Result<User, CommonError>,
        service_result: Result<User, RepositoryError>,
    }

    #[rstest]
    #[case::success(GetUserByNameTestCase{
        name:     "John".to_owned(),
        expected_result: Ok(User {
            id:
            Uuid::from_str("b73ccd26-1832-4d10-9251-271ce453cee3").unwrap(),
            name: "John Doe".to_string(),
            password: "1234".to_string(),
        }),
        service_result: Ok(User {
            id: Uuid::from_str("b73ccd26-1832-4d10-9251-271ce453cee3").unwrap(),
            name: "John Doe".to_string(),
            password: "1234".to_string(),
        }),
    })]
    #[case::error(GetUserByNameTestCase{
        name:    "John".to_owned(),
        expected_result: Err(CommonError { message: "db is down".to_owned(), code:1 }),
        service_result: Err(RepositoryError { message: "db is down".to_owned() }),
    })]
    #[tokio::test]
    async fn test_get_user_by_name(#[case] case: GetUserByNameTestCase) {
        // Create mocks for UserRepository and EventService
        let mut repo_mock = MockUserRepository::new();
        let event_mock = MockEventService::new();

        // Set up expected behavior for the mocks

        let service_result = case.service_result.clone();
        repo_mock
            .expect_get_by_name()
            .with(eq(case.name.clone()))
            .returning(move |_| service_result.clone());

        // Create the UserServiceImpl with the mocks
        let service = UserServiceImpl::new(Arc::new(repo_mock), Arc::new(event_mock));

        // Call the create method and check the result
        let result = service.get_by_name(case.name).await;
        assert_eq!(result, case.expected_result);
    }

    struct UpdateUserTestCase {
        id: uuid::Uuid,
        update: String,
        expected_result: Result<User, CommonError>,
        service_result: Result<User, RepositoryError>,
    }

    #[rstest]
    #[case::success(UpdateUserTestCase{
        id:      Uuid::from_str("b73ccd26-1832-4d10-9251-271ce453cee3").unwrap(),
        update: "John".to_owned(),
        expected_result: Ok(User {
            id:
            Uuid::from_str("b73ccd26-1832-4d10-9251-271ce453cee3").unwrap(),
            name: "John Doe".to_string(),
            password: "1234".to_string(),
        }),
        service_result: Ok(User {
            id: Uuid::from_str("b73ccd26-1832-4d10-9251-271ce453cee3").unwrap(),
            name: "John Doe".to_string(),
            password: "1234".to_string(),
        }),
    })]
    #[case::error(UpdateUserTestCase{
        id:     Uuid::from_str("b73ccd26-1832-4d10-9251-271ce453cee3").unwrap(),
        update: "John".to_owned(),
        expected_result: Err(CommonError { message: "db is down".to_owned(), code:1 }),
        service_result: Err(RepositoryError { message: "db is down".to_owned() }),
    })]
    #[tokio::test]
    async fn test_update_user(#[case] case: UpdateUserTestCase) {
        // Create mocks for UserRepository and EventService
        let mut repo_mock = MockUserRepository::new();
        let event_mock = MockEventService::new();

        // Set up expected behavior for the mocks

        let service_result = case.service_result.clone();
        repo_mock
            .expect_update()
            .with(eq(case.id.clone()), eq(case.update.clone()))
            .returning(move |_, _| service_result.clone());

        // Create the UserServiceImpl with the mocks
        let service = UserServiceImpl::new(Arc::new(repo_mock), Arc::new(event_mock));

        // Call the create method and check the result
        let result = service.update(case.id, case.update).await;
        assert_eq!(result, case.expected_result);
    }

    struct DeleteUserTestCase {
        id: uuid::Uuid,
        expected_result: Result<usize, CommonError>,
        service_result: Result<usize, RepositoryError>,
    }

    #[rstest]
    #[case::success(DeleteUserTestCase{
        id:      Uuid::from_str("b73ccd26-1832-4d10-9251-271ce453cee3").unwrap(),
        expected_result: Ok(1),
        service_result: Ok(1),
    })]
    #[case::error(DeleteUserTestCase{
        id:     Uuid::from_str("b73ccd26-1832-4d10-9251-271ce453cee3").unwrap(),
        expected_result: Err(CommonError { message: "db is down".to_owned(), code:1 }),
        service_result: Err(RepositoryError { message: "db is down".to_owned() }),
    })]
    #[tokio::test]
    async fn test_delete_user(#[case] case: DeleteUserTestCase) {
        // Create mocks for UserRepository and EventService
        let mut repo_mock = MockUserRepository::new();
        let event_mock = MockEventService::new();

        // Set up expected behavior for the mocks

        let service_result = case.service_result.clone();
        repo_mock
            .expect_delete()
            .with(eq(case.id.clone()))
            .returning(move |_| service_result.clone());

        // Create the UserServiceImpl with the mocks
        let service = UserServiceImpl::new(Arc::new(repo_mock), Arc::new(event_mock));

        // Call the create method and check the result
        let result = service.delete(case.id).await;
        assert_eq!(result, case.expected_result);
    }
}
