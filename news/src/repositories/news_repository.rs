use std::sync::Arc;

use diesel::prelude::*;
use utils::db::PgPool;
use uuid::Uuid;

use crate::models::news::News;
use crate::schema::news;

pub struct NewsRepository {
    pool: Arc<PgPool>,
}

impl NewsRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        NewsRepository { pool }
    }

    pub fn create(&self, news: &News) -> Result<News, diesel::result::Error> {
        let mut conn = self.pool.get().unwrap();

        diesel::insert_into(news::table)
            .values(news)
            .get_result(&mut conn)
    }

    pub fn find_by_id(&self, news_id: Uuid) -> Result<Option<News>, diesel::result::Error> {
        let mut conn = self.pool.get().unwrap();

        news::table
            .filter(news::id.eq(news_id))
            .first(&mut conn)
            .optional()
    }

    pub fn find_by_fields(
        &self,
        title: Option<String>,
        feed_id: Option<Uuid>,
    ) -> Result<Option<News>, diesel::result::Error> {
        let mut conn = self.pool.get().unwrap();

        let mut query = news::table.into_boxed();

        if let Some(title) = title {
            query = query.filter(news::title.eq(title));
        }

        if let Some(feed_id) = feed_id {
            query = query.filter(news::feed_id.eq(feed_id));
        }

        query.first(&mut conn).optional()
    }

    pub fn list(&self) -> Result<Vec<News>, diesel::result::Error> {
        let mut conn = self.pool.get().unwrap();

        news::table.load::<News>(&mut conn)
    }

    pub fn delete(&self, news_id: Uuid) -> Result<usize, diesel::result::Error> {
        let mut conn = self.pool.get().unwrap();

        diesel::delete(news::table.find(news_id)).execute(&mut conn)
    }
}
