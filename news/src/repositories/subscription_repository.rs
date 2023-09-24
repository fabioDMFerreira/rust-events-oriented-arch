use std::sync::Arc;

use diesel::prelude::*;
use mockall::automock;
use utils::db::PgPool;
use utils::error::DatabaseError;
use uuid::Uuid;

use crate::models::subscription::Subscription;
use crate::schema::subscriptions;

#[automock]
pub trait SubscriptionRepository: Send + Sync {
    fn list_by_user(&self, user_id: Uuid) -> Result<Vec<Subscription>, DatabaseError>;
    fn find_by_id(
        &self,
        feed_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<Subscription>, DatabaseError>;
    fn create(&self, subscription: &Subscription) -> Result<Subscription, DatabaseError>;
    fn delete(&self, feed_id: Uuid, user_id: Uuid) -> Result<usize, DatabaseError>;
}

pub struct SubscriptionsDieselRepository {
    pool: Arc<PgPool>,
}

impl SubscriptionsDieselRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        SubscriptionsDieselRepository { pool }
    }
}

impl SubscriptionRepository for SubscriptionsDieselRepository {
    fn create(&self, subscription: &Subscription) -> Result<Subscription, DatabaseError> {
        let mut conn = self.pool.get().unwrap();

        diesel::insert_into(subscriptions::table)
            .values(subscription)
            .get_result(&mut conn)
            .map_err(|err| DatabaseError {
                message: err.to_string(),
            })
    }

    fn find_by_id(
        &self,
        feed_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<Subscription>, DatabaseError> {
        let mut conn = self.pool.get().unwrap();

        subscriptions::table
            .filter(
                subscriptions::feed_id
                    .eq(feed_id)
                    .and(subscriptions::user_id.eq(user_id)),
            )
            .first(&mut conn)
            .optional()
            .map_err(|err| DatabaseError {
                message: err.to_string(),
            })
    }

    fn list_by_user(&self, user_id: Uuid) -> Result<Vec<Subscription>, DatabaseError> {
        let mut conn = self.pool.get().unwrap();

        subscriptions::table
            .filter(subscriptions::user_id.eq(user_id))
            .load::<Subscription>(&mut conn)
            .map_err(|err| DatabaseError {
                message: err.to_string(),
            })
    }

    fn delete(&self, feed_id: Uuid, user_id: Uuid) -> Result<usize, DatabaseError> {
        let mut conn = self.pool.get().unwrap();

        diesel::delete(
            subscriptions::table.filter(
                subscriptions::feed_id
                    .eq(feed_id)
                    .and(subscriptions::user_id.eq(user_id)),
            ),
        )
        .execute(&mut conn)
        .map_err(|err| DatabaseError {
            message: err.to_string(),
        })
    }
}
