use std::sync::Arc;

use api::{
    config::Config,
    repositories::user_repository::{UserDieselRepository, UserRepository},
};
use diesel::connection::SimpleConnection;
use utils::db;
use uuid::Uuid;

#[actix_rt::test]
async fn user_repo() {
    let config = Config::init();
    let db_connection = db::connect_db(config.database_url.clone());

    let pool = db_connection.get();
    if let Ok(mut pool) = pool {
        let result = pool.batch_execute("delete from users;");
        assert!(result.is_ok());
    }

    let user_repo = UserDieselRepository::new(Arc::new(db_connection));

    let result = user_repo
        .create(String::from("John"), String::from("123456"))
        .await;
    assert!(result.is_ok());
    let inserted_user = result.unwrap();
    assert_eq!(inserted_user.name, String::from("John"));
    assert_eq!(inserted_user.password, String::from("123456"));

    let result = user_repo.list().await;
    assert!(result.is_ok());
    if let Ok(users) = result {
        assert_eq!(users, vec![inserted_user.clone()])
    }

    let result = user_repo.get_by_id(inserted_user.id).await;
    assert!(result.is_ok());
    if let Ok(user) = result {
        assert_eq!(user, inserted_user.clone());
    }

    // user_id does not exist
    let result = user_repo.get_by_id(Uuid::new_v4()).await;
    assert_eq!(result.is_err(), true);
    if let Err(err) = result {
        assert_eq!(err.message, String::from("NotFound"));
    }

    let result = user_repo.get_by_name(inserted_user.name.clone()).await;
    assert!(result.is_ok());
    if let Ok(user) = result {
        assert_eq!(user, inserted_user.clone());
    }

    let result = user_repo.delete(inserted_user.id).await;
    assert!(result.is_ok());
    if let Ok(rows_affected) = result {
        assert_eq!(rows_affected, 1);
    }

    let result = user_repo.list().await;
    assert!(result.is_ok());
    if let Ok(users) = result {
        assert_eq!(users, vec![])
    }
}
