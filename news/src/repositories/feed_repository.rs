use std::sync::Arc;

use diesel::prelude::*;
use utils::db::PgPool;
use uuid::Uuid;

use crate::models::feed::Feed;
use crate::schema::feeds;

#[derive(Clone)]
pub struct FeedRepository {
    pool: Arc<PgPool>,
}

impl FeedRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        FeedRepository { pool }
    }

    pub fn create(&self, rss_feed: &Feed) -> Result<Feed, diesel::result::Error> {
        let mut conn = self.pool.get().unwrap();

        diesel::insert_into(feeds::table)
            .values(rss_feed)
            .get_result(&mut conn)
    }

    pub fn find_by_id(&self, feed_id: Uuid) -> Result<Option<Feed>, diesel::result::Error> {
        let mut conn = self.pool.get().unwrap();

        feeds::table
            .filter(feeds::id.eq(feed_id))
            .first(&mut conn)
            .optional()
    }

    pub fn list(&self) -> Result<Vec<Feed>, diesel::result::Error> {
        let mut conn = self.pool.get().unwrap();

        feeds::table.load::<Feed>(&mut conn)
    }

    pub fn delete(&self, feed_id: Uuid) -> Result<usize, diesel::result::Error> {
        let mut conn = self.pool.get().unwrap();

        diesel::delete(feeds::table.find(feed_id)).execute(&mut conn)
    }
}
