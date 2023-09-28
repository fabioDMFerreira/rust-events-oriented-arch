use std::sync::Arc;

use crate::db::PgPool;
use crate::error::DatabaseError;
use diesel::prelude::*;
use mockall::automock;
use uuid::Uuid;

use crate::news::models::feed::Feed;
use crate::news::schema::feeds;

#[automock]
pub trait FeedRepository: Send + Sync {
    fn create(&self, rss_feed: &Feed) -> Result<Feed, DatabaseError>;
    fn list(&self) -> Result<Vec<Feed>, DatabaseError>;
    fn find_by_id(&self, feed_id: Uuid) -> Result<Option<Feed>, DatabaseError>;
    fn delete(&self, feed_id: Uuid) -> Result<usize, DatabaseError>;
}

#[derive(Clone)]
pub struct FeedDieselRepository {
    pool: Arc<PgPool>,
}

impl FeedDieselRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        FeedDieselRepository { pool }
    }
}

impl FeedRepository for FeedDieselRepository {
    fn create(&self, rss_feed: &Feed) -> Result<Feed, DatabaseError> {
        let mut conn = self.pool.get().unwrap();

        diesel::insert_into(feeds::table)
            .values(rss_feed)
            .get_result(&mut conn)
            .map_err(|err| DatabaseError {
                message: err.to_string(),
            })
    }

    fn find_by_id(&self, feed_id: Uuid) -> Result<Option<Feed>, DatabaseError> {
        let mut conn = self.pool.get().unwrap();

        feeds::table
            .filter(feeds::id.eq(feed_id))
            .first(&mut conn)
            .optional()
            .map_err(|err| DatabaseError {
                message: err.to_string(),
            })
    }

    fn list(&self) -> Result<Vec<Feed>, DatabaseError> {
        let mut conn = self.pool.get().unwrap();

        feeds::table
            .load::<Feed>(&mut conn)
            .map_err(|err| DatabaseError {
                message: err.to_string(),
            })
    }

    fn delete(&self, feed_id: Uuid) -> Result<usize, DatabaseError> {
        let mut conn = self.pool.get().unwrap();

        diesel::delete(feeds::table.find(feed_id))
            .execute(&mut conn)
            .map_err(|err| DatabaseError {
                message: err.to_string(),
            })
    }
}
